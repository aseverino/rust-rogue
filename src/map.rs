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

use macroquad::prelude::*;
extern crate rand as external_rand;

use external_rand::Rng;
use external_rand::thread_rng;

use std::cmp::max;
use std::collections::{ HashMap, HashSet };
use crate::creature::Creature;
use crate::monster::Monster;
use crate::monster_type::MonsterType;
use crate::position::{ Position, Direction };
use crate::input::KeyboardAction;
use crate::player::Player;
use crate::tile::{Tile, TileKind, NO_CREATURE, PLAYER_CREATURE_ID};
use external_rand::seq::SliceRandom;
use std::rc::Rc;
use std::cell::RefCell;
use pathfinding::prelude::astar;
use crate::tile_map::TileMap;
use crate::player_spell::PlayerSpell;

// use fov::FovAlgorithm;
// use fov::Map as FovMap;

pub const TILE_SIZE: f32 = 32.0;
pub const GRID_WIDTH: usize = 30;
pub const GRID_HEIGHT: usize = 20;

pub struct Map {
    pub tiles: TileMap,
    pub walkable_cache: Vec<Position>,
    pub player: Player,
    pub monsters: Vec<Monster>,
    pub hovered: Option<Position>,
    pub hovered_changed: bool,
}

pub fn find_path<F>(start_pos: Position, goal_pos: Position, is_walkable: F) -> Option<Vec<Position>>
where
    F: Fn(Position) -> bool,
{
    let start = (start_pos.x, start_pos.y);
    let goal = (goal_pos.x, goal_pos.y);
    let path = astar(
        &start,
        |&(x, y)| {
            let mut neighbors = Vec::new();
            let directions = [
                (0isize, -1), (1, 0), (0, 1), (-1, 0), // cardinal
                (-1, -1), (1, -1), (1, 1), (-1, 1),    // diagonals
            ];

            for (dx, dy) in directions {
                let nx = x as isize + dx;
                let ny = y as isize + dy;
                if nx >= 0 && ny >= 0 {
                    let (ux, uy) = (nx as usize, ny as usize);
                    if is_walkable(Position { x: ux, y: uy }) {
                        // Diagonal steps have slightly higher cost
                        let cost = if dx != 0 && dy != 0 { 14 } else { 10 };
                        neighbors.push(((ux, uy), cost));
                    }
                }
            }

            neighbors
        },
        |&(x, y)| {
            let dx = (goal.0 as isize - x as isize).abs();
            let dy = (goal.1 as isize - y as isize).abs();
            10 * (dx + dy) - 6 * dx.min(dy) // diagonal heuristic
        },
        |&pos| pos == goal,
    )
    .map(|(path, _)| path);

    path.map(|p| p.into_iter().map(|(x, y)| Position { x, y }).collect())
}

impl Map {
    fn is_opaque(&self, x: isize, y: isize) -> bool {
        if x < 0 || y < 0 || x as usize >= GRID_WIDTH || y as usize >= GRID_HEIGHT {
            true // treat out-of-bounds as walls
        } else {
            matches!(self.tiles[Position::new(x as usize, y as usize)].kind, TileKind::Wall)
        }
    }

    fn compute_fov(&self, origin: Position, max_radius: usize) -> HashSet<Position> {
        let mut visible = HashSet::new();
        visible.insert(origin); // Always see self

        for octant in 0..8 {
            self.cast_light(
                &mut visible,
                origin.x as isize,
                origin.y as isize,
                1,
                1.0,
                0.0,
                max_radius as isize,
                octant,
            );
        }

        visible
    }

