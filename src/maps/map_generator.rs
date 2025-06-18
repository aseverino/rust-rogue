// SPDX-License-Identifier: MIT
//
// Copyright (c) 2025 Alexandre Severino
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use std::cmp::min;
use std::collections::HashMap;
use std::ops::BitOr;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use bitflags::bitflags;

use rand::rngs::ThreadRng;
use rand::{thread_rng, Rng};

use crate::lua_interface::LuaInterface;
use crate::maps::overworld::OverworldPos;
use crate::maps::{map::Map, GRID_WIDTH, GRID_HEIGHT};
use crate::monster_type::{load_monster_types, MonsterType};
use crate::tile::{Tile, TileKind};
use crate::position::Position;
use rand::seq::SliceRandom;

#[derive(Debug, Clone)]
pub enum MapTheme {
    Any,
    Chasm,
    Wall,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct BorderFlags: u32 {
        const NONE   = 0;
        const TOP    = 0b0001;
        const BOTTOM = 0b0010;
        const LEFT   = 0b0100;
        const RIGHT  = 0b1000;
        const DOWN   = 0b0001_0000;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Border {
    Top,
    Right,
    Bottom,
    Left,
}

impl Border {
    pub fn opposite(self) -> Border {
        match self {
            Border::Top    => Border::Bottom,
            Border::Bottom => Border::Top,
            Border::Left   => Border::Right,
            Border::Right  => Border::Left,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GenerationParams {
    pub num_walks: usize,
    pub walk_length: usize,
    pub min_dist_between_starts: usize,
    pub radius: usize,
    pub borders: BorderFlags,
    pub theme: MapTheme,
    pub predefined_borders: [Vec<Position>; 4],
    pub predefined_start_pos: Option<Position>,
    pub force_regen: bool,
    pub tier: u32
}

impl GenerationParams {
    pub fn default() -> Self {
        Self {
            num_walks: 6,
            walk_length: 80,
            min_dist_between_starts: 6,
            radius: 1, // 0 = 1x1, 1 = 3x3, or even 2 = 5x5
            borders: BorderFlags::NONE,
            theme: MapTheme::Any,
            predefined_borders: [
                Vec::new(), // Up border
                Vec::new(), // Right border
                Vec::new(), // Down border
                Vec::new(), // Left border
            ],
            predefined_start_pos: None,
            force_regen: false,
            tier: 0,
        }
    }
}

enum Command {
    Generate(OverworldPos, GenerationParams),
    Stop,
}

pub struct MapAssignment {
    pub opos: OverworldPos,
    pub map: Map,
}

type MonsterCollection = Vec<Arc<MonsterType>>;
type MonsterTypes = Arc<Mutex<MonsterCollection>>;

pub struct MapGenerator {
    command_tx: Option<Sender<Command>>,
    thread_handle: Option<JoinHandle<()>>,
    monster_types: MonsterTypes,
}

impl MapGenerator {
    pub async fn new(lua_interface: &mut LuaInterface) -> Self {
        Self {
            command_tx: None,
            thread_handle: None,
            monster_types: Arc::new(Mutex::new(load_monster_types(lua_interface).await))
        }
    }

    pub fn set_callback(&mut self, callback: Box<dyn Fn(MapAssignment) + Send + 'static>) {
        let (command_tx, command_rx) = mpsc::channel::<Command>();
        self.command_tx = Some(command_tx);
        let monster_types = Arc::clone(&self.monster_types);
        self.thread_handle = Some(thread::spawn(move || {
            for command in command_rx {
                match command {
                    Command::Generate(pos, params) => {
                        let mut map = Self::generate_map(&params);
                        Self::populate_map(&mut map, &params, &monster_types);
                        callback(MapAssignment { opos: pos, map });
                    }
                    Command::Stop => {
                        println!("[MapGenerator] Stopping...");
                        break;
                    }
                }
            }
        }));
    }

    pub fn request_generation(&mut self, opos: OverworldPos, params: GenerationParams) {
        println!("[MapGenerator] Requesting generation for {:?}", opos);
        if let Some(ref tx) = self.command_tx {
            let _ = tx.send(Command::Generate(opos, params));
        }
    }

    pub fn stop(&mut self) {
        if let Some(ref tx) = self.command_tx {
            let _ = tx.send(Command::Stop);
        }
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }

    fn carve_tile(tiles: &mut Vec<Vec<Tile>>, x: usize, y: usize, walkable_cache: &mut Vec<Position>) {
        if x >= GRID_WIDTH || y >= GRID_HEIGHT {
            return;
        }
        if tiles[x][y].kind != TileKind::Floor {
            tiles[x][y].kind = TileKind::Floor;
            walkable_cache.push(Position { x, y });
        }
    }

    fn carve_straight_path(
        tiles: &mut Vec<Vec<Tile>>,
        start: Position,
        end: Position,
        walkable_cache: &mut Vec<Position>,
    ) {
        let mut x = start.x;
        let mut y = start.y;

        while x != end.x {
            if tiles[x][y].kind != TileKind::Floor {
                tiles[x][y].kind = TileKind::Floor;
                walkable_cache.push(Position { x, y });
            }
            x = if end.x > x { x + 1 } else { x - 1 };
        }

        while y != end.y {
            if tiles[x][y].kind != TileKind::Floor {
                tiles[x][y].kind = TileKind::Floor;
                walkable_cache.push(Position { x, y });
            }
            y = if end.y > y { y + 1 } else { y - 1 };
        }
    }

    fn carve_jagged_path(
        tiles: &mut Vec<Vec<Tile>>,
        mut current: Position,
        goal: Position,
        walkable_cache: &mut Vec<Position>,
        rng: &mut ThreadRng,
        radius: usize, // 0 = 1x1, 1 = 3x3, or even 2 = 5x5
    ) {
        while current != goal {
            if radius == 0 {
                // Exact 1x1
                Self::carve_tile(tiles, current.x, current.y, walkable_cache);
            } else if radius == 1 {
                // Exact 2x2 (square)
                Self::carve_tile(tiles, current.x,     current.y,     walkable_cache);
                Self::carve_tile(tiles, current.x + 1, current.y,     walkable_cache);
                Self::carve_tile(tiles, current.x,     current.y + 1, walkable_cache);
                Self::carve_tile(tiles, current.x + 1, current.y + 1, walkable_cache);
            } else {
                // Circular area
                for dx in -(radius as isize)..=(radius as isize) {
                    for dy in -(radius as isize)..=(radius as isize) {
                        let nx = current.x as isize + dx;
                        let ny = current.y as isize + dy;
                        if nx >= 0 && ny >= 0 && nx < GRID_WIDTH as isize && ny < GRID_HEIGHT as isize {
                            Self::carve_tile(tiles, nx as usize, ny as usize, walkable_cache);
                        }
                    }
                }
            }

            // Movement logic (same as before)
            let dx = goal.x as isize - current.x as isize;
            let dy = goal.y as isize - current.y as isize;

            let mut step_x = 0;
            let mut step_y = 0;

            if rng.gen_bool(0.6) && dx != 0 {
                step_x = dx.signum();
            }
            if rng.gen_bool(0.6) && dy != 0 {
                step_y = dy.signum();
            }

            if step_x == 0 && dx != 0 {
                step_x = dx.signum();
            }
            if step_y == 0 && dy != 0 {
                step_y = dy.signum();
            }

            let new_x = (current.x as isize + step_x).clamp(0, GRID_WIDTH as isize - 1) as usize;
            let new_y = (current.y as isize + step_y).clamp(0, GRID_HEIGHT as isize - 1) as usize;
            current = Position { x: new_x, y: new_y };
        }
    }

    fn place_border_anchors(
        tiles: &mut Vec<Vec<Tile>>,
        params: &GenerationParams,
    ) -> Vec<(Position, Position)> {
        let mut anchors = Vec::new();
        let width = GRID_WIDTH;
        let height = GRID_HEIGHT;
        let mut rng = thread_rng();
        let borders = &params.borders;

        if borders.is_empty() {
            return anchors;
        }

        if borders.contains(BorderFlags::TOP) {
            if !params.predefined_borders[0].is_empty() {
                for &pos in &params.predefined_borders[0] {
                    tiles[pos.x][pos.y].kind = TileKind::Floor;
                    anchors.push((pos, Position::new(pos.x, pos.y + 1)));
                }
            } else {
                let anchor_width = rng.gen_range(1..4);
                let start_x = rng.gen_range(1..width - anchor_width);
                for dx in 0..anchor_width {
                    let border = Position::new(start_x + dx, 0);
                    let neighbor = Position::new(border.x, border.y + 1);
                    tiles[border.x][border.y].kind = TileKind::Floor;
                    anchors.push((border, neighbor));
                }
            }
        }
        if borders.contains(BorderFlags::RIGHT) {
            if !params.predefined_borders[1].is_empty() {
                for &pos in &params.predefined_borders[1] {
                    tiles[pos.x][pos.y].kind = TileKind::Floor;
                    anchors.push((pos, Position::new(pos.x - 1, pos.y)));
                }
            } else {
                let anchor_height = rng.gen_range(1..4);
                let start_y = rng.gen_range(1..height - anchor_height);
                for dy in 0..anchor_height {
                    let border = Position::new(width - 1, start_y + dy);
                    let neighbor = Position::new(border.x - 1, border.y);
                    tiles[border.x][border.y].kind = TileKind::Floor;
                    anchors.push((border, neighbor));
                }
            }
        }
        if borders.contains(BorderFlags::BOTTOM) {
            if !params.predefined_borders[2].is_empty() {
                for &pos in &params.predefined_borders[2] {
                    tiles[pos.x][pos.y].kind = TileKind::Floor;
                    anchors.push((pos, Position::new(pos.x, pos.y - 1)));
                }
            } else {
                let anchor_width = rng.gen_range(1..4);
                let start_x = rng.gen_range(1..width - anchor_width);
                for dx in 0..anchor_width {
                    let border = Position::new(start_x + dx, height - 1);
                    let neighbor = Position::new(border.x, border.y - 1);
                    tiles[border.x][border.y].kind = TileKind::Floor;
                    anchors.push((border, neighbor));
                }
            }
        }
        if borders.contains(BorderFlags::LEFT) {
            if !params.predefined_borders[3].is_empty() {
                for &pos in &params.predefined_borders[3] {
                    tiles[pos.x][pos.y].kind = TileKind::Floor;
                    anchors.push((pos, Position::new(pos.x + 1, pos.y)));
                }
            } else {
                let anchor_height = rng.gen_range(1..4);
                let start_y = rng.gen_range(1..height - anchor_height);
                for dy in 0..anchor_height {
                    let border = Position::new(0, start_y + dy);
                    let neighbor = Position::new(border.x + 1, border.y);
                    tiles[border.x][border.y].kind = TileKind::Floor;
                    anchors.push((border, neighbor));
                }
            }
        }
        
        anchors
    }

    fn generate_map(params: &GenerationParams) -> Map {
        let mut rng = thread_rng();

        let tile_type = match params.theme {
            MapTheme::Any => {
                if rng.gen_bool(0.5) {
                    TileKind::Chasm
                } else {
                    TileKind::Wall
                }
            }
            MapTheme::Chasm => TileKind::Chasm,
            MapTheme::Wall => TileKind::Wall,
        };

        let mut tiles = vec![vec![Tile::new(tile_type); GRID_HEIGHT]; GRID_WIDTH];
        let mut walkable_cache = Vec::new();
        let mut visited = vec![vec![false; GRID_HEIGHT]; GRID_WIDTH];

        //let borders = Self::choose_border_exits(params.exits as usize);
        let anchor_pairs = Self::place_border_anchors(&mut tiles, params);

        let num_walks: usize = params.num_walks;
        let walk_length: usize = params.walk_length;
        let min_distance_between_starts: usize = params.min_dist_between_starts;

        let mut start_positions = Vec::new();

        if let Some(predefined_start_pos) = params.predefined_start_pos {
            start_positions.push(predefined_start_pos);
            println!("Using predefined start position: {:?}", predefined_start_pos);
        }

        for &(_, neighbor) in &anchor_pairs {
            if !visited[neighbor.x][neighbor.y] {
                tiles[neighbor.x][neighbor.y].kind = TileKind::Floor;
                walkable_cache.push(neighbor);
                visited[neighbor.x][neighbor.y] = true;
            }
            start_positions.push(neighbor);
        }

        for i in 0..num_walks + start_positions.len() {
            let mut x;
            let mut y;

            if i >= start_positions.len() {
                // Ensure new walk starts away from previous walks
                loop {
                    x = rng.gen_range(3..GRID_WIDTH - 3);
                    y = rng.gen_range(3..GRID_HEIGHT - 3);

                    let too_close = start_positions.iter().any(|&pos: &Position| {
                        let dx = pos.x as isize - x as isize;
                        let dy = pos.y as isize - y as isize;
                        dx.abs() + dy.abs() < min_distance_between_starts as isize
                    });

                    if !too_close {
                        break;
                    }
                }

                start_positions.push(Position { x, y });
            }
            else {
                // Use existing start position
                x = start_positions[i].x;
                y = start_positions[i].y;
            }

            // Apply random walk
            for _ in 0..walk_length {
                if x >= GRID_WIDTH || y >= GRID_HEIGHT {
                    break;
                }

                if !visited[x][y] {
                    tiles[x][y].kind = TileKind::Floor;
                    walkable_cache.push(Position { x, y });
                    visited[x][y] = true;
                }

                match rng.gen_range(0..8) {
                    0 if x > 1 => x -= 1,
                    1 if x < GRID_WIDTH - 2 => x += 1,
                    2 if y > 1 => y -= 1,
                    3 if y < GRID_HEIGHT - 2 => y += 1,
                    4 if x > 1 && y > 1 => {
                        x -= 1;
                        y -= 1;
                    }
                    5 if x < GRID_WIDTH - 2 && y > 1 => {
                        x += 1;
                        y -= 1;
                    }
                    6 if x > 1 && y < GRID_HEIGHT - 2 => {
                        x -= 1;
                        y += 1;
                    }
                    7 if x < GRID_WIDTH - 2 && y < GRID_HEIGHT - 2 => {
                        x += 1;
                        y += 1;
                    }
                    _ => {}
                }
            }
        }

        start_positions.sort_by_key(|pos| pos.x + pos.y);

        for i in 1..start_positions.len() {
            let prev = start_positions[i - 1];
            let current = start_positions[i];

            //carve_path_between(&mut tiles, prev, current, &mut walkable_cache);
            Self::carve_jagged_path(&mut tiles, prev, current, &mut walkable_cache, &mut rng, params.radius);
        }

        let mut available_walkable_cache = walkable_cache.clone();
        available_walkable_cache.shuffle(&mut rng);
        let mut map = Map::new(tiles, walkable_cache, available_walkable_cache);

        for x in 0..GRID_WIDTH {
            if map.tiles[Position::new(x, 0)].kind == TileKind::Floor {
                map.border_positions[0].push(Position { x, y: 0 });
            }
            if map.tiles[Position::new(x, GRID_HEIGHT - 1)].kind == TileKind::Floor {
                map.border_positions[2].push(Position { x, y: GRID_HEIGHT - 1 });
            }
        }
        for y in 0..GRID_HEIGHT {
            if map.tiles[Position::new(0, y)].kind == TileKind::Floor {
                map.border_positions[3].push(Position { x: 0, y });
            }
            if map.tiles[Position::new(GRID_WIDTH - 1, y)].kind == TileKind::Floor {
                map.border_positions[1].push(Position { x: GRID_WIDTH - 1, y });
            }
        }    

        map
    }

    fn populate_map(map: &mut Map, params: &GenerationParams, monster_types: &MonsterTypes) {
        let monster_types_guard = monster_types.lock().unwrap();
        map.add_random_monsters(&*monster_types_guard, 1);

        let mut len = map.available_walkable_cache.len();
        let mut positions: Vec<Position> = map.available_walkable_cache
            .drain(len.saturating_sub(1)..)
            .collect();

        for pos in positions {
            map.tiles[pos].add_teleport();
            map.downstair_teleport = Some(pos);
        }
        
        len = map.available_walkable_cache.len();
        positions = map.available_walkable_cache
            .drain(len.saturating_sub(2)..)
            .collect();

        for pos in positions {
            map.tiles[pos].add_orb();
        }
    }
}

impl Drop for MapGenerator {
    fn drop(&mut self) {
        self.stop();
    }
}