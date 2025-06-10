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
use crate::map::{TILE_SIZE, GRID_WIDTH, GRID_HEIGHT, Map};
use crate::creature::Creature;
use crate::position::{ Position, POSITION_INVALID };
use crate::player_spell::PlayerSpell;
use crate::spell_type;
use std::cmp::{max, min};
use std::collections::HashSet;

pub struct Player {
    pub hp: u32,
    pub max_hp: u32,
    pub mp: u32,
    pub max_mp: u32,
    pub position: Position,
    pub goal_position: Option<Position>,
    pub spells: Vec<PlayerSpell>,
    pub selected_spell: Option<usize>,
    pub line_of_sight: HashSet<Position>,
}

impl Player {
    pub fn new(pos: Position) -> Self {
        let first_spell = spell_type::get_spell_types()[1].clone();
        let mut spells: Vec<PlayerSpell> = Vec::new();

        if let Some(spell) = first_spell {
            let max_charges = spell.max_charges;
            spells = vec![
                PlayerSpell {
                    spell_type: spell,
                    charges: max_charges,
                }
            ];
        }

        Self {
            hp: 50,
            max_hp: 50,
            mp: 10,
            max_mp: 10,
            position: pos,
            goal_position: None,
            spells: spells,
            selected_spell: None,
            line_of_sight: HashSet::new(),
        }
    }

    fn add_mana(&mut self, amount: i32) {
        self.mp = min(max((self.hp as i32) + amount, 0) as u32, self.max_hp);
    }

    fn get_mana(&self) -> (u32, u32) {
        (self.mp, self.max_mp)
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

    fn draw(&self, offset: (f32, f32)) {
        // Base colored rectangle
        draw_rectangle(
            offset.0 + self.position.x as f32 * TILE_SIZE + 4.0,
            offset.1 + self.position.y as f32 * TILE_SIZE + 4.0,
            TILE_SIZE - 8.0,
            TILE_SIZE - 8.0,
            BLUE,
        );

        // Glyph overlay
        draw_text(
            "@",
            offset.0 + self.position.x as f32 * TILE_SIZE + 10.0,
            offset.1 + self.position.y as f32 * TILE_SIZE + 20.0,
            18.0,
            WHITE,
        );
    }

    fn is_player(&self) -> bool {
        true
    }
}