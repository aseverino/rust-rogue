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

use crate::lua_interface::LuaScripted;
use crate::maps::TILE_SIZE;
use crate::ui::point_f::PointF;
use macroquad::prelude::*;
use rlua::{UserData, UserDataMethods};
use crate::creature::Creature;
use crate::position::Position;
use crate::monster_type::MonsterType;
use std::cell::RefCell;
use std::cmp::{max, min};
use std::sync::atomic::AtomicU32;
use std::sync::{Arc, RwLock};

pub struct Monster {
    pub hp: u32,
    pub kind: Arc<MonsterType>,
    pub position: Position,
    pub id: u32,
}

pub type MonsterRef = Arc<RwLock<Monster>>;

static MONSTER_ID_COUNTER: AtomicU32 = AtomicU32::new(0);

impl Monster {
    pub fn new(pos: Position, kind: Arc<MonsterType>) -> Self {
        let id = MONSTER_ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Self {
            position: pos,
            hp: kind.max_hp,
            kind,
            id,
        }
    }
}

impl Creature for Monster {
    fn name(&self) -> &str {
        &self.kind.name
    }

    fn pos(&self) -> Position {
        self.position
    }

    fn set_pos(&mut self, pos: Position) {
        self.position = pos;
    }

    fn add_health(&mut self, amount: i32) {
        self.hp = min(max((self.hp as i32) + amount, 0) as u32, self.kind.max_hp);
    }

    fn get_health(&self) -> (u32, u32) {
        (self.hp, self.kind.max_hp)
    }

    fn draw(&self, offset: PointF) {
        if self.hp <= 0 {
            return; // Don't draw dead monsters
        }
        draw_rectangle(
            offset.x + self.position.x as f32 * TILE_SIZE + 8.0,
            offset.y + self.position.y as f32 * TILE_SIZE + 8.0,
            TILE_SIZE - 16.0,
            TILE_SIZE - 16.0,
            self.kind.color(),
        );

        // Optional glyph drawing
        let glyph = self.kind.glyph.to_string();
        draw_text(&glyph, offset.x + self.position.x as f32 * TILE_SIZE + 12.0, offset.y + self.position.y as f32 * TILE_SIZE + 20.0, 16.0, WHITE);
    }

    fn is_monster(&self) -> bool { true }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl UserData for Monster {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
    }
}