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

use std::{any::Any, cell::RefCell, collections::HashMap, sync::{Arc, RwLock, Weak}};

use crate::{items::{base_item::Item, holdable::{HoldableGroup, HoldableGroupKind}}, lua_interface::LuaInterfaceRc};

pub struct Items {
    pub items: HashMap<u32, Item>,
    //holdable_items: HashMap<HoldableGroupKind, Vec<Weak<RwLock<dyn Item>>>>// = HashMap::new();
}

impl Items {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
            //holdable_items: HashMap::new(),
        }
    }
    pub async fn load_holdable_items(&mut self, lua_interface_rc: &LuaInterfaceRc) {
        let mut lua_interface = lua_interface_rc.borrow_mut();
        let json_str = std::fs::read_to_string("assets/items.json").unwrap();
        let groups: Vec<HoldableGroup> = serde_json::from_str(&json_str).unwrap();

        for group in groups {
            match group {
                HoldableGroup::Weapons { weapons } => {
                    for mut weapon in weapons {
                        if weapon.base_holdable.script.is_some() {
                            // Load the Lua script for the weapon
                            let script_result = lua_interface.load_script(&weapon);
                            if let Err(e) = script_result {
                                eprintln!("Error loading weapon script: {}", e);
                            } else {
                                weapon.base_holdable.scripted = script_result.unwrap();
                            }
                        }

                        self.items.insert(weapon.base_holdable.base_item.id, Item::Weapon(weapon));
                    }
                }
                HoldableGroup::Armor { armor } => {
                    for mut armor_item in armor {
                        if armor_item.base_holdable.script.is_some() {
                            // Load the Lua script for the armor
                            let script_result = lua_interface.load_script(&armor_item);
                            if let Err(e) = script_result {
                                eprintln!("Error loading armor script: {}", e);
                            } else {
                                armor_item.base_holdable.scripted = script_result.unwrap();
                            }
                        }

                        self.items.insert(armor_item.base_holdable.base_item.id, Item::Armor(armor_item));
                    }
                }

                HoldableGroup::Helmets { helmets } => {
                    for mut helmet_item in helmets {
                        if helmet_item.base_holdable.script.is_some() {
                            // Load the Lua script for the helmet
                            let script_result = lua_interface.load_script(&helmet_item);
                            if let Err(e) = script_result {
                                eprintln!("Error loading helmet script: {}", e);
                            } else {
                                helmet_item.base_holdable.scripted = script_result.unwrap();
                            }
                        }

                        self.items.insert(helmet_item.base_holdable.base_item.id, Item::Helmet(helmet_item));
                    }
                }
                HoldableGroup::Boots { boots } => {
                    for mut boots_item in boots {
                        if boots_item.base_holdable.script.is_some() {
                            // Load the Lua script for the boots
                            let script_result = lua_interface.load_script(&boots_item);
                            if let Err(e) = script_result {
                                eprintln!("Error loading boots script: {}", e);
                            } else {
                                boots_item.base_holdable.scripted = script_result.unwrap();
                            }
                        }

                        self.items.insert(boots_item.base_holdable.base_item.id, Item::Boots(boots_item));
                    }
                }
                HoldableGroup::Shields { shields } => {
                    for mut shield_item in shields {
                        if shield_item.base_holdable.script.is_some() {
                            // Load the Lua script for the shield
                            let script_result = lua_interface.load_script(&shield_item);
                            if let Err(e) = script_result {
                                eprintln!("Error loading shield script: {}", e);
                            } else {
                                shield_item.base_holdable.scripted = script_result.unwrap();
                            }
                        }

                        self.items.insert(shield_item.base_holdable.base_item.id, Item::Shield(shield_item));
                    }
                }
            }
        };
    }
}
