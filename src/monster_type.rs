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
use std::sync::Arc;
use serde_json::from_str;

use crate::lua_interface::{LuaInterface, LuaScripted};

pub async fn load_monster_types(lua_interface: &mut LuaInterface) -> Vec<Arc<MonsterType>> {
    let file: String = load_string("assets/monsters.json").await.unwrap();
    let list: Vec<MonsterType> = from_str(&file).unwrap();

    list
        .into_iter()
        .map(|mut mt| {
            if mt.script.is_some() {
                let script_result = lua_interface.load_script(&mt);
                if let Err(e) = script_result {
                    eprintln!("Error loading monster script: {}", e);
                } else {
                    mt.scripted = script_result.unwrap();
                }
            }
            Arc::new(mt)
        })
        .collect()
}

#[derive(Debug, Deserialize)]
pub struct MonsterType {
    pub id: u32,
    pub name: String,
    pub glyph: char,
    pub color: [u8; 3], // RGB, will convert to macroquad::Color
    pub max_hp: u32,
    pub melee_damage: i32,
    pub script: Option<String>,
    #[serde(default)]
    pub scripted: bool
}

impl MonsterType {
    pub fn color(&self) -> Color {
        Color::from_rgba(self.color[0], self.color[1], self.color[2], 255)
    }
}

impl LuaScripted for MonsterType {
    fn script_id(&self) -> u32 {
        self.id
    }

    fn script_path(&self) -> Option<String> {
        self.script.clone()
    }

    fn is_scripted(&self) -> bool {
        self.scripted
    }

    fn functions(&self) -> Vec<String> {
        vec!["on_update".to_string(), "on_death".to_string()]
    }
}