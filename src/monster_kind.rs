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
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::lua_interface::{LuaInterfaceRc, LuaScripted};

pub async fn load_monster_kinds(lua_interface_rc: &LuaInterfaceRc) -> MonsterKindsDataArc {
    let file: String = load_string("assets/monsters/monsters.json").await.unwrap();
    let list: Vec<MonsterKind> = from_str(&file).unwrap();
    let monster_kinds = MonsterKinds::new();

    use futures::future::join_all;
    let monsters_futures = list.into_iter().map(|mut mt| {
        let lua_interface_rc = lua_interface_rc.clone();
        let monster_kinds = monster_kinds.clone();

        async move {
            if mt.script.is_some() {
                let result = {
                    let mut lua_interface = lua_interface_rc.borrow_mut();
                    lua_interface.load_script(&mut mt)
                };

                match result {
                    Ok(val) => mt.scripted = val,
                    Err(e) => eprintln!("Error loading monster script: {}", e),
                }
            }
            if !mt.sprite_path().is_empty() {
                let mut mk = monster_kinds.write().unwrap();
                if !mk.sprite_cache.contains_key(&mt.sprite_image) {
                    match macroquad::texture::load_texture(&mt.sprite_path()).await {
                        Ok(texture) => {
                            texture.set_filter(FilterMode::Nearest);
                            mk.sprite_cache
                                .insert(mt.sprite_image.clone(), Arc::new(RwLock::new(texture)));
                        }
                        Err(e) => {
                            eprintln!("Failed to load texture from {}: {}", mt.sprite_path(), e);
                        }
                    }
                }

                if let Some(sprite) = mk.sprite_cache.get(&mt.sprite_image) {
                    mt.sprite = Some(sprite.clone());
                }
            }
            Arc::new(mt)
        }
    });
    monster_kinds.write().unwrap().vec = Arc::new(RwLock::new(join_all(monsters_futures).await));
    monster_kinds
}

pub type MonsterKindsVecArc = Arc<RwLock<Vec<Arc<MonsterKind>>>>;
pub type MonsterKindSprite = Arc<RwLock<Texture2D>>;
#[derive(Debug)]
pub struct MonsterKinds {
    pub vec: MonsterKindsVecArc,
    pub sprite_cache: HashMap<String, MonsterKindSprite>,
}

pub type MonsterKindsDataArc = Arc<RwLock<MonsterKinds>>;

impl MonsterKinds {
    pub fn new() -> MonsterKindsDataArc {
        Arc::new(RwLock::new(Self {
            vec: Arc::new(RwLock::new(Vec::new())),
            sprite_cache: HashMap::new(),
        }))
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(from = "MonsterKindHelper")]
pub struct MonsterKind {
    pub id: u32,
    pub tier: u32,
    pub name: String,
    pub glyph: char,
    pub colors: Vec<[u8; 3]>,
    #[serde(skip)]
    pub material_colors: [Color; 4],
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
    #[serde(default)]
    pub sprite_image: String,
    #[serde(skip)]
    pub sprite: Option<MonsterKindSprite>,
}

#[derive(Deserialize)]
struct MonsterKindHelper {
    pub id: u32,
    pub tier: u32,
    pub name: String,
    pub glyph: char,
    pub colors: Vec<[u8; 3]>,
    pub max_hp: u32,
    pub speed: u32,
    pub melee_damage: i32,
    #[serde(default)]
    pub flying: bool,
    pub script: Option<String>,
    #[serde(default)]
    pub scripted: bool,
    #[serde(default)]
    pub sprite_image: String,
}

fn to_color(rgb: &[u8; 3]) -> Color {
    Color::new(
        rgb[0] as f32 / 255.0,
        rgb[1] as f32 / 255.0,
        rgb[2] as f32 / 255.0,
        1.0,
    )
}

impl From<MonsterKindHelper> for MonsterKind {
    fn from(helper: MonsterKindHelper) -> Self {
        let color1 = &helper.colors.get(0).copied().unwrap_or([255, 0, 0]);
        let color2 = &helper.colors.get(1).copied().unwrap_or(*color1);
        let color3 = &helper.colors.get(2).copied().unwrap_or(*color2);
        let material_colors = [
            to_color(color1),
            to_color(color2),
            to_color(color3),
            to_color(&helper.colors.get(3).copied().unwrap_or(*color3)),
        ];

        Self {
            id: helper.id,
            tier: helper.tier,
            name: helper.name,
            glyph: helper.glyph,
            colors: helper.colors,
            material_colors,
            max_hp: helper.max_hp,
            speed: helper.speed,
            melee_damage: helper.melee_damage,
            flying: helper.flying,
            script: helper.script,
            scripted: helper.scripted,
            script_id: 0,
            sprite_image: helper.sprite_image,
            sprite: None,
        }
    }
}

impl MonsterKind {
    pub fn color(&self) -> Color {
        Color::from_rgba(self.colors[0][0], self.colors[0][1], self.colors[0][2], 255)
    }

    pub fn sprite_path(&self) -> String {
        format!("assets/sprites/monsters/{}.png", self.sprite_image)
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

// pub type MonsterCollection = Vec<Arc<MonsterKind>>;
// pub type MonsterKinds = Arc<Mutex<MonsterCollection>>;