    fn cast_light(
        &self,
        visible: &mut HashSet<Position>,
        cx: isize,
        cy: isize,
        row: isize,
        mut start_slope: f64,
        end_slope: f64,
        radius: isize,
        octant: u8,
    ) {
        if start_slope < end_slope {
            return;
        }

        let mut next_start_slope = start_slope;

        for i in row..=radius {
            let mut blocked = false;
            let mut dx = -i;
            while dx <= 0 {
                let dy = -i;
                let (nx, ny) = match octant {
                    0 => (cx + dx, cy + dy),
                    1 => (cx + dy, cy + dx),
                    2 => (cx + dy, cy - dx),
                    3 => (cx + dx, cy - dy),
                    4 => (cx - dx, cy - dy),
                    5 => (cx - dy, cy - dx),
                    6 => (cx - dy, cy + dx),
                    7 => (cx - dx, cy + dy),
                    _ => (cx, cy),
                };

                let l_slope = (dx as f64 - 0.5) / (dy as f64 + 0.5);
                let r_slope = (dx as f64 + 0.5) / (dy as f64 - 0.5);

                if r_slope > start_slope {
                    dx += 1;
                    continue;
                } else if l_slope < end_slope {
                    break;
                }

                let distance = dx * dx + dy * dy;
                if nx >= 0 && ny >= 0 && nx < GRID_WIDTH as isize && ny < GRID_HEIGHT as isize {
                    let is_blocking = self.is_opaque(nx, ny);
                    
                    // Only insert tiles that are not opaque
                    if !is_blocking && distance <= radius * radius {
                        visible.insert(Position {
                            x: nx as usize,
                            y: ny as usize,
                        });
                    }
                }

                if blocked {
                    if self.is_opaque(nx, ny) {
                        next_start_slope = r_slope;
                        dx += 1;
                        continue;
                    } else {
                        blocked = false;
                        start_slope = next_start_slope;
                    }
                } else {
                    if self.is_opaque(nx, ny) {
                        blocked = true;
                        self.cast_light(visible, cx, cy, i + 1, next_start_slope, l_slope, radius, octant);
                        next_start_slope = r_slope;
                    }
                }

                dx += 1;
            }

            if blocked {
                break;
            }
        }
    }


    pub fn generate(player: Player, monster_types: &HashMap<String, Rc<MonsterType>>) -> Self {
        let mut rng = thread_rng();
        let mut walkable_cache= Vec::new();

        let mut tiles = Vec::with_capacity(GRID_WIDTH);
        for x in 0..GRID_WIDTH {
            let mut column = Vec::with_capacity(GRID_HEIGHT);
            for y in 0..GRID_HEIGHT {
                let roll = rng.gen_range(0..100);
                let kind = match roll {
                    0..=65 => TileKind::Floor,
                    66..=85 => TileKind::Wall,
                    _ => TileKind::Chasm,
                };

                let pos = Position { x, y };
                
                if kind == TileKind::Floor {
                    walkable_cache.push(pos);
                }
                column.push(Tile::new(kind));
            }
            tiles.push(column);
        }

        tiles[1][1].kind = TileKind::Floor;

        let mut map = Self {
            tiles: TileMap::new(tiles),
            walkable_cache,
            monsters: Vec::new(),
            player,
            hovered: None,
            hovered_changed: false,
        };

        map.compute_player_fov(max(GRID_WIDTH, GRID_HEIGHT));
        map.add_random_monsters(monster_types, 10);
        map
    }

    fn compute_player_fov(&mut self, radius: usize) {
        let pos = {
            self.player.pos()
        };
        let visible = self.compute_fov(pos, radius);
        self.player.line_of_sight = visible;
    }

    fn has_line_of_sight(&self, from: Position, to: Position) -> bool {
        let mut x0 = from.x as isize;
        let mut y0 = from.y as isize;
        let x1 = to.x as isize;
        let y1 = to.y as isize;

        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        loop {
            if x0 == x1 && y0 == y1 {
                break; // reached target tile
            }

            // Skip the origin tile
            if !(x0 == from.x as isize && y0 == from.y as isize) {
                // Out of bounds safety
                if x0 < 0 || y0 < 0 || x0 >= GRID_WIDTH as isize || y0 >= GRID_HEIGHT as isize {
                    return false;
                }

                let tile = &self.tiles[Position::new(x0 as usize, y0 as usize)];
                if matches!(tile.kind, TileKind::Wall) {
                    return false;
                }
            }

            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }

        true
    }

