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

use crate::creature::Creature;
use crate::items::base_item::Item;
use crate::items::holdable::*;
use crate::maps::TILE_SIZE;
use crate::player_spell::PlayerSpell;
use crate::position::Position;
use crate::spell_type;
use crate::ui::point_f::PointF;
use macroquad::prelude::*;
use mlua::{UserData, UserDataMethods};
use std::cell::RefCell;
use std::cmp::{max, min};
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct Equipment {
    pub weapon: Option<Weapon>,
    pub shield: Option<Shield>,
    pub helmet: Option<Helmet>,
    pub armor: Option<Armor>,
    pub boots: Option<Boots>,
}

#[derive(Clone)]
pub struct Player {
    pub hp: u32,
    pub max_hp: u32,
    pub mp: u32,
    pub max_mp: u32,

    pub strength: u32,
    pub dexterity: u32,
    pub intelligence: u32,

    pub sp: u32,

    pub accumulated_speed: u32,

    pub position: Position,
    pub goal_position: Option<Position>,
    pub spells: Vec<PlayerSpell>,
    pub selected_spell: Option<usize>,
    pub line_of_sight: HashSet<Position>,

    pub equipment: Equipment,
}

impl Player {
    pub fn new(pos: Position) -> Self {
        let first_spell = spell_type::get_spell_types()[1].clone();
        let mut spells: Vec<PlayerSpell> = Vec::new();

        if let Some(spell) = first_spell {
            let max_charges = spell.max_charges;
            spells = vec![PlayerSpell {
                spell_type: spell,
                charges: max_charges,
            }];
        }

        Self {
            hp: 100,
            max_hp: 100,
            mp: 50,
            max_mp: 50,
            strength: 10,
            dexterity: 10,
            intelligence: 10,
            sp: 1,
            accumulated_speed: 0,
            position: pos,
            goal_position: None,
            spells: spells,
            selected_spell: None,
            line_of_sight: HashSet::new(),
            equipment: Equipment {
                weapon: None,
                shield: None,
                helmet: None,
                armor: None,
                boots: None,
            },
        }
    }

    fn add_mana(&mut self, amount: i32) {
        self.mp = min(max((self.hp as i32) + amount, 0) as u32, self.max_hp);
    }

    fn get_mana(&self) -> (u32, u32) {
        (self.mp, self.max_mp)
    }

    fn get_soul_points(&self) -> u32 {
        self.sp
    }

    pub fn get_speed(&self) -> u32 {
        self.dexterity * 10
    }

    pub fn add_item(&mut self, item: Item) {
        match item {
            Item::Weapon(w) => self.equipment.weapon = Some(w),
            Item::Armor(a) => self.equipment.armor = Some(a),
            Item::Shield(s) => self.equipment.shield = Some(s),
            Item::Helmet(h) => self.equipment.helmet = Some(h),
            Item::Boots(b) => self.equipment.boots = Some(b),
            _ => {
                println!("Cannot equip this item.");
            }
        }
    }
}

impl Creature for Player {
    fn name(&self) -> &str {
        "Player"
    }

    fn pos(&self) -> Position {
        self.position
    }

    fn set_pos(&mut self, pos: Position) {
        self.position = pos;
    }

    fn add_health(&mut self, amount: i32) {
        self.hp = min(max((self.hp as i32) + amount, 0) as u32, self.max_hp);
    }

    fn get_health(&self) -> (u32, u32) {
        (self.hp, self.max_hp)
    }

    fn draw(&self, offset: PointF) {
        // Base colored rectangle
        draw_rectangle(
            offset.x + self.position.x as f32 * TILE_SIZE + 4.0,
            offset.y + self.position.y as f32 * TILE_SIZE + 4.0,
            TILE_SIZE - 8.0,
            TILE_SIZE - 8.0,
            BLUE,
        );

        // Glyph overlay
        draw_text(
            "@",
            offset.x + self.position.x as f32 * TILE_SIZE + 10.0,
            offset.y + self.position.y as f32 * TILE_SIZE + 20.0,
            18.0,
            WHITE,
        );
    }

    fn is_player(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl UserData for Player {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("get_mana", |_, this, ()| Ok(this.get_mana()));

        methods.add_method("get_soul_points", |_, this, ()| Ok(this.get_soul_points()));
    }
}
