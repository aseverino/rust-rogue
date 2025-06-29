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
use crate::graphics;
use crate::graphics::graphics_manager::GraphicsManager;
use crate::maps::TILE_SIZE;
use crate::monster_kind::{MonsterKind, MonsterKindSprite};
use crate::position::Position;
use crate::ui::point_f::PointF;
use macroquad::prelude::*;
use mlua::{Table, UserData, UserDataMethods};
use std::cell::RefCell;
use std::cmp::{max, min};
use std::rc::Rc;
use std::sync::atomic::AtomicU32;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub struct Monster {
    pub hp: u32,
    pub kind: Arc<MonsterKind>,
    pub position: Position,
    pub id: u32,
    pub initialized: bool,
    pub accumulated_speed: u32,
}

pub type MonsterRef = Rc<RefCell<Monster>>;
pub type MonsterArc = Arc<RwLock<Monster>>;

static MONSTER_ID_COUNTER: AtomicU32 = AtomicU32::new(1);

impl Monster {
    pub fn new(pos: Position, kind: Arc<MonsterKind>) -> Self {
        let id = MONSTER_ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Self {
            position: pos,
            hp: kind.max_hp,
            kind,
            id,
            initialized: false,
            accumulated_speed: 0,
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

    fn draw(&self, material: &mut Material, offset: PointF) {
        if self.hp <= 0 {
            return; // Don't draw dead monsters
        }

        if let Some(sprite_arc) = &self.kind.sprite {
            let sprite = sprite_arc.read().unwrap();
            let sprite_size = Vec2::new(32.0, 32.0);
            graphics::graphics_manager::set_color_replacement_uniforms(
                material,
                self.kind.material_colors[0],
                self.kind.material_colors[1],
            );

            let draw_params = DrawTextureParams {
                dest_size: Some(sprite_size),
                source: Some(Rect {
                    x: 0.0,
                    y: 0.0,
                    w: 16.0,
                    h: 16.0,
                }),
                ..Default::default()
            };

            let x =
                offset.x + self.position.x as f32 * TILE_SIZE + (TILE_SIZE - sprite_size.x) / 2.0;
            let y =
                offset.y + self.position.y as f32 * TILE_SIZE + (TILE_SIZE - sprite_size.y) / 2.0;

            draw_texture_ex(&sprite, x, y, WHITE, draw_params);
        } else {
            draw_rectangle(
                offset.x + self.position.x as f32 * TILE_SIZE + 8.0,
                offset.y + self.position.y as f32 * TILE_SIZE + 8.0,
                TILE_SIZE - 16.0,
                TILE_SIZE - 16.0,
                self.kind.color(),
            );

            // Optional glyph drawing
            let glyph = self.kind.glyph.to_string();
            draw_text(
                &glyph,
                offset.x + self.position.x as f32 * TILE_SIZE + 12.0,
                offset.y + self.position.y as f32 * TILE_SIZE + 20.0,
                16.0,
                WHITE,
            );
        }
    }

    fn is_monster(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl UserData for Monster {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("get_kind", |_, this, ()| Ok(this.kind.clone()));

        methods.add_method(
            "get_position",
            |lua_ctx, this, ()| -> mlua::Result<Table<'lua>> {
                // `lua_ctx` is your Context<'lua>
                let tbl = lua_ctx.create_table()?;
                tbl.set("x", this.position.x)?;
                tbl.set("y", this.position.y)?;
                Ok(tbl)
            },
        );

        methods.add_method("get_id", |_, this, ()| Ok(this.id));
    }
}