    pub fn draw(&self) {
        for x in 0..GRID_WIDTH {
            for y in 0..GRID_HEIGHT {
                let tile = &self.tiles[Position::new(x, y)];
                let color = match tile.kind {
                    TileKind::Floor => Color { r: 0.5, g: 0.5, b: 0.5, a: 1.0 },
                    TileKind::Wall => Color { r: 0.3, g: 0.3, b: 0.3, a: 1.0 },
                    TileKind::Chasm => Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 },
                };

                draw_rectangle(
                    x as f32 * TILE_SIZE,
                    y as f32 * TILE_SIZE + 40.0,
                    TILE_SIZE - 1.0,
                    TILE_SIZE - 1.0,
                    color,
                );

                if let Some(selected_spell) = { self.player.selected_spell } {
                    let player_pos = self.player.pos();
                    let tile_pos = Position { x, y };
                    if let Some(spell) = self.player.spells.get(selected_spell) {
                        if spell.spell_type.range > 0 && player_pos.in_range(&tile_pos, spell.spell_type.range as usize) &&
                        self.player.line_of_sight.contains(&tile_pos) {
                            draw_rectangle(
                                x as f32 * TILE_SIZE,
                                y as f32 * TILE_SIZE + 40.0,
                                TILE_SIZE - 1.0,
                                TILE_SIZE - 1.0,
                                Color { r: 0.0, g: 1.0, b: 0.0, a: 0.2 },
                            );
                        }
                    }
                }
                
                if let Some(pos) = self.hovered {
                    draw_rectangle_lines(
                        pos.x as f32 * TILE_SIZE,
                        pos.y as f32 * TILE_SIZE + 40.0,
                        TILE_SIZE,
                        TILE_SIZE,
                        2.0,
                        YELLOW,
                    );
                }
            }
        }

        for monster in &self.monsters {
            monster.draw();
        }

