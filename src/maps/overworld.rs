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
use std::{sync::{mpsc, Arc, Mutex}};

use macroquad::miniquad::ElapsedQuery;

use crate::{maps::{map::Map, map_generator::{Border, BorderFlags, GenerationParams, MapAssignment, MapGenerator, MapTheme}, GRID_HEIGHT, GRID_WIDTH}, position::Position};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct OverworldPos {
    pub floor: usize,
    pub x: usize,
    pub y: usize,
}

pub struct Overworld {
    pub maps: Arc<Mutex<Vec<[[Option<Arc<Mutex<Map>>>; 5]; 5]>>>, // this is a list of floors; each floor has fixed 5x5 maps; The maps are represented by their IDs
    map_generator: MapGenerator,
}

impl Overworld {
    fn fill_predefined_borders(&self,
                               opos: OverworldPos,
                               params: &mut GenerationParams) {

        use Border::*;

        const DIRS: &[(Border, i32, i32, fn(&mut Position))] = &[
            // neighbour ABOVE us → take its BOTTOM edge, then clamp to y=0
            (Top,    0, -1, |p| p.y = 0),
            // neighbour to our RIGHT → take its LEFT edge, then clamp to x=GRID_WIDTH-1
            (Right,  1,  0, |p| p.x = GRID_WIDTH - 1),
            // neighbour BELOW us → take its TOP edge, then clamp to y=GRID_HEIGHT-1
            (Bottom, 0,  1, |p| p.y = GRID_HEIGHT - 1),
            // neighbour to our LEFT → take its RIGHT edge, then clamp to x=0
            (Left,  -1,  0, |p| p.x = 0),
        ];

        for (side, dx, dy, mirror) in DIRS {
            let nx = opos.x as i32 + dx;
            let ny = opos.y as i32 + dy;

            // outside the 5×5 slice – ignore
            if !(0..5).contains(&nx) || !(0..5).contains(&ny) { continue; }

            if let Some(neigh_map) = self.get_map_ptr(OverworldPos { floor: opos.floor,
                                                                     x: nx as usize,
                                                                     y: ny as usize }) {
                let mut vec = neigh_map.lock().unwrap()
                                        .border_positions[side.opposite() as usize]
                                        .clone();

                // convert neighbour coordinates to *our* side
                for p in &mut vec { mirror(p); }

                params.predefined_borders[*side as usize] = vec;

                // make sure this side is opened in the new map too
                params.borders |= match side {
                    Top    => BorderFlags::TOP,
                    Right  => BorderFlags::RIGHT,
                    Bottom => BorderFlags::BOTTOM,
                    Left   => BorderFlags::LEFT,
                };
            }
        }
    }
    
    pub async fn new() -> Arc<Mutex<Self>> {
        let maps: Arc<Mutex<Vec<[[Option<Arc<Mutex<Map>>>; 5]; 5]>>> =
            Arc::new(Mutex::new(vec![
                std::array::from_fn(|_| std::array::from_fn(|_| None))
            ]));

        let map_generator = MapGenerator::new().await;

        let overworld = Arc::new(Mutex::new(Self {
            maps: Arc::clone(&maps),
            map_generator,
        }));

        let overworld_weak = Arc::downgrade(&overworld);
        let maps_clone = Arc::clone(&maps);

        let callback = Box::new(move |assignment: MapAssignment| {
            let mut maps = maps_clone.lock().unwrap();
            let OverworldPos { floor, x, y } = assignment.opos;

            if floor >= maps.len() {
                maps.resize_with(floor + 1, || {
                    std::array::from_fn(|_| std::array::from_fn(|_| None))
                });
            }

            maps[floor][x][y] = Some(Arc::new(Mutex::new(assignment.map)));
            drop(maps);

            if x == 2 && y == 2 && floor == 0 {
                // This is the center map, setup adjacent maps
                if let Some(overworld_strong) = overworld_weak.upgrade() {
                    let mut o = overworld_strong.lock().unwrap();
                    o.setup_adjacent_maps(floor, x, y);
                }
            }
        });

        overworld.lock().unwrap().map_generator.set_callback(callback);
        
        // Request center map
        let mut gen_params = GenerationParams::default();
        gen_params.borders = BorderFlags::TOP | BorderFlags::BOTTOM | BorderFlags::LEFT | BorderFlags::RIGHT | BorderFlags::DOWN;
        gen_params.theme = MapTheme::Chasm;

        let center = OverworldPos { floor: 0, x: 2, y: 2 };
        overworld.lock().unwrap().map_generator.request_generation(center, gen_params);

        overworld
    }

    pub fn setup_adjacent_maps(&mut self, floor: usize, x: usize, y: usize) {
        for dx in -1i32..=1 {
            for dy in -1i32..=1 {
                // Skip diagonals
                if dx.abs() + dy.abs() != 1 { continue; }
                let new_x = x as i32 + dx;
                let new_y = y as i32 + dy;
                if new_x >= 0 && new_x < 5 && new_y >= 0 && new_y < 5 {
                    let opos = OverworldPos { floor, x: new_x as usize, y: new_y as usize };
                    // Check if this adjacent map is already generated
                    if self.get_map_ptr(opos).is_none() {
                        // If not, setup the GenerationParams with appropriate borders (they should not have borders at the edges of the 5x5 grid)
                        let mut gen_params = GenerationParams::default();
                        if new_x != 0 {
                            gen_params.borders |= BorderFlags::LEFT;
                        }
                        if new_x != 4 {
                            gen_params.borders |= BorderFlags::RIGHT;
                        }
                        if new_y != 0 {
                            gen_params.borders |= BorderFlags::TOP;
                        }
                        if new_y != 4 {
                            gen_params.borders |= BorderFlags::BOTTOM;
                        }
                        gen_params.borders |= BorderFlags::DOWN;

                        gen_params.theme = MapTheme::Chasm;
                        self.fill_predefined_borders(opos, &mut gen_params);
                        self.map_generator.request_generation(opos, gen_params);
                    }
                }
            }
        }
    }

    pub fn get_map_ptr(&self, opos: OverworldPos) -> Option<Arc<Mutex<Map>>> {
        let maps_guard = self.maps.lock().ok()?;
        if opos.floor < maps_guard.len() && opos.x < 5 && opos.y < 5 {
            maps_guard[opos.floor][opos.x][opos.y].as_ref().map(Arc::clone)
        } else {
            None
        }
    }
}