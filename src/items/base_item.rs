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

use std::{cell::RefCell, rc::Rc, sync::{Arc, RwLock}};

use serde::Deserialize;

use crate::items::{container::Container, holdable::{Armor, Boots, Helmet, HoldableGroupKind, Shield, Weapon}, orb::Orb, teleport::Teleport};

pub fn downcast_arc_item<T: 'static>(arc: &Arc<RwLock<dyn Item>>) -> Option<Arc<RwLock<T>>> {
    if arc.write().unwrap().as_any().is::<T>() {
        // SAFETY: we just checked type, so we can clone Rc and transmute its type
        let raw = Arc::as_ptr(arc) as *const RwLock<T>;
        let cloned = unsafe { Arc::from_raw(raw) };
        let result = Arc::clone(&cloned);
        std::mem::forget(cloned); // avoid dropping original
        Some(result)
    } else {
        None
    }
}

#[derive(Clone, Deserialize, Debug)]
pub struct BaseItemData {
    pub id: u32,
    pub name: String,
    pub description: String,
}

#[derive(Clone, Debug)]
pub enum ItemKind {
    Orb(Orb),
    Teleport(Teleport),
    Holdable(HoldableGroupKind),
    Container(Container),
}

pub trait Item {
    fn get_id(&self) -> u32;
    fn get_name(&self) -> &str;
    // Type query methods
    fn is_weapon(&self) -> bool { false }
    fn is_shield(&self) -> bool { false }
    fn is_helmet(&self) -> bool { false }
    fn is_armor(&self) -> bool { false }
    fn is_boots(&self) -> bool { false }
    fn as_weapon(&self) -> Option<&Weapon> { None }
    fn as_shield(&self) -> Option<&Shield> { None }
    fn as_helmet(&self) -> Option<&Helmet> { None }
    fn as_armor(&self) -> Option<&Armor> { None }
    fn as_boots(&self) -> Option<&Boots> { None }
    fn as_any(&self) -> &dyn std::any::Any;
}

// use std::{collections::HashMap, rc::Rc};

// use crate::map::Map;
// use macroquad::file::load_string;
// use serde::Deserialize;
// use serde_json::from_str;

// // pub enum ItemType {
// //     Portal,
// //     Orb,
// //     Consumable,
// //     Chest,
// //     Health,
// // }

// pub async fn load_holdable_items() -> HashMap<u32, Rc<dyn Item + 'static>> {
//     let file = load_string("assets/items.json").await.unwrap();
//     let list: Vec<HoldableItemKind> = from_str(&file).unwrap();

//     let mut items: HashMap<u32, Rc<dyn Item>> = HashMap::new();
//     for item in list {
//         match item {
//             HoldableItemKind::Weapon(weapon) => {
//                 items.insert(weapon.index, Rc::new(weapon));
//             }
//             HoldableItemKind::Shield(shield) => {
//                 items.insert(shield.index, Rc::new(shield));
//             }
//             HoldableItemKind::Helmet(helmet) => {
//                 items.insert(helmet.index, Rc::new(helmet));
//             }
//             HoldableItemKind::Armor(armor) => {
//                 items.insert(armor.index, Rc::new(armor));
//             }
//             HoldableItemKind::Boots(boots) => {
//                 items.insert(boots.index, Rc::new(boots));
//             }
//         }
//     }
//     items
// }



// pub trait Item {
//    fn pickup(&self); 
// }

// impl Item for ItemKind {
//     fn pickup(&self) {
//         match self {
//             ItemKind::Orb(orb) => orb.pickup(),
//             ItemKind::Portal(portal) => portal.pickup(),
//             //ItemKind::Holdable(holdable) => holdable.pickup(),
//         }
//     }
// }

// #[derive(Clone, Debug)]
// pub struct Orb {
//     //
// }

// impl Item for Orb {
//     fn pickup(&self) {
//         //ItemType::Orb
//     }
// }

// // pub trait Consumable: Item {
// //     fn use_item(&mut self);
// // }

// #[derive(Clone, Debug)]
// pub struct Portal {
//     // pub destination: Option<Map>,
//     pub active: bool,
// }

// impl Item for Portal {
//     fn pickup(&self) {
//         //ItemType::Orb
//     }
// }


// pub trait Holdable: Item {
// }

// #[derive(Clone, Debug, Deserialize)]
// pub struct Weapon {
//     pub index: u32,
//     pub name: String,
//     pub attack_dice: Vec<u32>,
//     pub modifier: i32,
//     pub attribute_modifier: String,
//     pub required: Vec<(String, u32)>,
//     pub slot: String,
//     #[serde(rename = "two-handed")]
//     pub two_handed: bool,
// }

// impl Item for Weapon {
//     fn pickup(&self) {
//         // Handle weapon pickup logic
//     }
// }

// impl Holdable for Weapon {
    
// }

// #[derive(Clone, Debug, Deserialize)]
// pub struct Shield {
//     pub index: u32,
// }

// impl Item for Shield {
//     fn pickup(&self) {
//         // Handle shield pickup logic
//     }
// }

// impl Holdable for Shield {
    
// }

// #[derive(Clone, Debug, Deserialize)]
// pub struct Helmet {
//     pub index: u32,
// }

// impl Item for Helmet {
//     fn pickup(&self) {
//         // Handle helmet pickup logic
//     }
// }

// impl Holdable for Helmet {
    
// }

// #[derive(Clone, Debug, Deserialize)]
// pub struct Armor {
//     pub index: u32,
// }

// impl Item for Armor {
//     fn pickup(&self) {
//         // Handle armor pickup logic
//     }
// }

// impl Holdable for Armor {
    
// }

// #[derive(Clone, Debug, Deserialize)]
// pub struct Boots {
//     pub index: u32,
// }

// impl Item for Boots {
//     fn pickup(&self) {
//         // Handle boots pickup logic
//     }
// }

// impl Holdable for Boots {
    
// }