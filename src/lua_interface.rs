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

use mlua::{
    AnyUserData, AnyUserDataExt, Error, FromLuaMulti, Function, IntoLuaMulti, Lua, MultiValue,
    RegistryKey, Result, Table, UserDataMetatable, Value,
};

use std::fs;
use std::rc::Rc;
use std::{cell::RefCell, collections::HashMap};

use crate::maps::map::{Map, MapRef};
use crate::monster::MonsterRef;
use crate::{items::holdable::Weapon, player::Player, position::Position};

pub trait LuaScripted {
    fn set_script_id(&mut self, id: u32);
    fn get_script_id(&self) -> u32;
    fn script_path(&self) -> Option<String>;
    fn is_scripted(&self) -> bool;
    fn functions(&self) -> Vec<String>;
}

/// Holds the registry key for each Lua function we care about.
struct ScriptedFunctions {
    on_map_peeked: Option<RegistryKey>,
    on_get_attack_damage: Option<RegistryKey>,
    on_spawn: Option<RegistryKey>,
    on_update: Option<RegistryKey>,
    on_death: Option<RegistryKey>,
}

/// Manages one Lua VM and a cache of loaded scripts → functions.
pub struct LuaInterface {
    pub lua: Lua,
    script_cache: HashMap<u32, ScriptedFunctions>,
    //pub add_monster_callback: Option<Rc<dyn Fn(u32, Position) -> MonsterRef>>,
    pub get_monster_by_id_callback: Option<Rc<dyn Fn(u32) -> Option<MonsterRef> + 'static>>,
    pub get_current_map_callback: Option<Rc<dyn Fn() -> MapRef>>,
    pub map_add_monster_callback: Option<Rc<dyn Fn(MapRef, u32, Position) -> MonsterRef>>,
    pub script_id_counter: u32,
}

pub type LuaInterfaceRc = Rc<RefCell<LuaInterface>>;

impl LuaInterface {
    /// Create a fresh Lua VM.
    pub fn new() -> Rc<RefCell<Self>> {
        let i = Rc::new(RefCell::new(Self {
            lua: Lua::new(),
            script_cache: HashMap::new(),
            //add_monster_callback: None,
            get_monster_by_id_callback: None,
            get_current_map_callback: None,
            map_add_monster_callback: None,
            script_id_counter: 1,
        }));

        i.borrow_mut()
            .load_global_script()
            .expect("Failed to load global Lua script");

        i
    }

    pub fn register_api(lua_if_rc: &LuaInterfaceRc) -> Result<()> {
        let lua_if = lua_if_rc.borrow();

        // lua_if.add_lua_fn("add_monster", move |lua, (id, pos): (u32, Table)| {
        //     /* … */
        //     Ok(())
        // })?;

        // lua_if.add_lua_fn("add_monster", {
        //     let cb_opt = lua_if.add_monster_callback.clone();
        //     move |_lua, (id, pos): (u32, Table)| {
        //         let p = Position {
        //             x: pos.get("x")?,
        //             y: pos.get("y")?,
        //         };
        //         if let Some(cb) = &cb_opt {
        //             Ok(cb(id, p)) // returns u32
        //         } else {
        //             Err(Error::external("…"))
        //         }
        //     }
        // })?;

        lua_if.add_lua_fn("get_monster_by_id", {
            let cb_opt = lua_if.get_monster_by_id_callback.clone();
            move |lua, id: u32| {
                if let Some(cb) = &cb_opt {
                    if let Some(monster_rc) = cb(id) {
                        lua.create_userdata(monster_rc.clone()).map(Value::UserData)
                    } else {
                        Err(Error::external(format!("No monster with ID {}", id)))
                    }
                } else {
                    Err(Error::external("No get_monster_by_id callback set!"))
                }
            }
        })?;

        lua_if.add_lua_fn("get_current_map", {
            let cb_opt = lua_if.get_current_map_callback.clone();
            move |lua, ()| {
                if let Some(cb) = &cb_opt {
                    let map_rc = cb();
                    lua.create_userdata(map_rc.clone())
                } else {
                    Err(Error::external("No get_monster_by_id callback set!"))
                }
            }
        })?;

        Ok(())
    }

