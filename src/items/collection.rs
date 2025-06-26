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

use std::{
    any::Any,
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
    sync::{Arc, RwLock, Weak},
};

use crate::{
    items::{
        base_item::Item,
        holdable::{HoldableGroup, HoldableGroupKind},
    },
    lua_interface::LuaInterfaceRc,
};

type ItemsById = HashMap<u32, Item>;
pub struct Items {
    pub items_by_id: ItemsById,
    pub items_ids_by_tier: Vec<HashSet<u32>>,
    //holdable_items: HashMap<HoldableGroupKind, Vec<Weak<RwLock<dyn Item>>>>// = HashMap::new();
}

pub type ItemsArc = Arc<RwLock<Items>>;

impl Items {
    pub fn new() -> Self {
        Self {
            items_by_id: HashMap::new(),
            items_ids_by_tier: Vec::new(),
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
                            let script_result = lua_interface.load_script(&mut weapon);
                            if let Err(e) = script_result {
                                eprintln!("Error loading weapon script: {}", e);
                            } else {
                                weapon.base_holdable.scripted = script_result.unwrap();
                            }
                        }

                        let id = weapon.base_holdable.base_item.id;
                        let tier = weapon.base_holdable.tier;

                        self.items_by_id.insert(id, Item::Weapon(weapon));

                        // Add the weapon to the items_ids_by_tier map
                        if tier as usize >= self.items_ids_by_tier.len() {
                            self.items_ids_by_tier
                                .resize(tier as usize + 1, HashSet::new());
                        }
                        self.items_ids_by_tier[tier as usize].insert(id);
                    }
                }
                HoldableGroup::Armor { armor } => {
                    for mut armor_item in armor {
                        if armor_item.base_holdable.script.is_some() {
                            // Load the Lua script for the armor
                            let script_result = lua_interface.load_script(&mut armor_item);
                            if let Err(e) = script_result {
                                eprintln!("Error loading armor script: {}", e);
                            } else {
                                armor_item.base_holdable.scripted = script_result.unwrap();
                            }
                        }

                        let id = armor_item.base_holdable.base_item.id;
                        let tier = armor_item.base_holdable.tier;

                        self.items_by_id.insert(id, Item::Armor(armor_item));

                        // Add the armor to the items_ids_by_tier map
                        if tier as usize >= self.items_ids_by_tier.len() {
                            self.items_ids_by_tier
                                .resize(tier as usize + 1, HashSet::new());
                        }
                        self.items_ids_by_tier[tier as usize].insert(id);
                    }
                }

                HoldableGroup::Helmets { helmets } => {
                    for mut helmet_item in helmets {
                        if helmet_item.base_holdable.script.is_some() {
                            // Load the Lua script for the helmet
                            let script_result = lua_interface.load_script(&mut helmet_item);
                            if let Err(e) = script_result {
                                eprintln!("Error loading helmet script: {}", e);
                            } else {
                                helmet_item.base_holdable.scripted = script_result.unwrap();
                            }
                        }

                        let id = helmet_item.base_holdable.base_item.id;
                        let tier = helmet_item.base_holdable.tier;

                        self.items_by_id.insert(id, Item::Helmet(helmet_item));

                        // Add the helmet to the items_ids_by_tier map
                        if tier as usize >= self.items_ids_by_tier.len() {
                            self.items_ids_by_tier
                                .resize(tier as usize + 1, HashSet::new());
                        }
                        self.items_ids_by_tier[tier as usize].insert(id);
                    }
                }
                HoldableGroup::Boots { boots } => {
                    for mut boots_item in boots {
                        if boots_item.base_holdable.script.is_some() {
                            // Load the Lua script for the boots
                            let script_result = lua_interface.load_script(&mut boots_item);
                            if let Err(e) = script_result {
                                eprintln!("Error loading boots script: {}", e);
                            } else {
                                boots_item.base_holdable.scripted = script_result.unwrap();
                            }
                        }

                        let id = boots_item.base_holdable.base_item.id;
                        let tier = boots_item.base_holdable.tier;

                        self.items_by_id.insert(id, Item::Boots(boots_item));

                        // Add the boots to the items_ids_by_tier map
                        if tier as usize >= self.items_ids_by_tier.len() {
                            self.items_ids_by_tier
                                .resize(tier as usize + 1, HashSet::new());
                        }
                        self.items_ids_by_tier[tier as usize].insert(id);
                    }
                }
                HoldableGroup::Shields { shields } => {
                    for mut shield_item in shields {
                        if shield_item.base_holdable.script.is_some() {
                            // Load the Lua script for the shield
                            let script_result = lua_interface.load_script(&mut shield_item);
                            if let Err(e) = script_result {
                                eprintln!("Error loading shield script: {}", e);
                            } else {
                                shield_item.base_holdable.scripted = script_result.unwrap();
                            }
                        }

                        let id = shield_item.base_holdable.base_item.id;
                        let tier = shield_item.base_holdable.tier;

                        self.items_by_id.insert(id, Item::Shield(shield_item));

                        // Add the shield to the items_ids_by_tier map
                        if tier as usize >= self.items_ids_by_tier.len() {
                            self.items_ids_by_tier
                                .resize(tier as usize + 1, HashSet::new());
                        }
                        self.items_ids_by_tier[tier as usize].insert(id);
                    }
                }
            }
        }
    }
}
