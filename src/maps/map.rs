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

use std::cell::RefCell;
use std::cmp::max;
use std::collections::{ HashMap, HashSet };
use std::sync::Arc;
use std::sync::RwLock;
use crate::creature::Creature;
use crate::items::container::Container;
use crate::items::base_item::ItemKind;
use crate::lua_interface;
use crate::lua_interface::LuaInterfaceRc;
use crate::lua_interface::LuaScripted;
use crate::maps::{ GRID_HEIGHT, GRID_WIDTH, TILE_SIZE, navigator::Navigator };
use crate::monster::Monster;
use crate::monster::MonsterArc;
use crate::monster::MonsterRef;
use crate::monster_type::MonsterType;
use crate::player;
use crate::position::POSITION_INVALID;
use crate::position::{ Position, Direction };
use crate::input::KeyboardAction;
use crate::player::Player;
use crate::tile::{Tile, NO_CREATURE, PLAYER_CREATURE_ID};
use crate::ui::point_f::PointF;
use external_rand::seq::SliceRandom;
use std::rc::Rc;
use crate::tile_map::TileMap;
use crate::monster_type::load_monster_types;

#[derive(Debug)]
pub struct SpellFovCache {
    pub radius: u32,
    pub origin: Position,
    pub area: HashSet<Position>,
}

