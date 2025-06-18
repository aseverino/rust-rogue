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

// Cargo.toml
// [dependencies]
// rlua = "0.20.1"

use std::collections::HashMap;
use std::fs;
use rlua::{Lua, Table, Function, RegistryKey, Error, Result};

use crate::{items::holdable::Weapon, monster::{Monster, MonsterRef}, player::{Player, PlayerRef, WeaponRef}};

pub trait LuaScripted {
    fn script_id(&self) -> u32;
    fn script_path(&self) -> Option<String>;
    fn is_scripted(&self) -> bool;
    fn functions(&self) -> Vec<String>;
}

/// Holds the registry key for each Lua function we care about.
struct ScriptedFunctions {
    on_get_attack_damage: Option<RegistryKey>,
    on_update: Option<RegistryKey>,
    on_death: Option<RegistryKey>,
}

/// Manages one Lua VM and a cache of loaded scripts → functions.
pub struct LuaInterface {
    lua: Lua,
    script_cache: HashMap<u32, ScriptedFunctions>,
}

impl LuaInterface {
    /// Create a fresh Lua VM.
    pub fn new() -> Self {
        LuaInterface {
            lua: Lua::new(),
            script_cache: HashMap::new(),
        }
    }

    pub fn load_script<T: LuaScripted>(&mut self, entity: &T) -> Result<bool> {
        let path = match entity.script_path() {
            Some(p) => p,
            None => return Ok(false),
        };

        let script = fs::read_to_string(path.clone())
            .map_err(|e| Error::external(format!("Failed to read {}: {}", path, e)))?;

        // new, isolated env
        let env: Table = self.lua.create_table()?;
        let globals = self.lua.globals();             // Table<'_>
        let mt: Table = self.lua.create_table()?;
        mt.set("__index", globals)?;
        env.set_metatable(Some(mt));

        // run under that env
        self.lua.load(&script).set_environment(env.clone()).exec()?;

        // 4) For each func name your trait advertises, extract & stash it
        for name in entity.functions() {
            // skip missing entries
            if !env.contains_key(name.clone())? {
                continue;
            }

            let mut holder = ScriptedFunctions {
                on_get_attack_damage: None,
                on_update:            None,
                on_death:             None,
            };

            let f: Function       = env.get(name.clone())?;
            let key: RegistryKey  = self.lua.create_registry_value(f)?;
            match name.as_str() {
                "on_get_attack_damage" => holder.on_get_attack_damage = Some(key),
                "on_update"            => holder.on_update            = Some(key),
                "on_death"             => holder.on_death             = Some(key),
                _                      => {}  // ignore anything else
            }

            self.script_cache.insert(
                entity.script_id(),
                holder,
            );
        }

        Ok(true)
    }

    pub fn on_get_attack_damage(&self, weapon_ref: &WeaponRef, player_ref: &PlayerRef, monster_ref: &MonsterRef) -> Result<f32> {
        let weapon = weapon_ref.borrow();
        let funcs = self
            .script_cache
            .get(&weapon.base_holdable.base_item.id)
            .ok_or_else(|| {
                Error::external(format!(
                    "No Lua script loaded for weapon `{}`",
                    weapon.base_holdable.base_item.id
                ))
            })?;

        // Retrieve the Function from the registry
        let func: Function = self.lua.registry_value(&funcs.on_get_attack_damage.as_ref().unwrap())?;

        let lua_weapon = self.lua.create_userdata(weapon_ref.clone())?;
        let lua_player = self.lua.create_userdata(player_ref.clone())?;
        let lua_target = self.lua.create_userdata(monster_ref.clone())?;

        // Invoke and return result
        func.call((lua_weapon, lua_player, lua_target))
    }

    pub fn on_death(&self, monster_ref: &MonsterRef) -> Result<bool> {
        let monster = monster_ref.read().unwrap();
        let funcs = self
            .script_cache
            .get(&monster.kind.id)
            .ok_or_else(|| {
                Error::external(format!(
                    "No Lua script loaded for monster type `{}`",
                    monster.kind.id
                ))
            })?;

        // Retrieve the Function from the registry
        let func: Function = self.lua.registry_value(funcs.on_death.as_ref().unwrap())?;

        let lua_monster = self.lua.create_userdata(monster_ref.clone())?;

        // Invoke and return result
        func.call(lua_monster)
    }
}
