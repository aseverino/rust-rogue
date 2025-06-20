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

    pub add_monster_callback: Option<Rc<dyn Fn(u32, Position)>>,
}

pub type LuaInterfaceRc = Rc<RefCell<LuaInterface>>;

impl LuaInterface {
    /// Create a fresh Lua VM.
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            lua: Lua::new(),
            script_cache: HashMap::new(),
            add_monster_callback: None,
        }))
    }

    pub fn register_api(lua_if: &LuaInterfaceRc) -> Result<()> {
        // 1) Clone the Rc so cb_opt can be cheaply cloned into the closure
        let cb_opt = lua_if.borrow().add_monster_callback.clone();

        lua_if.add_lua_fn("add_monster", move |id, pos: Table<'_>| {
            let x: usize = pos.get("x")?;
            let y: usize = pos.get("y")?;
            let p = Position { x, y };

            if let Some(cb) = &cb_opt {
                cb(id, p);
            } else {
                eprintln!("No add_monster callback set!");
            }
            Ok(())
        })?;

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
        if let Some(func_key) = &funcs.on_death {
            let func: Function = self.lua.registry_value(func_key)?;

            let lua_monster = Rc::new(RefCell::new(monster.clone()));
            let lua_monster_ud = self.lua.create_userdata(lua_monster.clone())?;

            // Invoke and return result
            let result = func.call(lua_monster_ud);

            *monster = lua_monster.borrow().clone();

            result
        } else {
            Ok(false)
        }
    }
}

pub trait LuaBinder {
    /// name: the global Lua function name (“add_monster”)
    /// f: a pure Rust closure that takes (id, pos) and returns a Lua Result
    fn add_lua_fn<F>(&self, name: &'static str, f: F) -> Result<()>
    where
        F: Fn(u32, Table<'_>) -> Result<()> + 'static;
}

impl LuaBinder for LuaInterfaceRc {
    fn add_lua_fn<F>(&self, name: &'static str, f: F) -> Result<()>
    where
        F: Fn(u32, Table<'_>) -> Result<()> + 'static
    {
        // grab the Lua handle
        let lua = &self.borrow().lua;
        let globals = lua.globals();

        // build the Lua‐callable function
        // we clone the Rc so that the closure can refer back to our interface
        let this_rc = self.clone();
        let func: Function = lua.create_function(move |_ctx: Context<'_>, (id, pos): (u32, Table<'_>)| {
            // now *inside* here you can do:
            //    let this = this_rc.borrow();
            // or directly call your callback.
            f(id, pos)
        })?;

        // register in Lua globals
        globals.set(name, func)?;
        Ok(())
    }
}