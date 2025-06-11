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

    pub fn call_function<T: rlua::IntoLua>(&self, func_name: &str, args: T) -> rlua::Result<rlua::Value> {
        let func: rlua::Function = self.lua.globals().get(func_name)?;
        func.call(args)
    }

    pub fn on_get_attack_damage(&self, item: Item) -> f32 {
        // Load the Lua script
        let script_contents = fs::read_to_string(&item.script)?;
        lua.load(&script_contents).exec()?;

        // Prepare item table to pass to Lua
        let lua_item = lua.create_table()?;
        lua_item.set("attack_dice", item.attack_dice.clone())?;
        lua_item.set("modifier", item.modifier)?;
        lua_item.set("attribute_modifier", item.attribute_modifier.clone())?;
        lua_item.set("slot", item.slot.clone())?;
        lua_item.set("two_handed", item.two_handed)?;

        // Call on_get_attack_damage
        let on_get_attack_damage: Function = lua.globals().get("on_get_attack_damage")?;
        let damage: i32 = on_get_attack_damage.call(lua_item.clone())?;
        println!(" -> Damage: {}", damage);

        // Call on_check_accuracy
        let on_check_accuracy: Function = lua.globals().get("on_check_accuracy")?;
        let accuracy: f32 = on_check_accuracy.call(lua_item)?;
        println!(" -> Accuracy: {:.0}%", accuracy * 100.0);
}