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
use once_cell::sync::OnceCell;
use serde::Deserialize;
use serde_json::from_str;
use std::sync::{Arc, RwLock};

use crate::ui::point_f::PointF;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
pub enum SpellKind {
    Attack,
    Heal,
    Buff,
    Debuff,
    Summon,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
pub enum SpellStrategy {
    Aim,
    Fixed,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SpellType {
    pub index: u32, // Unique index for the spell type
    pub name: String,
    pub kind: SpellKind,
    pub strategy: SpellStrategy,
    pub area_radius: Option<u32>,
    pub description: String,
    pub mp_cost: u32,
    pub range: Option<u32>, // Range in tiles
    pub basepower: u32,     // Base Power of the spell
    pub cost: u32,          // Cost to buy
    #[serde(default)]
    pub sprite_path: String,
    #[serde(skip)]
    pub sprite: Option<Arc<RwLock<Texture2D>>>,
}

impl SpellType {
    fn draw(&self, offset: PointF) {
        if let Some(sprite_arc) = &self.sprite {}
    }
}

pub async fn load_spell_types() -> Vec<Option<Arc<SpellType>>> {
    let file = load_string("assets/spells/spells.json").await.unwrap();
    let list: Vec<SpellType> = from_str(&file).unwrap();

    // Find the highest index to size the vector
    let max_index = list.iter().map(|st| st.index).max().unwrap_or(0);

    // Create a vector of None, sized to max_index + 1
    let mut spell_vec: Vec<Option<Arc<SpellType>>> = vec![None; (max_index + 1) as usize];

    // Insert spells at their index positions
    for mut spell in list {
        if !spell.sprite_path.is_empty() {
            let sprite_path = format!("assets/sprites/effects/{}.png", spell.sprite_path);
            match macroquad::texture::load_texture(&sprite_path).await {
                Ok(texture) => {
                    texture.set_filter(FilterMode::Nearest);
                    spell.sprite = Some(Arc::new(RwLock::new(texture)));
                }
                Err(e) => {
                    eprintln!("Failed to load texture from {}: {}", sprite_path, e);
                }
            };
        }

        let index = spell.index as usize;
        spell_vec[index] = Some(Arc::new(spell));
    }

    spell_vec
}

pub static SPELL_TYPES: OnceCell<Vec<Option<Arc<SpellType>>>> = OnceCell::new();

pub fn set_global_spell_types(vec: Vec<Option<Arc<SpellType>>>) {
    SPELL_TYPES
        .set(vec)
        .expect("GLOBAL_SPELL_TYPES already set!");
}

pub fn get_spell_types() -> &'static Vec<Option<Arc<SpellType>>> {
    SPELL_TYPES
        .get()
        .expect("GLOBAL_SPELL_TYPES not initialized")
}
