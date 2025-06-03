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
use std::collections::HashMap;
use std::rc::Rc;
use once_cell::sync::OnceCell;
use serde::Deserialize;
use serde_json::from_str;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
pub enum SpellKind {
    Attack,
    Heal,
    Buff,
    Debuff,
    Summon,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
pub enum SpellAreaKind {
    Missile,
    Area,
    Bomb,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SpellType {
    pub index: u32, // Unique index for the spell type
    pub name: String,
    pub kind: SpellKind,
    pub area_kind: SpellAreaKind,
    pub description: String,
    pub max_charges: u32, // Number of charges available
    pub range: u32, // Range in tiles
    pub basepower: u32, // Base Power of the spell
    pub cost: u32, // Cost to buy
}

pub async fn load_spell_types() -> Vec<Option<Arc<SpellType>>> {
    let file = load_string("assets/spells.json").await.unwrap();
    let list: Vec<SpellType> = from_str(&file).unwrap();

    // Find the highest index to size the vector
    let max_index = list.iter().map(|st| st.index).max().unwrap_or(0);

    // Create a vector of None, sized to max_index + 1
    let mut spell_vec: Vec<Option<Arc<SpellType>>> = vec![None; (max_index + 1) as usize];

    // Insert spells at their index positions
    for spell in list {
        let index = spell.index as usize;
        spell_vec[index] = Some(Arc::new(spell));
    }

    spell_vec
}

pub static SPELL_TYPES: OnceCell<Vec<Option<Arc<SpellType>>>> = OnceCell::new();

pub fn set_global_spell_types(vec: Vec<Option<Arc<SpellType>>>) {
    SPELL_TYPES.set(vec).expect("GLOBAL_SPELL_TYPES already set!");
}

pub fn get_spell_types() -> &'static Vec<Option<Arc<SpellType>>> {
    SPELL_TYPES.get().expect("GLOBAL_SPELL_TYPES not initialized")
}
