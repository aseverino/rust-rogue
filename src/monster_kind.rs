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
use mlua::{UserData, UserDataMethods};
use serde::Deserialize;
use serde_json::from_str;
use std::sync::{Arc, Mutex};

use crate::lua_interface::{LuaInterfaceRc, LuaScripted};

pub async fn load_monster_kinds(lua_interface_rc: &LuaInterfaceRc) -> Vec<Arc<MonsterKind>> {
    let mut lua_interface = lua_interface_rc.borrow_mut();
    let file: String = load_string("assets/monsters/monsters.json").await.unwrap();
    let list: Vec<MonsterKind> = from_str(&file).unwrap();

    list.into_iter()
        .map(|mut mt| {
            if mt.script.is_some() {
                let script_result = lua_interface.load_script(&mut mt);
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

#[derive(Clone, Debug, Deserialize)]
pub struct MonsterKind {
    pub id: u32,
    pub tier: u32,
    pub name: String,
    pub glyph: char,
    pub color: [u8; 3], // RGB, will convert to macroquad::Color
    pub max_hp: u32,
    pub speed: u32,
    pub melee_damage: i32,
    #[serde(default)]
    pub flying: bool,
    pub script: Option<String>,
    #[serde(default)]
    pub scripted: bool,
    #[serde(skip)]
    pub script_id: u32,
}

impl MonsterKind {
    pub fn color(&self) -> Color {
        Color::from_rgba(self.color[0], self.color[1], self.color[2], 255)
    }
}

impl LuaScripted for MonsterKind {
    fn set_script_id(&mut self, id: u32) {
        self.script_id = id;
    }
    fn get_script_id(&self) -> u32 {
        self.script_id
    }

    fn script_path(&self) -> Option<String> {
        if self.script.clone().is_some() {
            Some(format!("assets/monsters/{}", self.script.clone().unwrap()))
        } else {
            None
        }
    }

    fn is_scripted(&self) -> bool {
        self.scripted
    }

    fn functions(&self) -> Vec<String> {
        vec![
            "on_spawn".to_string(),
            "on_update".to_string(),
            "on_death".to_string(),
        ]
    }
}

impl UserData for MonsterKind {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("get_id", |_, this, ()| Ok(this.id));
        methods.add_method("can_fly", |_, this, ()| Ok(this.flying));
    }
}

pub type MonsterCollection = Vec<Arc<MonsterKind>>;
pub type MonsterKinds = Arc<Mutex<MonsterCollection>>;
