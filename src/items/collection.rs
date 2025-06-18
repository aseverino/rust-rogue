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

use std::{cell::RefCell, collections::HashMap, rc::{Rc, Weak}};

use crate::{items::{base_item::Item, holdable::{HoldableGroup, HoldableGroupKind}}, lua_interface::LuaInterface};

pub struct Items {
    pub items: Vec<Rc<RefCell<dyn Item>>>,
    holdable_items: HashMap<HoldableGroupKind, Vec<Weak<RefCell<dyn Item>>>>// = HashMap::new();
}

impl Items {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            holdable_items: HashMap::new(),
        }
    }
    pub async fn load_holdable_items(&mut self, lua_interface: &mut LuaInterface) {
        let json_str = std::fs::read_to_string("assets/items.json").unwrap();
        let groups: Vec<HoldableGroup> = serde_json::from_str(&json_str).unwrap();

        for group in groups {
            match group {
                HoldableGroup::Weapons { weapons } => {
                    for mut weapon in weapons {
                        if weapon.base_holdable.script.is_some() {
                            // Load the Lua script for the weapon
                            let script_result = lua_interface.load_script_for_weapon(&weapon);
                            if let Err(e) = script_result {
                                eprintln!("Error loading weapon script: {}", e);
                            } else {
                                weapon.base_holdable.scripted = script_result.unwrap();
                            }
                        }

                        let weapon_ref = Rc::new(RefCell::new(weapon));
                        self.items.push(weapon_ref);
                        let ptr_copy = self.items.last().unwrap().clone();
                        self.holdable_items.entry(HoldableGroupKind::Weapons).or_default().push(Rc::downgrade(&ptr_copy));
                    }
                }
                HoldableGroup::Armor { armor } => {
                    for armor in armor {
                        self.items.push(Rc::new(RefCell::new(armor)));
                        let ptr_copy = self.items.last().unwrap().clone();
                        self.holdable_items.entry(HoldableGroupKind::Armor).or_default().push(Rc::downgrade(&ptr_copy));
                    }
                }
                HoldableGroup::Shields { shields } => {
                    for shield in shields {
                        self.items.push(Rc::new(RefCell::new(shield)));
                        let ptr_copy = self.items.last().unwrap().clone();
                        self.holdable_items.entry(HoldableGroupKind::Shields).or_default().push(Rc::downgrade(&ptr_copy));
                    }
                }
                HoldableGroup::Helmets { helmets } => {
                    for helmet in helmets {
                        self.items.push(Rc::new(RefCell::new(helmet)));
                        let ptr_copy = self.items.last().unwrap().clone();
                        self.holdable_items.entry(HoldableGroupKind::Helmets).or_default().push(Rc::downgrade(&ptr_copy));
                    }
                }
                HoldableGroup::Boots { boots } => {
                    for boots in boots {
                        self.items.push(Rc::new(RefCell::new(boots)));
                        let ptr_copy = self.items.last().unwrap().clone();
                        self.holdable_items.entry(HoldableGroupKind::Boots).or_default().push(Rc::downgrade(&ptr_copy));
                    }
                }
            }
        };
    }
}