        self.player.draw();
    }

    pub fn is_tile_walkable(&self, pos: Position) -> bool {
        pos.x < GRID_WIDTH && pos.y < GRID_HEIGHT && self.tiles[pos].is_walkable()
    }
    
    pub(crate) fn add_random_monsters(
        &mut self,
        monster_types: &HashMap<String, Rc<MonsterType>>,
        count: usize,
    ) {
        let mut rng = thread_rng();

        let mut positions = self.walkable_cache.clone(); // clone so we can shuffle safely
        // 2. Shuffle the positions randomly
        positions.shuffle(&mut rng);

        // 3. Pick up to `count` positions
        let positions = positions.into_iter().take(count);

        let all_types: Vec<_> = monster_types.values().cloned().collect();

        for pos in positions {
            let kind = all_types
                .choose(&mut rng)
                .expect("Monster type list is empty")
                .clone();

            // Wrap the monster in Rc and push to creatures
            self.monsters.push(Monster::new(pos, kind));
        }
    }

    pub fn do_combat(&mut self, attacker_pos: Position, target_pos: Position, spell_index: usize) {
        if !self.has_line_of_sight(attacker_pos, target_pos) {
            println!("Target is out of line of sight!");
            return;
        }

        let spell = self.player.spells.get_mut(spell_index)
            .expect("Selected spell index out of bounds");

        let damage = spell.spell_type.basepower as i32;
        let target_creature = self.tiles[target_pos].creature;

        if target_creature == NO_CREATURE || target_creature == PLAYER_CREATURE_ID {
            println!("No monster at target position to attack!");
            return;
        }

        let target = self.monsters.get_mut(target_creature as usize)
            .expect("Target creature not found");
        target.hp -= damage;
        println!("{} takes {} damage!", target.name(), damage);

        if target.hp <= 0 {
            self.tiles[target_pos].creature = NO_CREATURE; // Remove monster from tile
            println!("{} has been defeated!", target.name());
            // Optionally, remove the monster from the list
            // self.monsters.remove(target_creature as usize);
        } else {
            println!("{} has {} HP left.", target.name(), target.hp);
        }
    }

    pub fn update(&mut self, player_action: KeyboardAction, player_direction: Direction, spell_action: i32, player_goal_position: Option<Position>) -> bool {
        let player_pos = {
            self.player.position
        };

        let mut new_player_pos: Option<Position> = None;
        let mut update_monsters = false;

        if let Some(player_goal) = player_goal_position {
            let spell_index = { self.player.selected_spell };

            if let Some(index) = spell_index {
                let (in_line_of_sight, spell_range) = {
                    let in_line_of_sight = self.player.line_of_sight.contains(&player_goal);
                    let spell_range = self.player.spells.get(index)
                        .expect("Selected spell index out of bounds")
                        .spell_type.range;
                    (in_line_of_sight, spell_range)
                };

                let mut should_cast = false;

                if in_line_of_sight && player_pos.in_range(&player_goal, spell_range as usize) {
                    if let Some(spell) = self.player.spells.get_mut(index) {
                        if spell.charges > 0 {
                            spell.charges -= 1;
                            println!("Casting spell charges {}", spell.charges);
                            should_cast = true;
                        }
                        else {
                            println!("No charges left for this spell!");
                        }
                    }
                }

                if should_cast {
                    self.do_combat(player_pos, player_goal, index);
                    update_monsters = true;
                }

                self.player.selected_spell = None;
                self.player.goal_position = None; // Clear goal position
            }
            else {
                let path = find_path(player_pos, player_goal, |pos| {
                    self.is_tile_walkable(pos)
                });

                if let Some(path) = path {
                    if path.len() > 1 {
                        new_player_pos = Some(path[1]);
                    }
                    self.player.goal_position = player_goal_position;
                }
                else {
                    self.player.goal_position = None; // Clear goal if no path found
                }
            }
            
        }
        else if player_action == KeyboardAction::SpellSelect && spell_action > 0 {
            let index = spell_action as usize - 1;

            let spell_name = {
                self.player.spells.get(index).map(|spell| spell.spell_type.name.clone())
            };
            if let Some(name) = spell_name {
                self.player.selected_spell = Some(index);
                println!("Spell selected: {}", name);
                return true;
            } else {
                println!("No spell selected!");
                return false;
            }
        }
        else if player_action == KeyboardAction::Cancel {
            self.player.selected_spell = None;
            self.player.goal_position = None; // Clear goal position
            return true;
        }
        else if player_action == KeyboardAction::Move {
            let pos_change = match player_direction {
                Direction::Up => (0, -1),
                Direction::Right => (1, 0),
                Direction::Down => (0, 1),
                Direction::Left => (-1, 0),
                Direction::UpRight => (1, -1),
                Direction::DownRight => (1, 1),
                Direction::DownLeft => (-1, 1),
                Direction::UpLeft => (-1, -1),
                Direction::None => (0, 0),
            };

            let pos = Position {
                x: (player_pos.x as isize + pos_change.0) as usize,
                y: (player_pos.y as isize + pos_change.1) as usize
            };

            if self.is_tile_walkable(pos) {
                new_player_pos = Some(pos);
                update_monsters = true; // Update monsters if player moves
            }

            self.player.goal_position = None;
        }
        else if player_action == KeyboardAction::Wait {
            new_player_pos = Some(player_pos); // Stay in place
            self.player.goal_position = None; // Clear goal position
        }

        if let Some(pos) = new_player_pos {
            self.tiles[player_pos].creature = NO_CREATURE;
            self.tiles[pos].creature = PLAYER_CREATURE_ID;

            self.player.set_pos(pos);
            
            if new_player_pos == self.player.goal_position {
                self.player.goal_position = None; // Clear goal position if reached
            }

            self.compute_player_fov(max(GRID_WIDTH, GRID_HEIGHT));
            update_monsters = true;
        }

        if update_monsters {
            for (i, monster) in self.monsters.iter_mut().enumerate() {
                if monster.hp <= 0 {
                    continue; // Skip dead monsters
                }
                let monster_pos = monster.pos();

                let path = find_path(monster_pos, self.player.position, |pos| {
                    pos.x < GRID_WIDTH && pos.y < GRID_HEIGHT && self.tiles[pos].is_walkable()
                });

                if let Some(path) = path {
                    if path.len() > 1 {
                        let next_step = path[1];

                        if next_step == self.player.position {
                            monster.hp -= 1;
                            println!("Monster {} hit!", monster.name());
                            if monster.hp <= 0 {
                                self.tiles[monster_pos].creature = NO_CREATURE;
                                // self.monsters.remove(i);
                                // todo death
                            }
                            continue;
                        }

                        self.tiles[monster_pos].creature = NO_CREATURE;
                        self.tiles[next_step].creature = i as i32;
                        
                        monster.set_pos(next_step);
                    }
                }
            }
        }

        true
    }
}

// impl FovMap for Map {
//     fn is_opaque(&self, x: i32, y: i32) -> bool {
//         if x < 0 || y < 0 || x as usize >= GRID_WIDTH || y as usize >= GRID_HEIGHT {
//             return true;
//         }
//         matches!(self.tiles[x as usize][y as usize].kind, TileKind::Wall)
//     }
// }

// pub fn compute_visible_positions(map: &Map, origin: Position, radius: i32) -> Vec<Position> {
//     let mut visible = Vec::new();
//     fov::compute_fov(origin.x as i32, origin.y as i32, radius, &*map, |x, y| {
//         visible.push(Position { x: x as usize, y: y as usize });
//     });
//     visible
// }