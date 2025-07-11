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

use external_rand::thread_rng;
use mlua::{Table, UserData, UserDataMethods};

use crate::creature::Creature;
use crate::graphics;
use crate::graphics::graphics_manager::GraphicsManager;
use crate::items::base_item::ItemKind;
use crate::items::container::Container;
use crate::lua_interface::LuaInterface;
use crate::maps::generated_map::GeneratedMap;
use crate::maps::overworld::VisitedState;
use crate::maps::{GRID_HEIGHT, GRID_WIDTH, TILE_SIZE, navigator::Navigator};
use crate::monster::MonsterArc;
use crate::monster::{Monster, MonsterRc};
use crate::player::Player;
use crate::position::POSITION_INVALID;
use crate::position::Position;
use crate::spell_type::SpellStrategy;
use crate::tile::{NO_CREATURE, PLAYER_CREATURE_ID};
use crate::ui::point_f::PointF;
use external_rand::seq::SliceRandom;
use std::cell::RefCell;
use std::cmp::max;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::{Arc, RwLock};

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug, PartialEq)]
pub enum FovToShow {
    None,
    Attack,
    Spell,
}

#[derive(Clone, Debug)]
pub struct Map {
    pub generated_map: GeneratedMap,
    pub monsters: HashMap<u32, MonsterRc>,
    pub hovered_tile: Option<Position>,
    pub hovered_tile_changed: bool,
    pub spell_or_attack_fov_cache: SpellFovCache,
    pub shown_fov: FovToShow,
}

impl Map {
    pub fn new(generated_map: GeneratedMap) -> Self {
        let mut m = Self {
            generated_map,
            monsters: HashMap::new(),
            hovered_tile: None,
            hovered_tile_changed: false,
            spell_or_attack_fov_cache: SpellFovCache::new(),
            shown_fov: FovToShow::None,
        };

        m.monsters = Self::convert_monsters(m.generated_map.monsters.clone());
        m
    }

    fn convert_monsters(monsters: Vec<MonsterArc>) -> HashMap<u32, MonsterRc> {
        monsters
            .into_iter()
            .map(|arc_mutex| {
                let monster = arc_mutex.read().unwrap();
                (monster.id, Rc::new(RefCell::new(monster.clone())))
            })
            .collect()
    }

    pub fn remove_creature<T: Creature>(&mut self, creature: &mut T) {
        let pos = creature.pos();
        if pos.x < GRID_WIDTH && pos.y < GRID_HEIGHT {
            self.generated_map.tiles[pos].creature = NO_CREATURE; // Remove creature from tile
            creature.set_pos(POSITION_INVALID); // Set creature position to invalid
        } else {
            println!("Creature position out of bounds, cannot remove.");
        }
    }

    pub fn remove_downstairs_teleport(&mut self) {
        //self.generated_map.tiles[self.generated_map.downstair_teleport].remove;
        if let Some(teleport_pos) = self.generated_map.downstair_teleport {
            let items_to_remove: Vec<_> = self.generated_map.tiles[teleport_pos]
                .items
                .iter()
                .filter(|item| matches!(item, ItemKind::Teleport(_)))
                .cloned()
                .collect();

            for item in items_to_remove {
                self.generated_map.tiles[teleport_pos]
                    .items
                    .retain(|i| *i != item);
            }
        } else {
            println!("Downstairs teleport position is not set.");
        }

        self.generated_map.downstair_teleport = None; // Clear the teleport position
    }

    pub fn add_player(&mut self, player: &mut Player, pos: Position) {
        if !self.is_tile_walkable(pos) {
            println!("Position is not walkable, cannot set player position.");
            return;
        }

        self.generated_map.tiles[pos].creature = PLAYER_CREATURE_ID;
        player.set_pos(pos);

        self.compute_player_fov(player, max(GRID_WIDTH, GRID_HEIGHT));
    }

