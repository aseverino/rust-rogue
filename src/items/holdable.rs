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

use serde::Deserialize;

use crate::items::base_item::{BaseItemData, Item};
use crate::lua_interface::LuaScripted;

#[derive(Debug, Deserialize)]
pub struct BaseHoldableItemData {
    #[serde(flatten)]
    pub base_item: BaseItemData,
    pub class: String,
    pub modifier: i32,
    pub attribute_modifier: String,
    pub required: Vec<Vec<serde_json::Value>>,
    pub slot: String,
    pub script: Option<String>,
    #[serde(default)]
    pub scripted: bool,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum HoldableGroup {
    Weapons { weapons: Vec<Weapon> },
    Armor { armor: Vec<Armor> },
    Shields { shields: Vec<Shield> },
    Helmets { helmets: Vec<Helmet> },
    Boots { boots: Vec<Boots> },
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum HoldableGroupKind {
    Weapons,
    Armor,
    Shields,
    Helmets,
    Boots,
}

#[macro_export]
macro_rules! impl_lua_scripted {
    ($t:ty, [ $($func_name:literal),* $(,)? ]) => {
        impl LuaScripted for $t {
            fn script_id(&self) -> u32 {
                self.base_holdable.base_item.id
            }

            fn script_path(&self) -> Option<String> {
                self.base_holdable.script.clone()
            }

            fn is_scripted(&self) -> bool {
                self.base_holdable.scripted
            }

            fn functions(&self) -> Vec<String> {
                vec![
                    $( String::from($func_name) ),*
                ]
            }
        }
    };
}

#[derive(Debug, Deserialize)]
pub struct Weapon {
    #[serde(flatten)]
    pub base_holdable: BaseHoldableItemData,
    pub attack_dice: Vec<u32>,
    #[serde(rename = "two-handed")]
    pub two_handed: bool,
}

impl_lua_scripted!(Weapon, [ "on_get_attack_damage" ]);

impl Item for Weapon {
    fn get_id(&self) -> u32 {
        self.base_holdable.base_item.id
    }
    fn get_name(&self) -> &str {
        &self.base_holdable.base_item.name
    }

    fn is_weapon(&self) -> bool {
        true
    }

    fn as_weapon(&self) -> Option<&Weapon> {
        Some(self)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug, Deserialize)]
pub struct Armor {
    #[serde(flatten)]
    pub base_holdable: BaseHoldableItemData,
    pub defense_dice: Vec<u32>,
}

impl_lua_scripted!(Armor, []);

impl Item for Armor {
    fn get_id(&self) -> u32 {
        self.base_holdable.base_item.id
    }
    fn get_name(&self) -> &str {
        &self.base_holdable.base_item.name
    }

    fn is_armor(&self) -> bool {
        true
    }

    fn as_armor(&self) -> Option<&Armor> {
        Some(self)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug, Deserialize)]
pub struct Shield {
    #[serde(flatten)]
    pub base_holdable: BaseHoldableItemData,
}

impl_lua_scripted!(Shield, []);

impl Item for Shield {
    fn get_id(&self) -> u32 {
        self.base_holdable.base_item.id
    }
    fn get_name(&self) -> &str {
        &self.base_holdable.base_item.name
    }

    fn is_shield(&self) -> bool {
        true
    }

    fn as_shield(&self) -> Option<&Shield> {
        Some(self)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug, Deserialize)]
pub struct Helmet {
    #[serde(flatten)]
    pub base_holdable: BaseHoldableItemData,
}

impl_lua_scripted!(Helmet, []);

impl Item for Helmet {
    fn get_id(&self) -> u32 {
        self.base_holdable.base_item.id
    }
    fn get_name(&self) -> &str {
        &self.base_holdable.base_item.name
    }

    fn is_helmet(&self) -> bool {
        true
    }

    fn as_helmet(&self) -> Option<&Helmet> {
        Some(self)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug, Deserialize)]
pub struct Boots {
    #[serde(flatten)]
    pub base_holdable: BaseHoldableItemData,
}

impl_lua_scripted!(Boots, []);

impl Item for Boots {
    fn get_id(&self) -> u32 {
        self.base_holdable.base_item.id
    }
    fn get_name(&self) -> &str {
        &self.base_holdable.base_item.name
    }

    fn is_boots(&self) -> bool {
        true
    }

    fn as_boots(&self) -> Option<&Boots> {
        Some(self)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}