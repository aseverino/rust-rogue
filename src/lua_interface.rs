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

use std::rc::Rc;
use std::{cell::RefCell, collections::HashMap};
use std::fs;
use rlua::{Context, Error, Function, Lua, RegistryKey, Result, Table};

use crate::{items::holdable::Weapon, monster::Monster, player::Player, position::Position};

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

    pub fn init(&mut self) -> Result<()> {
        let add_fn: Function = self.lua.create_function(Self::add_monster)?;
        self.lua.globals().set("add_monster", add_fn)?;
        Ok(())
    }

    fn add_monster<'lua>(
        _ctx: Context<'lua>,
        (id, pos): (u32, Table<'lua>)
    ) -> Result<()> {
        Position::new(pos.get("x")?, pos.get("y")?);

        Ok(())
    }

    pub fn add_position<'lua>(&'lua self, pos: &Position) -> rlua::Result<Table<'lua>> {
        let lua_pos = self.lua.create_table()?;
        lua_pos.set("x", pos.x)?;
        lua_pos.set("y", pos.y)?;
        Ok(lua_pos)
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

    pub fn on_get_attack_damage(&self, weapon: &mut Weapon, player: &mut Player, monster: &mut Monster) -> Result<f32> {
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

        let lua_weapon = Rc::new(RefCell::new(weapon.clone()));
        let lua_player = Rc::new(RefCell::new(player.clone()));
        let lua_monster = Rc::new(RefCell::new(monster.clone()));

        let lua_weapon_ud = self.lua.create_userdata(lua_weapon.clone())?;
        let lua_player_ud = self.lua.create_userdata(lua_player.clone())?;
        let lua_monster_ud = self.lua.create_userdata(lua_monster.clone())?;

        // Invoke and return result
        let result = func.call((lua_weapon_ud, lua_player_ud, lua_monster_ud));

        *weapon = lua_weapon.borrow().clone();
        *player = lua_player.borrow().clone();
        *monster = lua_monster.borrow().clone();

        result
    }

    pub fn on_death(&self, monster: &mut Monster) -> Result<bool> {
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

        let lua_monster = Rc::new(RefCell::new(monster.clone()));
        let lua_monster_ud = self.lua.create_userdata(lua_monster.clone())?;

        // Invoke and return result
        let result = func.call(lua_monster_ud);

        *monster = lua_monster.borrow().clone();

        result
    }
}
