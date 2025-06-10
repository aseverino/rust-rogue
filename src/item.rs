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

use crate::map::Map;

// pub enum ItemType {
//     Portal,
//     Orb,
//     Consumable,
//     Chest,
//     Health,
// }

pub trait Item {
   fn pickup(&self); 
}

#[derive(Clone)]
pub enum ItemKind {
    Orb(Orb),
    Portal(Portal),
}

impl Item for ItemKind {
    fn pickup(&self) {
        match self {
            ItemKind::Orb(orb) => orb.pickup(),
            ItemKind::Portal(portal) => portal.pickup(),
        }
    }
}

#[derive(Clone)]
pub struct Orb {
    //
}

impl Item for Orb {
    fn pickup(&self) {
        //ItemType::Orb
    }
}

// pub trait Consumable: Item {
//     fn use_item(&mut self);
// }

#[derive(Clone)]
pub struct Portal {
    // pub destination: Option<Map>,
    pub active: bool,
}

impl Item for Portal {
    fn pickup(&self) {
        //ItemType::Orb
    }
}


pub trait Weapon: Item {
    //fn attack(&self);
}

pub trait Shield: Item {
}

pub trait Helmet: Item {
}

pub trait Armor: Item {
}

pub trait Boots: Item {
}