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

use crate::{items::holdable::Weapon, monster::Monster, player::Player};

/// Holds the registry key for each Lua function we care about.
struct HoldableFunctions {
    on_get_attack_damage: RegistryKey,
}

/// Manages one Lua VM and a cache of loaded scripts → functions.
pub struct LuaInterface {
    lua: Lua,
    script_cache: HashMap<u32, HoldableFunctions>,
}

impl LuaInterface {
    /// Create a fresh Lua VM.
    pub fn new() -> Self {
        LuaInterface {
            lua: Lua::new(),
            script_cache: HashMap::new(),
        }
    }

    fn setup_player<'lua>(&'lua self, player: &Player) -> rlua::Result<Table<'lua>> {
        // Prepare player table to pass to Lua
        let lua_player = self.lua.create_table()?;
        lua_player.set("strength", player.strength)?;
        lua_player.set("dexterity", player.dexterity)?;
        lua_player.set("intelligence", player.intelligence)?;
        Ok(lua_player)
    }

    pub fn setup_weapon<'lua>(&'lua self, weapon: &Weapon) -> rlua::Result<Table<'lua>> {
        let lua_weapon = self.lua.create_table()?;
        lua_weapon.set("attack_dice", weapon.attack_dice.clone())?;
        lua_weapon.set("modifier", weapon.base_holdable.modifier)?;
        lua_weapon.set("attribute_modifier", weapon.base_holdable.attribute_modifier.clone())?;
        lua_weapon.set("slot", weapon.base_holdable.slot.clone())?;
        lua_weapon.set("two_handed", weapon.two_handed)?;

        Ok(lua_weapon)
    }

    fn setup_target<'lua>(&'lua self, target: &Monster) -> rlua::Result<Table<'lua>> {
        // Prepare target table to pass to Lua
        let lua_target = self.lua.create_table()?;
        lua_target.set("health", target.hp)?;
        Ok(lua_target)
    }

    /// Load the script once (if present & non-empty), extract `on_get_attack_damage`,
    /// and stash it in the registry.  If `script` is `None` or `""`, this is a no-op.
    pub fn load_script_for_weapon(&mut self, weapon: &Weapon) -> Result<bool> {
        // Check for an actual script path
        let script_path = match &weapon.base_holdable.script {
            Some(path) if !path.is_empty() => path,
            _ => return Ok(false),
        };

        // Read the Lua file
        let script = fs::read_to_string(script_path)
            .map_err(|e| Error::external(format!("Failed to read {}: {}", script_path, e)))?;

        // Create an isolated environment table
        let env: Table = self.lua.create_table()?;
        let globals = self.lua.globals();
        let mt: Table = self.lua.create_table()?;
        mt.set("__index", globals)?;
        env.set_metatable(Some(mt));

        // Load & execute the chunk in that env
        let chunk = self.lua.load(&script).set_environment(env.clone());
        chunk.exec()?;

        // Grab the function and store it in the registry
        let func: Function = env.get("on_get_attack_damage")?;
        let key: RegistryKey = self.lua.create_registry_value(func)?;

        // Cache by weapon ID
        self.script_cache.insert(
            weapon.base_holdable.base_item.id,
            HoldableFunctions { on_get_attack_damage: key },
        );

        Ok(true)
    }

    /// Call the cached `on_get_attack_damage(item)` for this weapon.
    /// You should only call this if you know a script was loaded.
    pub fn on_get_attack_damage(&self, weapon: &Weapon, player: &Player, monster: &Monster) -> Result<f32> {
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
        let func: Function = self.lua.registry_value(&funcs.on_get_attack_damage)?;

        let lua_weapon = self.setup_weapon(weapon)?;
        let lua_player = self.setup_player(player)?;
        let lua_target = self.setup_target(monster)?; // Dummy target for now

        // Invoke and return result
        func.call((lua_weapon, lua_player, lua_target))
    }
}
