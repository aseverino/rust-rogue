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

use crate::items::base_item::{BaseItemData, Item};

#[derive(Clone, Debug)]
pub struct Container {
    pub base_item: BaseItemData,
    pub items: Vec<u32>
}

impl Container {
    pub fn new() -> Self {
        Self {
            base_item: BaseItemData {
                id: 0,
                name: String::from("Generic Container"),
                description: String::from("A container to hold items.")
            },
            items: Vec::new(),
        }
    }

    pub fn add_item(&mut self, item_id: u32) {
        self.items.push(item_id);
    }

    pub fn remove_item(&mut self, item_id: u32) -> Option<u32> {
        if let Some(pos) = self.items.iter().position(|&id| id == item_id) {
            Some(self.items.remove(pos))
        } else {
            None
        }
    }
}