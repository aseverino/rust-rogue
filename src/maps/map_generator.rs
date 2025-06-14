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

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use rand::rngs::ThreadRng;
use rand::{thread_rng, Rng};

use crate::maps::{map::Map, GRID_WIDTH, GRID_HEIGHT};
use crate::tile::{Tile, TileKind};
use crate::position::Position;
use rand::seq::SliceRandom;

#[derive(Debug, Clone)]
pub enum MapTheme {
    Any,
    Chasm,
    Wall,
}

#[derive(Debug, Clone)]
pub struct GenerationParams {
    pub num_walks: usize,
    pub walk_length: usize,
    pub min_dist_between_starts: usize,
    pub radius: usize,
    pub exits: u8,
    pub theme: MapTheme,
}

impl GenerationParams {
    pub fn default() -> Self {
        Self {
            num_walks: 6,
            walk_length: 80,
            min_dist_between_starts: 6,
            radius: 1, // 0 = 1x1, 1 = 3x3, or even 2 = 5x5
            exits: 4,
            theme: MapTheme::Any,
        }
    }
}

enum Command {
    Generate(GenerationParams),
    Stop,
}

pub struct MapGenerator {
    command_tx: Sender<Command>,
    result_rx: Arc<Mutex<Receiver<Map>>>,
    thread_handle: Option<JoinHandle<()>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Border {
    Top,
    Bottom,
    Left,
    Right,
}

impl MapGenerator {
    pub fn new() -> Self {
        let (command_tx, command_rx) = mpsc::channel::<Command>();
        let (result_tx, result_rx) = mpsc::channel::<Map>();
        let result_rx = Arc::new(Mutex::new(result_rx));
        let result_tx_clone = result_tx.clone();

        let handle = {
            let result_tx = result_tx_clone.clone();
            thread::spawn(move || {
                for command in command_rx {
                    match command {
                        Command::Generate(params) => {
                            let map = Self::generate_map(params);
                            let _ = result_tx.send(map); // send result back
                        }
                        Command::Stop => {
                            println!("[MapGenerator] Stopping...");
                            break;
                        }
                    }
                }
            })
        };

        Self {
            command_tx,
            result_rx,
            thread_handle: Some(handle),
        }
    }

    pub fn request_generation(&self, params: GenerationParams) {
        let _ = self.command_tx.send(Command::Generate(params));
    }

    // pub fn get_generated_map(&self) -> Option<Map> {
    //     // Try to receive without blocking
    //     self.result_rx.lock().unwrap().recv().ok()
    // }

    pub fn get_generated_map_blocking(&self) -> Option<Map> {
        self.result_rx.lock().unwrap().recv().ok()
    }

    pub fn stop(&mut self) {
        let _ = self.command_tx.send(Command::Stop);
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

    fn choose_border_exits(count: usize) -> Vec<Border> {
        use Border::*;
        let all = vec![Top, Bottom, Left, Right];
        let mut rng = thread_rng();
        all.choose_multiple(&mut rng, count).cloned().collect()
    }

    fn place_border_anchors(tiles: &mut Vec<Vec<Tile>>, borders: &[Border]) -> Vec<Position> {
        let mut anchors = Vec::new();
        let width = GRID_WIDTH as usize;
        let height = GRID_HEIGHT as usize;

        let mut rng = thread_rng();
        for border in borders {
            let pos = match border {
                Border::Top => Position::new(rng.gen_range(1..width - 1), 0),
                Border::Bottom => Position::new(rng.gen_range(1..width - 1), height - 1),
                Border::Left => Position::new(0, rng.gen_range(1..height - 1)),
                Border::Right => Position::new(width - 1, rng.gen_range(1..height - 1)),
            };
            tiles[pos.x][pos.y].kind = TileKind::Floor;
            anchors.push(pos);
        }

        anchors
    }

    pub fn generate_map(params: GenerationParams) -> Map {
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

        let borders = Self::choose_border_exits(params.exits as usize);
        let anchor_points = Self::place_border_anchors(&mut tiles, &borders);

        let num_walks: usize = params.num_walks;
        let walk_length: usize = params.walk_length;
        let min_distance_between_starts: usize = params.min_dist_between_starts;

        let mut start_positions = Vec::new();

        for &anchor in &anchor_points {
            if !visited[anchor.x][anchor.y] {
                tiles[anchor.x][anchor.y].kind = TileKind::Floor;
                walkable_cache.push(anchor);
                visited[anchor.x][anchor.y] = true;
            }
            start_positions.push(anchor);
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

        let map = Map::new(tiles, walkable_cache, available_walkable_cache);

        map
    }
}

impl Drop for MapGenerator {
    fn drop(&mut self) {
        self.stop();
    }
}