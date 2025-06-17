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

use std::fs;

use rlua::{Function, Table};

use crate::{items::{base_item::Item, holdable::Weapon}, monster::Monster, player::Player};

pub struct LuaInterface {
    lua: rlua::Lua,
}

impl LuaInterface {
    pub fn new() -> Self {
        let lua = rlua::Lua::new();
        Self { lua }
    }

    pub fn execute_script(&self, script: &str) -> rlua::Result<()> {
        self.lua.load(script).exec()
    }

    fn setup_player(&self, player: &Player) -> rlua::Result<Table<'_>> {
        // Prepare player table to pass to Lua
        let lua_player = self.lua.create_table()?;
        lua_player.set("strength", player.strength)?;
        lua_player.set("dexterity", player.dexterity)?;
        lua_player.set("intelligence", player.intelligence)?;
        Ok(lua_player)
    }

    fn setup_weapon(&self, weapon: &Weapon) -> rlua::Result<Table<'_>> {
        // Load the Lua script
        let script_contents = fs::read_to_string(&weapon.base_holdable.script)?;
        self.lua.load(&script_contents).exec()?;

        // Prepare item table to pass to Lua
        let lua_item = self.lua.create_table()?;
        lua_item.set("attack_dice", weapon.attack_dice.clone())?;
        lua_item.set("modifier", weapon.base_holdable.modifier)?;
        lua_item.set("attribute_modifier", weapon.base_holdable.attribute_modifier.clone())?;
        lua_item.set("slot", weapon.base_holdable.slot.clone())?;
        lua_item.set("two_handed", weapon.two_handed)?;
        Ok(lua_item)
    }

    fn setup_target(&self, target: &Monster) -> rlua::Result<Table<'_>> {
        // Prepare target table to pass to Lua
        let lua_target = self.lua.create_table()?;
        lua_target.set("health", target.hp)?;
        Ok(lua_target)
    }

    pub fn on_get_attack_damage(&self, player: &Player, weapon: &Weapon, target: &Monster) -> rlua::Result<f32> {
        let lua_player = self.setup_player(&player)?;
        let lua_weapon = self.setup_weapon(&weapon)?;
        let lua_target = self.setup_target(&target)?;

        // Call on_get_attack_damage
        let on_get_attack_damage: Function = self.lua.globals().get("on_get_attack_damage")?;
        let damage: i32 = on_get_attack_damage.call((lua_player, lua_weapon, lua_target))?;
        println!(" -> Damage: {}", damage);

        // Call on_check_accuracy
        // let on_check_accuracy: Function = self.lua.globals().get("on_check_accuracy")?;
        // let accuracy: f32 = on_check_accuracy.call(lua_item)?;
        // println!(" -> Accuracy: {:.0}%", accuracy * 100.0);

        Ok(damage as f32)
    }

    pub fn on_check_accuracy(&self, player: &Player, weapon: &Weapon, target: &Monster) -> rlua::Result<f32> {
        let lua_player = self.setup_player(&player)?;
        let lua_weapon = self.setup_weapon(&weapon)?;
        let lua_target = self.setup_target(&target)?;

        // Call on_check_accuracy
        let on_check_accuracy: Function = self.lua.globals().get("on_check_accuracy")?;
        let accuracy: f32 = on_check_accuracy.call((lua_player, lua_weapon, lua_target))?;
        println!(" -> Accuracy: {:.0}%", accuracy * 100.0);

        Ok(accuracy as f32)
    }
}