    pub fn add_player_first_map(&mut self, player: &mut Player) {
        let pos = self
            .generated_map
            .available_walkable_cache
            .pop()
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
            self.generated_map.tiles[pos]
                .items
                .push(ItemKind::Container(container));
            self.generated_map
                .available_walkable_cache
                .retain(|&p| p != pos); // Remove chest position from available walkable cache
        } else {
            println!("No available position for chest.");
        }
    }

    pub fn compute_player_fov(&mut self, player: &mut Player, radius: usize) {
        let pos = { player.pos() };
        let visible = Navigator::compute_fov(&self.generated_map.tiles, pos, radius);
        player.line_of_sight = visible;
    }

    fn update_fov_caches(&mut self, player: &mut Player) {
        self.shown_fov = FovToShow::None;

        let mut spell_fov_needs_update = false;
        if let Some(selected_spell) = player.selected_spell {
            if selected_spell == u8::MAX {
                if let Some(weapon) = player.equipment.weapon.as_ref() {
                    if weapon.range.unwrap_or(1u32) != self.spell_or_attack_fov_cache.radius {
                        spell_fov_needs_update = true;
                    } else {
                        if self.spell_or_attack_fov_cache.radius != 1 {
                            spell_fov_needs_update = true;
                        }
                    }
                } else if self.spell_or_attack_fov_cache.radius != 1 {
                    spell_fov_needs_update = true;
                }
                self.shown_fov = FovToShow::Attack;
            } else if let Some(player_spell) = player.spells.get(selected_spell as usize) {
                if player_spell.spell_type.strategy == crate::spell_type::SpellStrategy::Aim {
                    if let Some(hovered) = self.hovered_tile {
                        let spell_type = &player_spell.spell_type;
                        let radius = self.spell_or_attack_fov_cache.radius;
                        if spell_type.area_radius != Some(radius) {
                            spell_fov_needs_update = true;
                        } else if hovered != self.spell_or_attack_fov_cache.origin {
                            spell_fov_needs_update = true;
                        }
                        self.shown_fov = FovToShow::Spell;
                    }
                } else {
                    self.shown_fov = FovToShow::Spell;
                    self.spell_or_attack_fov_cache.origin = player.pos();
                    self.spell_or_attack_fov_cache.radius =
                        player_spell.spell_type.area_radius.unwrap_or(0);
                    self.spell_or_attack_fov_cache.area = Navigator::compute_fov(
                        &self.generated_map.tiles,
                        self.spell_or_attack_fov_cache.origin,
                        self.spell_or_attack_fov_cache.radius as usize,
                    );
                    return;
                }
            }
        }

        if spell_fov_needs_update {
            if player.selected_spell.unwrap() == u8::MAX {
                self.spell_or_attack_fov_cache.radius = player
                    .equipment
                    .weapon
                    .as_ref()
                    .and_then(|w| w.range)
                    .unwrap_or(1);
                self.spell_or_attack_fov_cache.origin = player.pos();
                self.spell_or_attack_fov_cache.area = Navigator::compute_fov(
                    &self.generated_map.tiles,
                    self.spell_or_attack_fov_cache.origin,
                    self.spell_or_attack_fov_cache.radius as usize,
                );
            } else {
                self.spell_or_attack_fov_cache.radius = player.spells
                    [player.selected_spell.unwrap() as usize]
                    .spell_type
                    .area_radius
                    .unwrap_or(0);
                self.spell_or_attack_fov_cache.origin =
                    self.hovered_tile.unwrap_or(POSITION_INVALID);
                self.spell_or_attack_fov_cache.area = Navigator::compute_fov(
                    &self.generated_map.tiles,
                    self.spell_or_attack_fov_cache.origin,
                    self.spell_or_attack_fov_cache.radius as usize,
                );
            }
        } else {
        }
    }

    pub fn draw(
        &mut self,
        graphics_manager: &mut GraphicsManager,
        player: &mut Player,
        offset: PointF,
        animating_effects: &HashMap<Position, Arc<RwLock<Texture2D>>>,
        animate_for: f32,
    ) {
        self.update_fov_caches(player);

        for x in 0..GRID_WIDTH {
            for y in 0..GRID_HEIGHT {
                let tile = &self.generated_map.tiles[Position::new(x, y)];
                tile.draw(
                    Position::new(x, y),
                    offset,
                    !self.monsters.is_empty(),
                    animating_effects.get(&Position::new(x, y)),
                    animate_for,
                );

                if self.shown_fov != FovToShow::None && animate_for == 0.0 {
                    let player_pos = player.pos();
                    let tile_pos = Position { x, y };
                    if player.selected_spell.unwrap() == u8::MAX
                        && self.spell_or_attack_fov_cache.radius > 0
                        && (player_pos
                            .in_range(&tile_pos, self.spell_or_attack_fov_cache.radius as usize)
                            || player_pos.is_neighbor(&tile_pos))
                        && player.line_of_sight.contains(&tile_pos)
                    {
                        draw_rectangle(
                            offset.x + x as f32 * TILE_SIZE,
                            offset.y + y as f32 * TILE_SIZE,
                            TILE_SIZE - 1.0,
                            TILE_SIZE - 1.0,
                            Color {
                                r: 1.0,
                                g: 0.0,
                                b: 0.0,
                                a: 0.2,
                            },
                        );
                    } else if let Some(spell) =
                        player.spells.get(player.selected_spell.unwrap() as usize)
                    {
                        if spell.spell_type.strategy == SpellStrategy::Fixed {
                            if tile_pos == player_pos {
                                continue;
                            }
                        }
                        if spell.spell_type.range.is_some()
                            && player_pos
                                .in_range(&tile_pos, spell.spell_type.range.unwrap() as usize)
                            && player.line_of_sight.contains(&tile_pos)
                        {
                            draw_rectangle(
                                offset.x + x as f32 * TILE_SIZE,
                                offset.y + y as f32 * TILE_SIZE,
                                TILE_SIZE - 1.0,
                                TILE_SIZE - 1.0,
                                Color {
                                    r: 0.0,
                                    g: 1.0,
                                    b: 0.0,
                                    a: 0.2,
                                },
                            );
                        }

                        if self
                            .spell_or_attack_fov_cache
                            .area
                            .contains(&Position { x, y })
                        {
                            draw_rectangle(
                                offset.x + x as f32 * TILE_SIZE,
                                offset.y + y as f32 * TILE_SIZE,
                                TILE_SIZE - 1.0,
                                TILE_SIZE - 1.0,
                                Color {
                                    r: 0.0,
                                    g: 0.0,
                                    b: 1.0,
                                    a: 0.5,
                                },
                            );
                        }
                    }
                }
            }
        }

        let mut material = graphics_manager.get_color_replace_material();
        gl_use_material(&material);

        for (_, monster) in &self.monsters {
            monster.borrow().draw(&mut material, offset);
        }

        if self.generated_map.visited_state == VisitedState::Visited {
            player.draw(&mut material, offset);
        }

        gl_use_default_material();
    }

    pub fn is_tile_enemy_occupied(&self, pos: Position) -> bool {
        pos.x < GRID_WIDTH && pos.y < GRID_HEIGHT && self.generated_map.tiles[pos].has_enemy()
    }

    pub fn is_tile_walkable(&self, pos: Position) -> bool {
        pos.x < GRID_WIDTH && pos.y < GRID_HEIGHT && self.generated_map.tiles[pos].is_walkable()
    }

    pub fn is_tile_blocking(&self, pos: Position) -> bool {
        pos.x >= GRID_WIDTH || pos.y >= GRID_HEIGHT || self.generated_map.tiles[pos].is_blocking()
    }

    pub fn is_tile_blocking_by_object(&self, pos: Position) -> bool {
        pos.x >= GRID_WIDTH
            || pos.y >= GRID_HEIGHT
            || self.generated_map.tiles[pos].is_solid_blocking()
    }

    pub fn get_chest_items(&self, position: &Position) -> Option<&Vec<u32>> {
        if position.x < GRID_WIDTH && position.y < GRID_HEIGHT {
            let tile = &self.generated_map.tiles[*position];
            if let Some(item) = tile.get_top_item() {
                if let ItemKind::Container(container) = item {
                    return Some(&container.items);
                }
            }
        }
        None
    }

    pub fn remove_chest(&mut self, position: Position) {
        for (idx, item) in self.generated_map.tiles[position].items.iter().enumerate() {
            if let ItemKind::Container(_) = item {
                self.generated_map.tiles[position].items.remove(idx);
                return; // Exit after removing the first container
            }
        }
    }

    pub fn get_random_adjacent_position(
        &mut self,
        position: Position,
        must_be_walkable: bool,
    ) -> Option<Position> {
        let mut rng = thread_rng();
        let mut adjacent_positions: Vec<Position> = position
            .positions_around()
            .into_iter()
            .filter(|&pos| {
                if must_be_walkable {
                    self.is_tile_walkable(pos)
                } else {
                    !self.is_tile_blocking(pos)
                }
            })
            .collect();

        if !adjacent_positions.is_empty() {
            adjacent_positions.shuffle(&mut rng);
            return Some(adjacent_positions[0]);
        }
        None
    }
}

