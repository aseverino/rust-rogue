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

use std::{sync::{mpsc, Arc, Mutex}};

use crate::maps::{map::Map, map_generator::{GenerationParams, MapAssignment, MapGenerator, MapTheme, OverworldPos}};

pub struct Overworld {
    pub maps: Arc<Mutex<Vec<[[Option<Arc<Mutex<Map>>>; 5]; 5]>>>, // this is a list of floors; each floor has fixed 5x5 maps; The maps are represented by their IDs
    map_generator: MapGenerator,
}

impl Overworld {
    pub fn new() -> Self {
        let maps: Arc<Mutex<Vec<[[Option<Arc<Mutex<Map>>>; 5]; 5]>>> =
            Arc::new(Mutex::new(vec![
                std::array::from_fn(|_| std::array::from_fn(|_| None))
            ]));

        let maps_clone = Arc::clone(&maps);

        // Define the actual callback function
        let callback = Box::new(move |assignment: MapAssignment| {
            let mut maps = maps_clone.lock().unwrap();
            let OverworldPos { floor, x, y } = assignment.opos;

            // Expand floors as needed
            if floor >= maps.len() {
                maps.resize_with(floor + 1, || {
                    std::array::from_fn(|_| std::array::from_fn(|_| None))
                });
            }

            maps[floor][x][y] = Some(Arc::new(Mutex::new(assignment.map)));
        });

        let map_generator = MapGenerator::new(callback);

        // Request center map
        let mut gen_params = GenerationParams::default();
        gen_params.exits = 5;
        gen_params.theme = MapTheme::Chasm;

        let center = OverworldPos { floor: 0, x: 2, y: 2 };
        map_generator.request_generation(center, gen_params);

        Self {
            maps,
            map_generator,
        }
    }

    pub fn get_map_ptr(&self, floor: usize, x: usize, y: usize) -> Option<Arc<Mutex<Map>>> {
        let maps_guard = self.maps.lock().ok()?;
        if floor < maps_guard.len() && x < 5 && y < 5 {
            maps_guard[floor][x][y].as_ref().map(Arc::clone)
        } else {
            None
        }
    }
}