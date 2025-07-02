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
use crate::graphics::graphics_manager::GraphicsManager;
use crate::items::base_item::Item;
use crate::items::holdable::*;
use crate::maps::TILE_SIZE;
use crate::player_spell::PlayerSpell;
use crate::position::Position;
use crate::ui::point_f::PointF;
use crate::{graphics, spell_type};
use macroquad::prelude::*;
use mlua::{Table, UserData, UserDataMethods};
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
    pub selected_spell: Option<u8>,
    pub line_of_sight: HashSet<Position>,

    pub equipment: Equipment,

    pub sprite: Option<Arc<RwLock<Texture2D>>>,
    pub material_colors: [Color; 2],
}

pub type PlayerRc = Rc<RefCell<Player>>;

impl Player {
    pub async fn new(pos: Position) -> Self {
        // let first_spell = spell_type::get_spell_types()[2].clone();
        // let mut spells: Vec<PlayerSpell> = Vec::new();

        // if let Some(spell) = first_spell {
        //     spells = vec![PlayerSpell { spell_type: spell }];
        // }

        let mut p = Self {
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
            spells: vec![],
            selected_spell: None,
            line_of_sight: HashSet::new(),
            equipment: Equipment {
                weapon: None,
                shield: None,
                helmet: None,
                armor: None,
                boots: None,
            },

            sprite: None,
            material_colors: [
                Color::from_rgba(0, 0, 255, 255),
                Color::from_rgba(255, 255, 255, 255),
            ],
        };

        let path = "assets/sprites/player/player.png";
        match macroquad::texture::load_texture(path).await {
            Ok(texture) => {
                texture.set_filter(FilterMode::Nearest);
                p.sprite = Some(Arc::new(RwLock::new(texture)));
            }
            Err(e) => {
                eprintln!("Failed to load texture from {}: {}", path, e);
            }
        };

        p
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

    fn draw(&self, material: &mut Material, offset: PointF) {
        if let Some(sprite_arc) = &self.sprite {
            let time = get_time();
            let frame = ((time * 3.0) as usize) % 2; // 3 fps

            let sprite = sprite_arc.read().unwrap();
            let sprite_size = Vec2::new(32.0, 32.0);
            graphics::graphics_manager::set_color_replacement_uniforms(
                material,
                self.material_colors[0],
                self.material_colors[1],
                self.material_colors[1],
                self.material_colors[1],
            );

            let draw_params = DrawTextureParams {
                dest_size: Some(sprite_size),
                source: Some(Rect {
                    x: 0.0,
                    y: frame as f32 * 16.0,
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
        methods.add_method("get_mana", |_, this, ()| Ok(this.get_mana()));
        methods.add_method("get_soul_points", |_, this, ()| Ok(this.get_soul_points()));
    }
}
