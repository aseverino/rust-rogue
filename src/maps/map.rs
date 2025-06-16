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
use std::sync::Arc;
use crate::creature::Creature;
use crate::items::container::Container;
use crate::items::base_item::ItemKind;
use crate::maps::{ GRID_HEIGHT, GRID_WIDTH, TILE_SIZE, navigator::Navigator };
use crate::monster::Monster;
use crate::monster_type::MonsterType;
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

// use fov::FovAlgorithm;
// use fov::Map as FovMap;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum PlayerEvent {
    Move,
    AutoMove,
    AutoMoveEnd,
    Wait,
    Cancel,
    MeleeAttack,
    SpellSelect,
    SpellCast,
    OpenChest,
    Death,
    ReachBorder,
}

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

pub struct Map {
    pub tiles: TileMap,
    pub walkable_cache: Vec<Position>,
    pub available_walkable_cache: Vec<Position>,
    pub monsters: Vec<Monster>,
    pub hovered_tile: Option<Position>,
    pub hovered_tile_changed: bool,
    pub last_player_event: Option<PlayerEvent>,
    pub spell_fov_cache: SpellFovCache,
    pub should_draw_spell_fov: bool,
}

impl Map {
    pub fn new(tiles: Vec<Vec<Tile>>, walkable_cache: Vec<Position>, available_walkable_cache: Vec<Position>) -> Self {
        Self {
            tiles: TileMap::new(tiles),
            walkable_cache,
            available_walkable_cache,
            monsters: Vec::new(),
            hovered_tile: None,
            hovered_tile_changed: false,
            last_player_event: None,
            spell_fov_cache: SpellFovCache::new(),
            should_draw_spell_fov: false
        }
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

    pub fn set_player_random_position(&mut self, player: &mut Player) {
        let pos = self.available_walkable_cache.pop()
            .unwrap_or_else(|| Position::new(1, 1)); // Default to (1, 1) if no walkable positions

        self.tiles[pos].creature = PLAYER_CREATURE_ID;
        player.set_pos(pos);

        self.compute_player_fov(player, max(GRID_WIDTH, GRID_HEIGHT));

        let mut positions_around = player.position.positions_around();
        positions_around.shuffle(&mut thread_rng());

        let chest_pos: Option<Position> = positions_around
            .into_iter()
            .find(|&pos| self.is_tile_walkable(pos));

        if let Some(pos) = chest_pos {
            let mut container = Container::new();
            container.add_item(0); // Add some items to the container
            container.add_item(1);
            container.add_item(2);
            self.tiles[pos].items.push(ItemKind::Container(container));
            self.available_walkable_cache.retain(|&p| p != pos); // Remove chest position from available walkable cache
        } else {
            println!("No available position for chest.");
        }
    }

    fn compute_player_fov(&mut self, player: &mut Player, radius: usize) {
        let pos = {
            player.pos()
        };
        let visible = Navigator::compute_fov(&self.tiles, pos, radius);
        player.line_of_sight = visible;
    }

    fn update_spell_fov_cache(&mut self, player: &Player) {
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

    pub fn draw(&mut self, player: &Player, offset: PointF) {
        self.update_spell_fov_cache(player);

        for x in 0..GRID_WIDTH {
            for y in 0..GRID_HEIGHT {
                let tile = &self.tiles[Position::new(x, y)];
                tile.draw(Position::new(x, y), offset);

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

        for monster in &self.monsters {
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
    
    pub(crate) fn add_random_monsters(
        &mut self,
        monster_types: &HashMap<String, Arc<MonsterType>>,
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

        let all_types: Vec<_> = monster_types.values().cloned().collect();

        for pos in positions {
            let kind = all_types
                .choose(&mut rng)
                .expect("Monster type list is empty")
                .clone();

            let monster = Monster::new(pos.clone(), kind);
            self.tiles[pos].creature = self.monsters.len() as i32; // Set the creature ID in the tile
            // Wrap the monster in Rc and push to creatures
            self.monsters.push(monster);
        }

        self.walkable_cache.shuffle(&mut rng);
    }

    fn do_damage(&mut self, player: &mut Player, target_id: u32, damage: i32) {
        let target: &mut dyn Creature = if target_id == PLAYER_CREATURE_ID as u32 {
            player
        } else {
            self.monsters.get_mut(target_id as usize)
                .expect("Target creature not found")
        };
        
        target.add_health(-damage);
        println!("{} takes {} damage!", target.name(), damage);

        if target.get_health().0 <= 0 {
            self.tiles[target.pos()].creature = NO_CREATURE; // Remove monster from tile
            println!("{} has been defeated!", target.name());
            // Optionally, remove the monster from the list
            // self.monsters.remove(target_creature as usize);
        } else {
            println!("{} has {} HP left.", target.name(), target.get_health().0);
        }
    }

    fn do_melee_combat(&mut self, player: &mut Player, _attacker_pos: Position, target_pos: Position) {
        let damage = {
            if let Some(weapon) = &player.equipment.weapon {
                let mut damage: u32 = 0;
                for &d in weapon.borrow().attack_dice.iter() {
                    let mut rng = thread_rng();
                    let roll = rng.gen_range(1..=d);
                    println!("Rolled {} on a {}-sided die", roll, d);
                    damage += roll + weapon.borrow().base_holdable.modifier as u32;
                }
                damage
            }
            else {
                1 as u32
            }
        };

        let creature_id = self.tiles[target_pos].creature;
        if creature_id >= 0 {
            self.do_damage(player, creature_id as u32, damage as i32);
        }
    }

    fn do_spell_combat(&mut self, player: &mut Player, _attacker_pos: Position, target_pos: Position, spell_index: usize) {
        if !self.is_tile_walkable(target_pos) {
            println!("Target position is not walkable for spell casting.");
            return;
        }

        let spell = player.spells.get_mut(spell_index)
            .expect("Selected spell index out of bounds");

        let damage = spell.spell_type.basepower as i32;
        
        let mut target_positions: Vec<Position> = Vec::new();
        let mut target_creatures: Vec<u32> = Vec::new();

        self.spell_fov_cache.area.iter().for_each(|&pos| {
            target_positions.push(pos);
            let creature_id = self.tiles[pos].creature;
            if creature_id >= 0 {
                target_creatures.push(creature_id as u32);
            }
        });

        for target_creature in target_creatures {
            self.do_damage(player, target_creature, damage);
        }

        // let target = self.monsters.get_mut(target_creature as usize)
        //     .expect("Target creature not found");
        // target.hp -= damage;
        // println!("{} takes {} damage!", target.name(), damage);

        // if target.hp <= 0 {
        //     self.tiles[target_pos].creature = NO_CREATURE; // Remove monster from tile
        //     println!("{} has been defeated!", target.name());
        //     // Optionally, remove the monster from the list
        //     // self.monsters.remove(target_creature as usize);
        // } else {
        //     println!("{} has {} HP left.", target.name(), target.hp);
        // }
    }

    pub fn update(&mut self, player: &mut Player, player_action: KeyboardAction, player_direction: Direction, spell_action: i32, player_goal_position: Option<Position>) {
        self.last_player_event = None;
        let player_pos = {
            player.position
        };

        let mut new_player_pos: Option<Position> = None;
        let mut update_monsters = false;

        if let Some(player_goal) = player_goal_position {
            let spell_index = { player.selected_spell };

            if let Some(index) = spell_index {
                let (in_line_of_sight, spell_range) = {
                    let in_line_of_sight = player.line_of_sight.contains(&player_goal);
                    let spell_range = player.spells.get(index)
                        .expect("Selected spell index out of bounds")
                        .spell_type.range;
                    (in_line_of_sight, spell_range)
                };

                let mut should_cast = false;

                if in_line_of_sight && player_pos.in_range(&player_goal, spell_range as usize) {
                    if let Some(spell) = player.spells.get_mut(index) {
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
                    self.do_spell_combat(player, player_pos, player_goal, index);
                    update_monsters = true;
                }

                player.selected_spell = None;
                player.goal_position = None; // Clear goal position
                self.last_player_event = Some(PlayerEvent::SpellCast);
            }
            else {
                let path = Navigator::find_path(player_pos, player_goal, |pos| {
                    self.is_tile_walkable(pos)
                });

                if let Some(path) = path {
                    if path.len() > 1 {
                        new_player_pos = Some(path[1]);
                        self.last_player_event = Some(PlayerEvent::AutoMove);
                    }
                    else {
                        self.last_player_event = Some(PlayerEvent::AutoMoveEnd);
                    }
                    player.goal_position = player_goal_position;
                }
                else {
                    player.goal_position = None; // Clear goal if no path found
                    self.last_player_event = Some(PlayerEvent::AutoMoveEnd);
                }

                
            }
            
        }
        else if player_action == KeyboardAction::SpellSelect && spell_action > 0 {
            let index = spell_action as usize - 1;

            let spell_name = {
                player.spells.get(index).map(|spell| spell.spell_type.name.clone())
            };
            if let Some(name) = spell_name {
                player.selected_spell = Some(index);
                println!("Spell selected: {}", name);
            } else {
                println!("No spell selected!");
            }

            self.last_player_event = Some(PlayerEvent::SpellSelect);
        }
        else if player_action == KeyboardAction::Cancel {
            player.selected_spell = None;
            player.goal_position = None; // Clear goal position

            self.last_player_event = Some(PlayerEvent::Cancel);
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

            if self.is_tile_enemy_occupied(pos) {
                self.last_player_event = Some(PlayerEvent::MeleeAttack);
                update_monsters = true; // Update monsters if player attacks
                self.do_melee_combat(player, player_pos, pos);
            }
            else {
                if self.is_tile_walkable(pos) {
                    new_player_pos = Some(pos);
                    update_monsters = true; // Update monsters if player moves
                }

                self.last_player_event = Some(PlayerEvent::Move);
            }
            player.goal_position = None;
        }
        else if player_action == KeyboardAction::Wait {
            new_player_pos = Some(player_pos); // Stay in place
            player.goal_position = None; // Clear goal position

            self.last_player_event = Some(PlayerEvent::Wait);
        }

        if let Some(pos) = new_player_pos {
            self.tiles[player_pos].creature = NO_CREATURE;
            self.tiles[pos].creature = PLAYER_CREATURE_ID;

            player.set_pos(pos);
            
            if new_player_pos == player.goal_position {
                player.goal_position = None; // Clear goal position if reached
            }

            self.compute_player_fov(player, max(GRID_WIDTH, GRID_HEIGHT));
            update_monsters = true;

            let tile = &mut self.tiles[pos];

            let mut to_remove: Vec<usize> = Vec::new();
            
            for (idx, item) in tile.items.iter().rev().enumerate() {
                match item {
                    ItemKind::Orb(_) => {
                        println!("Player picked up an orb at index {idx}!");
                        player.sp += 1;
                        to_remove.push(idx); // Collect for removal
                    }
                    ItemKind::Container(_) => {
                        self.last_player_event = Some(PlayerEvent::OpenChest);
                        //to_remove.push(idx); // Collect for removal
                    }
                    _ => {
                        // println!("Player found an item: {:?}", other_item);
                    }
                }
            }

            for idx in to_remove {
                tile.remove_item(idx);
            }

            if tile.is_border(&pos) {
                self.last_player_event = Some(PlayerEvent::ReachBorder);
            }

            // for (idx, item) in &tile.items {
            //     match item {
            //         (idx, ItemKind::Orb(orb)) => {
            //             tile.remove_item(*idx);
            //             println!("Player picked up an orb!");
            //             player.sp += 1; // Increase soul points
            //         }
            //         // (_, ItemKind::Portal(_)) => {
                        
            //         // }
            //         (_, other_item) => {
            //             //println!("Player found an item: {:?}", other_item);
            //         }
            //     }
            // }
        }

        if update_monsters {
            for (i, monster) in self.monsters.iter_mut().enumerate() {
                if monster.hp <= 0 {
                    continue; // Skip dead monsters
                }
                let monster_pos = monster.pos();

                let path = Navigator::find_path(monster_pos, player.position, |pos| {
                    pos.x < GRID_WIDTH && pos.y < GRID_HEIGHT && self.tiles[pos].is_walkable()
                });

                if let Some(path) = path {
                    if path.len() > 1 {
                        let next_step = path[1];

                        if next_step == player.position {
                            println!("Monster {} hit player for {} damage!", monster.name(), monster.kind.melee_damage);
                            player.add_health(-monster.kind.melee_damage);
                            if player.hp <= 0 {
                                println!("Player has been defeated!");
                                self.last_player_event = Some(PlayerEvent::Death);
                                return;
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