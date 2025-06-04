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
use serde::Deserialize;
use std::collections::HashMap;
use serde_json::from_str;
use std::rc::Rc;

pub async fn load_monster_types() -> HashMap<String, Rc<MonsterType>> {
    let file = load_string("assets/monsters.json").await.unwrap();
    let list: Vec<MonsterType> = from_str(&file).unwrap();

    list.into_iter()
        .map(|mt| (mt.name.clone(), Rc::new(mt)))
        .collect()
}

#[derive(Debug, Deserialize)]
pub struct MonsterType {
    pub name: String,
    pub glyph: char,
    pub color: [u8; 3], // RGB, will convert to macroquad::Color
    pub max_hp: i32,
    pub melee_damage: i32,
}

impl MonsterType {
    pub fn color(&self) -> Color {
        Color::from_rgba(self.color[0], self.color[1], self.color[2], 255)
    }
}