impl SpellFovCache {
    pub fn new() -> Self {
        Self {
            radius: 0,
            origin: POSITION_INVALID,
            area: HashSet::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct GeneratedMap {
    pub tiles: TileMap,
    pub walkable_cache: Vec<Position>,
    pub available_walkable_cache: Vec<Position>,
    pub monsters: Vec<MonsterArc>,
    pub border_positions: [Vec<Position>; 4],
    pub downstair_teleport: Option<Position>,
}

impl GeneratedMap {
    pub fn new(tiles: Vec<Vec<Tile>>, walkable_cache: Vec<Position>, available_walkable_cache: Vec<Position>) -> Self {
        Self {
            tiles: TileMap::new(tiles),
            walkable_cache,
            available_walkable_cache,
            monsters: Vec::new(),
            border_positions: [
                Vec::new(), // Up border
                Vec::new(), // Right border
                Vec::new(), // Down border
                Vec::new(), // Left border
            ],
            downstair_teleport: None,
        }
    }

    pub(crate) fn add_random_monsters(
        &mut self,
        monster_types: &Vec<Arc<MonsterType>>,
        count: usize,
    ) {
        let mut rng = thread_rng();

        // let mut positions = self.walkable_cache.clone(); // clone so we can shuffle safely
        // 2. Shuffle the positions randomly
        // positions.shuffle(&mut rng);

        // 3. Pick up to `count` positions
        let len = self.available_walkable_cache.len();
        let positions: Vec<Position> = self.available_walkable_cache
            .drain(len.saturating_sub(count)..)
            .collect();

        for pos in positions {
            let kind = monster_types
                .choose(&mut rng)
                .expect("Monster type list is empty")
                .clone();

            let monster = Arc::new(RwLock::new(Monster::new(pos.clone(), kind)));
            if let Ok(monster_guard) = monster.read() {
                self.tiles[pos].creature = monster_guard.id;
            } else {
                println!("Failed to read monster data due to poisoning.");
            }
            // Wrap the monster in Rc and push to creatures
            self.monsters.push(monster);
        }

        self.walkable_cache.shuffle(&mut rng);
    }
}

#[derive(Debug)]
pub struct Map {
    pub tiles: TileMap,
    pub walkable_cache: Vec<Position>,
    pub available_walkable_cache: Vec<Position>,
    pub monsters: HashMap<u32, Monster>,
    pub hovered_tile: Option<Position>,
    pub hovered_tile_changed: bool,
    pub spell_fov_cache: SpellFovCache,
    pub should_draw_spell_fov: bool,
    pub border_positions: [Vec<Position>; 4],
    pub downstair_teleport: Option<Position>,
}

impl Map {
    pub fn new(tiles: TileMap, walkable_cache: Vec<Position>, available_walkable_cache: Vec<Position>) -> Self {
        Self {
            tiles,
            walkable_cache,
            available_walkable_cache,
            monsters: HashMap::new(),
            hovered_tile: None,
            hovered_tile_changed: false,
            spell_fov_cache: SpellFovCache::new(),
            should_draw_spell_fov: false,
            border_positions: [
                Vec::new(), // Up border
                Vec::new(), // Right border
                Vec::new(), // Down border
                Vec::new(), // Left border
            ],
            downstair_teleport: None,
        }
    }

    fn convert_monsters(monsters: Vec<MonsterArc>) -> HashMap<u32, Monster> 
    where
        Monster: Clone,
    {
        monsters
            .into_iter()
            .map(|arc_mutex| {
                let monster = arc_mutex.read().unwrap();
                (monster.id, monster.clone())
            })
            .collect()
    }

    pub fn from_generated_map(generated_map: GeneratedMap) -> Self {
        let mut map = Self::new(
            generated_map.tiles,
            generated_map.walkable_cache,
            generated_map.available_walkable_cache,
        );
        map.monsters = Self::convert_monsters(generated_map.monsters);
        map.border_positions = generated_map.border_positions;
        map.downstair_teleport = generated_map.downstair_teleport;
        map
    }

    //pub async fn init(&mut self, _player: &mut Player) {
    //    let monster_types = load_monster_types().await;
    //    self.add_random_monsters(&monster_types, 20);
    //    
    //    let len = self.available_walkable_cache.len();
    //    let positions: Vec<Position> = self.available_walkable_cache
    //        .drain(len.saturating_sub(2)..)
    //        .collect();
//
    //    for pos in positions {
    //        self.tiles[pos].add_orb();
    //    }

        

        // let chest_pos = self.available_walkable_cache.pop();

        // if let Some(pos) = chest_pos {
        //     let mut container = Container::new();
        //     container.add_item(0);
        //     container.add_item(1);
        //     container.add_item(2);
        //     self.tiles[pos].items.push_back(ItemKind::Container(container));
        // } else {
        //     println!("No available position for chest.");
        // }
    //}

    pub fn remove_creature<T: Creature>(&mut self, creature: &mut T) {
        let pos = creature.pos();
        if pos.x < GRID_WIDTH && pos.y < GRID_HEIGHT {
            self.tiles[pos].creature = NO_CREATURE; // Remove creature from tile
            creature.set_pos(POSITION_INVALID); // Set creature position to invalid
        } else {
            println!("Creature position out of bounds, cannot remove.");
        }
    }

    pub fn add_player(&mut self, player: &mut Player, pos: Position) {
        if !self.is_tile_walkable(pos) {
            println!("Position is not walkable, cannot set player position.");
            return;
        }

        self.tiles[pos].creature = PLAYER_CREATURE_ID;
        player.set_pos(pos);

        self.compute_player_fov(player, max(GRID_WIDTH, GRID_HEIGHT));
    }

    pub fn add_player_first_map(&mut self, player: &mut Player) {
        let pos = self.available_walkable_cache.pop()
            .unwrap_or_else(|| Position::new(1, 1)); // Default to (1, 1) if no walkable positions

        self.add_player(player, pos);

        let mut positions_around = player.position.positions_around();
        positions_around.shuffle(&mut thread_rng());

        let chest_pos: Option<Position> = positions_around
            .into_iter()
            .find(|&pos| self.is_tile_walkable(pos));

        if let Some(pos) = chest_pos {
            let mut container = Container::new();
            for i in 1..6 {
                container.add_item(i);
            }
            self.tiles[pos].items.push(ItemKind::Container(container));
            self.available_walkable_cache.retain(|&p| p != pos); // Remove chest position from available walkable cache
        } else {
            println!("No available position for chest.");
        }
    }

    pub fn compute_player_fov(&mut self, player: &mut Player, radius: usize) {
        let pos = {
            player.pos()
        };
        let visible = Navigator::compute_fov(&self.tiles, pos, radius);
        player.line_of_sight = visible;
    }

    fn update_spell_fov_cache(&mut self, player: &mut Player) {
        self.should_draw_spell_fov = false;
        let mut spell_fov_needs_update = false;
        if let Some(selected_spell) = player.selected_spell {
            if let Some(player_spell) = player.spells.get(selected_spell) {
                if let Some(hovered) = self.hovered_tile {
                    let spell_type = &player_spell.spell_type;
                    let radius = self.spell_fov_cache.radius;
                    if spell_type.area_radius != Some(radius) {
                        spell_fov_needs_update = true;
                    }
                    else if hovered != self.spell_fov_cache.origin {
                        spell_fov_needs_update = true;
                    }
                    self.should_draw_spell_fov = true;
                }
            }
        }

        if spell_fov_needs_update {
            self.spell_fov_cache.radius = player.spells[player.selected_spell.unwrap()].spell_type.area_radius.unwrap_or(0);
            self.spell_fov_cache.origin = self.hovered_tile.unwrap_or(POSITION_INVALID);
            self.spell_fov_cache.area = Navigator::compute_fov(&self.tiles, self.spell_fov_cache.origin, self.spell_fov_cache.radius as usize);
        }
    }

    pub fn draw(&mut self, player: &mut Player, offset: PointF) {
        self.update_spell_fov_cache(player);

        for x in 0..GRID_WIDTH {
            for y in 0..GRID_HEIGHT {
                let tile = &self.tiles[Position::new(x, y)];
                tile.draw(Position::new(x, y), offset, !self.monsters.is_empty());

                if self.should_draw_spell_fov {
                    let player_pos = player.pos();
                    let tile_pos = Position { x, y };
                    if let Some(spell) = player.spells.get(player.selected_spell.unwrap()) {
                        if spell.spell_type.range > 0 && player_pos.in_range(&tile_pos, spell.spell_type.range as usize) &&
                        player.line_of_sight.contains(&tile_pos) {
                            draw_rectangle(
                                offset.x + x as f32 * TILE_SIZE,
                                offset.y + y as f32 * TILE_SIZE,
                                TILE_SIZE - 1.0,
                                TILE_SIZE - 1.0,
                                Color { r: 0.0, g: 1.0, b: 0.0, a: 0.2 },
                            );
                        }
                    }

                    if self.spell_fov_cache.area.contains(&Position { x, y }) {
                        draw_rectangle(
                            offset.x + x as f32 * TILE_SIZE,
                            offset.y + y as f32 * TILE_SIZE,
                            TILE_SIZE - 1.0,
                            TILE_SIZE - 1.0,
                            Color { r: 0.0, g: 0.0, b: 1.0, a: 0.5 });
                    }
                }
                
                // if self.player.selected_spell.is_some() {
                //     self.in_spell_area(Position { x, y });
                //     if let Some(pos) = self.hovered {
                //         draw_rectangle_lines(
                //             offset.0 + x as f32 * TILE_SIZE,
                //             offset.1 + y as f32 * TILE_SIZE,
                //             TILE_SIZE,
                //             TILE_SIZE,
                //             2.0,
                //             YELLOW,
                //         );
                //     }
                // }
            }
        }

        for (_, monster) in &self.monsters {
            monster.draw(offset);
        }

        player.draw(offset);
    }

    pub fn is_tile_enemy_occupied(&self, pos: Position) -> bool {
        pos.x < GRID_WIDTH && pos.y < GRID_HEIGHT && self.tiles[pos].has_enemy()
    }

    pub fn is_tile_walkable(&self, pos: Position) -> bool {
        pos.x < GRID_WIDTH && pos.y < GRID_HEIGHT && self.tiles[pos].is_walkable()
    }

    pub fn get_chest_items(&self, position: &Position) -> Option<&Vec<u32>> {
        if position.x < GRID_WIDTH && position.y < GRID_HEIGHT {
            let tile = &self.tiles[*position];
            if let Some(item) = tile.get_top_item() {
                if let ItemKind::Container(container) = item {
                    return Some(&container.items);
                }
            }
        }
        None
    }

    pub fn remove_chest(&mut self, position: Position) {
        for (idx, item) in self.tiles[position].items.iter().enumerate() {
            if let ItemKind::Container(_) = item {
                self.tiles[position].items.remove(idx);
                return; // Exit after removing the first container
            }
        }
    }
}

pub type MapRef = Rc<RefCell<Map>>;

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