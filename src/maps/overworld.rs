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

use core::panic;
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
};

use crate::maps::{
    generated_map::GeneratedMap,
    map::{Map, MapRc},
};

#[derive(Clone, Debug, PartialEq)]
pub enum VisitedState {
    Unvisited,
    Peeked,
    Visited,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct OverworldPos {
    pub floor: usize,
    pub x: usize,
    pub y: usize,
}

impl OverworldPos {
    pub fn new(floor: usize, x: usize, y: usize) -> Self {
        Self { floor, x, y }
    }
}

pub struct Overworld {
    pub maps: Rc<RefCell<Vec<[[Option<MapRc>; 5]; 5]>>>,
}

impl Overworld {
    pub fn new() -> Self {
        let maps: Rc<RefCell<Vec<[[Option<MapRc>; 5]; 5]>>> =
            Rc::new(RefCell::new(vec![std::array::from_fn(|_| {
                std::array::from_fn(|_| None)
            })]));
        Self { maps: maps }
    }

    pub fn clear_unvisited(&mut self, opos: OverworldPos) {
        let to_clear = {
            let maps = self.maps.borrow();
            let mut result = Vec::new();
            for (row_idx, row) in maps[opos.floor].iter().enumerate() {
                for (col_idx, map_opt) in row.iter().enumerate() {
                    if (row_idx, col_idx) == (opos.x, opos.y) {
                        continue; // Skip the current position
                    }
                    if let Some(map) = map_opt {
                        if map.0.borrow().generated_map.visited_state != VisitedState::Visited {
                            result.push((row_idx, col_idx));
                        }
                    }
                }
            }
            result
        };

        {
            let mut maps = self.maps.borrow_mut();
            for (row_idx, col_idx) in to_clear {
                println!(
                    "Overworld: Clearing unvisited map at position: ({}, {}) on floor {}",
                    row_idx, col_idx, opos.floor
                );
                maps[opos.floor][row_idx][col_idx] = None;
            }

            if maps.len() > opos.floor + 1 {
                // Clear the map below if it exists
                maps[opos.floor + 1][2][2] = None;
            }
        }
    }

    pub fn add_map(&self, opos: OverworldPos, generated_map: Arc<Mutex<GeneratedMap>>) -> MapRc {
        let mut maps_guard = self.maps.borrow_mut();
        if opos.floor >= maps_guard.len() {
            maps_guard.resize_with(opos.floor + 1, || {
                std::array::from_fn(|_| std::array::from_fn(|_| None))
            });
        }

        if maps_guard[opos.floor][opos.x][opos.y].is_none() {
            // This is getting ridiculous
            let map = MapRc(Rc::new(RefCell::new(Map::new(
                generated_map.lock().unwrap().clone(),
            ))));
            maps_guard[opos.floor][opos.x][opos.y] = Some(map.clone());
            map
        } else {
            panic!("Map at position {:?} already exists!", opos);
        }
    }

    pub fn get_map_ptr(&self, opos: OverworldPos) -> Option<MapRc> {
        let maps_guard = self.maps.borrow();
        if opos.floor < maps_guard.len() && opos.x < 5 && opos.y < 5 {
            maps_guard[opos.floor][opos.x][opos.y].as_ref().cloned()
        } else {
            None
        }
    }
}