    pub fn add_position<'lua>(lua: &'lua Lua, pos: &Position) -> mlua::Result<Table<'lua>> {
        let lua_pos = lua.create_table()?;
        lua_pos.set("x", pos.x)?;
        lua_pos.set("y", pos.y)?;
        Ok(lua_pos)
    }

    pub fn load_global_script(&mut self) -> Result<bool> {
        // 1) Read the script file
        let path = "assets/global.lua";
        let script = fs::read_to_string(path)
            .map_err(|e| Error::external(format!("Failed to read {}: {}", path, e)))?;

        // 2) Create a new isolated environment
        let env: Table = self.lua.create_table()?;
        let globals = self.lua.globals(); // Table<'_>
        let mt: Table = self.lua.create_table()?;

        if let Ok(old_gd) = globals.get::<_, Table>("GlobalData") {
            env.set("GlobalData", old_gd)?;
        }

        mt.set("__index", globals)?;
        env.set_metatable(Some(mt));

        // 3) Load and execute the script in that environment
        self.lua.load(&script).set_environment(env.clone()).exec()?;

        let gd: Table = env.get("GlobalData")?;
        self.lua.globals().set("GlobalData", gd)?;

        let f: Function = env.get("on_map_peeked")?;
        let key: RegistryKey = self.lua.create_registry_value(f)?;
        let holder = ScriptedFunctions {
            on_map_peeked: Some(key),
            on_get_attack_damage: None,
            on_spawn: None,
            on_update: None,
            on_death: None,
        };

        self.script_cache.insert(0, holder);

        Ok(true)
    }

    pub fn load_script<T: LuaScripted>(&mut self, entity: &mut T) -> Result<bool> {
        let path = match entity.script_path() {
            Some(p) => p,
            None => return Ok(false),
        };

        entity.set_script_id(self.script_id_counter);
        self.script_id_counter += 1;

        let script = fs::read_to_string(path.clone())
            .map_err(|e| Error::external(format!("Failed to read {}: {}", path, e)))?;

        // new, isolated env
        let env: Table = self.lua.create_table()?;
        let globals = self.lua.globals(); // Table<'_>
        let mt: Table = self.lua.create_table()?;

        let gd: Table = globals.get("GlobalData")?;
        env.set("GlobalData", gd)?;

        mt.set("__index", globals)?;
        env.set_metatable(Some(mt));

        // run under that env
        self.lua.load(&script).set_environment(env.clone()).exec()?;

        // 4) For each func name your trait advertises, extract & stash it
        let mut holder = ScriptedFunctions {
            on_map_peeked: None,
            on_get_attack_damage: None,
            on_spawn: None,
            on_update: None,
            on_death: None,
        };

        for name in entity.functions() {
            // skip missing entries
            if !env.contains_key(name.clone())? {
                continue;
            }

            let f: Function = env.get(name.clone())?;
            let key: RegistryKey = self.lua.create_registry_value(f)?;
            match name.as_str() {
                "on_get_attack_damage" => holder.on_get_attack_damage = Some(key),
                "on_spawn" => holder.on_spawn = Some(key),
                "on_update" => holder.on_update = Some(key),
                "on_death" => holder.on_death = Some(key),
                _ => {} // ignore anything else
            }
        }

        self.script_cache.insert(entity.get_script_id(), holder);

        Ok(true)
    }

    pub fn on_get_attack_damage(
        &self,
        weapon: &mut Weapon,
        player: &mut Player,
        monster: &mut MonsterRef,
    ) -> Result<f32> {
        let binding = &self.script_cache;
        let funcs = binding.get(&weapon.get_script_id()).ok_or_else(|| {
            Error::external(format!(
                "No Lua script loaded for weapon `{}`",
                weapon.get_script_id()
            ))
        })?;

        // Retrieve the Function from the registry
        let func: Function = self
            .lua
            .registry_value(&funcs.on_get_attack_damage.as_ref().unwrap())?;

        let lua_weapon = Rc::new(RefCell::new(weapon.clone()));
        let lua_player = Rc::new(RefCell::new(player.clone()));

        let lua_weapon_ud = self.lua.create_userdata(lua_weapon.clone())?;
        let lua_player_ud = self.lua.create_userdata(lua_player.clone())?;
        let lua_monster_ud = self.lua.create_userdata(monster.clone())?;

        // Invoke and return result
        let result = func.call((lua_weapon_ud, lua_player_ud, lua_monster_ud));

        *weapon = lua_weapon.borrow().clone();
        *player = lua_player.borrow().clone();

        result
    }

    pub fn on_spawn(&self, monster_ref: &mut MonsterRef) -> Result<bool> {
        let monster = monster_ref.borrow_mut();
        let binding = &self.script_cache;
        let funcs = binding.get(&monster.kind.get_script_id()).ok_or_else(|| {
            Error::external(format!(
                "No Lua script loaded for monster type `{}`",
                monster.kind.get_script_id()
            ))
        })?;

        drop(monster);
        // Retrieve the Function from the registry
        if let Some(func_key) = &funcs.on_spawn {
            let func: Function = self.lua.registry_value(func_key)?;

            let lua_monster_ud = self.lua.create_userdata(monster_ref.clone())?;

            // Invoke and return result
            let result = func.call(lua_monster_ud);

            result
        } else {
            Ok(false)
        }
    }

    pub fn on_update(&self, monster_ref: &mut MonsterRef) -> Result<bool> {
        let monster = monster_ref.borrow_mut();
        let binding = &self.script_cache;
        let funcs = binding.get(&monster.kind.get_script_id()).ok_or_else(|| {
            Error::external(format!(
                "No Lua script loaded for monster type `{}`",
                monster.kind.get_script_id()
            ))
        })?;

        // Retrieve the Function from the registry
        if let Some(func_key) = &funcs.on_update {
            let func: Function = self.lua.registry_value(func_key)?;

            let lua_monster_ud = self.lua.create_userdata(monster_ref.clone())?;

            drop(monster);
            // Invoke and return result
            let result = func.call(lua_monster_ud);

            result
        } else {
            Ok(false)
        }
    }

    pub fn on_death(&self, monster_ref: &mut MonsterRef) -> Result<bool> {
        let monster = monster_ref.borrow_mut();
        let binding = &self.script_cache;
        let funcs = binding.get(&monster.kind.id).ok_or_else(|| {
            Error::external(format!(
                "No Lua script loaded for monster type `{}`",
                monster.kind.get_script_id()
            ))
        })?;

        // Retrieve the Function from the registry
        if let Some(func_key) = &funcs.on_death {
            let func: Function = self.lua.registry_value(func_key)?;

            let lua_monster_ud = self.lua.create_userdata(monster_ref.clone())?;

            // Invoke and return result
            let result = func.call(lua_monster_ud);

            result
        } else {
            Ok(false)
        }
    }

    fn setup_lua_map_methods(&self, lua_map_ud: AnyUserData) -> mlua::Result<()> {
        let map_add_monster_callback = self.map_add_monster_callback.clone();
        let mt = lua_map_ud.get_metatable()?; // mt: UserDataMetatable

        let methods_tbl: Table = mt.get("__index")?;
        methods_tbl.set(
            "add_monster",
            self.lua.create_function(
                move |lua_ctx, (lua_self, kind_id, pos): (AnyUserData, u32, Table)| {
                    // pull the MapRef back out of the userdata:
                    let map_ref: MapRef = lua_self.borrow::<MapRef>()?.clone();

                    // build the Position
                    let p = Position {
                        x: pos.get("x")?,
                        y: pos.get("y")?,
                    };

                    // call your Rust callback
                    if let Some(cb) = &map_add_monster_callback {
                        let monster_rc = cb(map_ref, kind_id, p);
                        // return the new monster userdata back into Lua
                        let ud = lua_ctx.create_userdata(monster_rc)?;
                        Ok(ud)
                    } else {
                        Err(mlua::Error::external("No map_add_monster_callback set!"))
                    }
                },
            )?,
        )?;

        Ok(())
    }

    pub fn on_map_peeked(&self, map: &MapRef) -> Result<bool> {
        let binding = &self.script_cache;
        let funcs = binding
            .get(&0)
            .ok_or_else(|| Error::external(format!("No Lua script loaded for on_map_peeked")))?;

        // Retrieve the Function from the registry
        if let Some(func_key) = &funcs.on_map_peeked {
            let func: Function = self.lua.registry_value(func_key)?;

            let lua_map_ud = self.lua.create_userdata(map.clone())?;
            let setup_result = self.setup_lua_map_methods(lua_map_ud.clone());
            if setup_result.is_err() {
                return Err(Error::external(format!(
                    "Failed to setup Lua map methods: {}",
                    setup_result.unwrap_err(),
                )));
            }
            // Invoke and return result
            let result: Result<bool> = func.call(lua_map_ud);
            if let Err(e) = result {
                eprintln!("Error calling Lua on_map_peeked: {}", e);
                return Err(e);
            } else {
                return Ok(true);
            }
        } else {
            return Ok(false);
        }
    }
}

pub trait LuaBinder {
    fn add_lua_fn<'lua, A, R, F>(&'lua self, name: &'static str, f: F) -> Result<()>
    where
        A: FromLuaMulti<'lua>,
        R: IntoLuaMulti<'lua>,
        F: Fn(&'lua Lua, A) -> Result<R> + 'static;
}

impl LuaBinder for LuaInterface {
    fn add_lua_fn<'lua, A, R, F>(&'lua self, name: &'static str, f: F) -> Result<()>
    where
        A: FromLuaMulti<'lua>,
        R: IntoLuaMulti<'lua>,
        F: Fn(&'lua Lua, A) -> Result<R> + 'static,
    {
        //let lua = &guard.lua; // &Lua also lives for 'lua

        // 2) Pass your f directly to mlua::create_function
        //    mlua will do FromLuaMulti->A, call f, then IntoLuaMulti->MultiValue
        let func: Function = self.lua.create_function(move |lua, args: A| {
            let r: R = f(lua, args)?; // returns your R
            Ok(r) // mlua packs R into Lua return values
        })?;

        // 3) Register & drop the guard
        self.lua.globals().set(name, func)?;
        Ok(())
    }
}