#[derive(Clone)]
pub struct MapRc(pub Rc<RefCell<Map>>);

impl UserData for MapRc {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("get_tier", |_, this, ()| {
            Ok(this.0.borrow().generated_map.tier)
        });

        methods.add_method("get_monster_kinds", |_, this, ()| {
            let test = this.0.borrow().generated_map.monster_kinds.clone();
            println!("Monster kinds: {:?}", test);
            Ok(test)
        });

        methods.add_method("get_walkable_tiles", |lua, this, ()| {
            // borrow your Rust-side LuaInterface
            // grab your Vec<Position>
            let walkable = &this.0.borrow().generated_map.available_walkable_cache;
            // create a new Lua table to hold the sequence
            let tbl = lua.create_table()?;
            // for each Position, call add_position and insert into tbl
            for (i, pos) in walkable.iter().enumerate() {
                let pos_tbl = LuaInterface::add_position(lua, pos)?;
                tbl.set(i + 1, pos_tbl)?;
            }
            Ok(tbl)
        });

        methods.add_method(
            "get_random_adjacent_position",
            |lua, this, (pos, must_be_walkable): (Table, bool)| {
                let position = Position {
                    x: pos.get("x")?,
                    y: pos.get("y")?,
                };
                let new_pos = this
                    .0
                    .borrow_mut()
                    .get_random_adjacent_position(position, must_be_walkable);
                match new_pos {
                    Some(pos) => {
                        let tbl = LuaInterface::add_position(&lua, &pos)?;
                        Ok(tbl)
                    }
                    None => Ok(LuaInterface::add_position(&lua, &POSITION_INVALID)?),
                }
            },
        );

        // methods.add_method("add_monster", |_, this, (kind_id, pos): (u32, Table)| {
        //     let p = Position {
        //         x: pos.get("x")?,
        //         y: pos.get("y")?,
        //     };

        //     let kind = monster_types
        //         .lock()
        //         .unwrap()
        //         .iter()
        //         .find(|mt| mt.id == kind_id)
        //         .expect("Monster type not found");

        //     let monster = Rc::new(RefCell::new(Monster::new(p, kind.clone())));
        //     this.generated_map.tiles[p].creature = id;
        //     let id = monster.borrow().id;
        //     this.monsters.insert(id, monster);
        //     Ok(())
        // });
    }
}
