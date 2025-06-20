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
use std::{cell::RefCell, rc::Rc, sync::{mpsc, Arc, Mutex}};

use macroquad::miniquad::ElapsedQuery;

use crate::{lua_interface::LuaInterfaceRc, maps::{map::{GeneratedMap, Map}, map_generator::{Border, BorderFlags, GenerationParams, MapAssignment, MapGenerator, MapStatus, MapTheme}, GRID_HEIGHT, GRID_WIDTH}, monster_type::MonsterTypes, position::Position};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct OverworldPos {
    pub floor: usize,
    pub x: usize,
    pub y: usize,
}

pub struct Overworld {
    pub maps: Rc<RefCell<Vec<[[Option<Rc<RefCell<Map>>>; 5]; 5]>>>,
}

impl Overworld {
    pub fn new() -> Self {
        let maps: Rc<RefCell<Vec<[[Option<Rc<RefCell<Map>>>; 5]; 5]>>> =
            Rc::new(RefCell::new(vec![
                std::array::from_fn(|_| std::array::from_fn(|_| None))
            ]));
        Self {
            maps: maps,
        }
    }

    pub fn add_map(&self, opos: OverworldPos, generated_map: Arc<Mutex<GeneratedMap>>) -> Rc<RefCell<Map>> {
        let mut maps_guard = self.maps.borrow_mut();
        if opos.floor >= maps_guard.len() {
            maps_guard.resize_with(opos.floor + 1, || {
                std::array::from_fn(|_| std::array::from_fn(|_| None))
            });
        }
        
        if maps_guard[opos.floor][opos.x][opos.y].is_none() {
            let map = Rc::new(RefCell::new(Map::from_generated_map(generated_map.lock().unwrap().clone())));
            maps_guard[opos.floor][opos.x][opos.y] = Some(Rc::clone(&map));
            map
        } else {
            panic!("Map at position {:?} already exists!", opos);
        }
    }

    pub fn get_map_ptr(&self, opos: OverworldPos) -> Option<Rc<RefCell<Map>>> {
        let maps_guard = self.maps.borrow_mut();
        if opos.floor < maps_guard.len() && opos.x < 5 && opos.y < 5 {
            maps_guard[opos.floor][opos.x][opos.y].as_ref().map(Rc::clone)
        } else {
            None
        }
    }
}

pub struct OverworldGenerator {
    pub generated_maps: Arc<Mutex<Vec<[[Option<Arc<Mutex<GeneratedMap>>>; 5]; 5]>>>,
    map_generator: MapGenerator,
}

impl OverworldGenerator {
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

            if let Some(neigh_map) = self.get_generated_map_ptr(OverworldPos { floor: opos.floor,
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
    
    pub async fn new(lua_interface: &LuaInterfaceRc, monster_types: &MonsterTypes) -> Arc<Mutex<Self>> {
        let generated_maps: Arc<Mutex<Vec<[[Option<Arc<Mutex<GeneratedMap>>>; 5]; 5]>>> =
            Arc::new(Mutex::new(vec![
                std::array::from_fn(|_| std::array::from_fn(|_| None))
            ]));

        let map_generator = MapGenerator::new(lua_interface, monster_types).await;

        let overworld = Arc::new(Mutex::new(Self {
            generated_maps: Arc::clone(&generated_maps),
            //maps: Arc::clone(&maps),
            map_generator,
        }));

        let overworld_weak = Arc::downgrade(&overworld);
        let generated_maps_clone = Arc::clone(&generated_maps);

        let callback = Box::new(move |assignment: MapAssignment| {
            let mut generated_maps = generated_maps_clone.lock().unwrap();
            let OverworldPos { floor, x, y } = assignment.opos;

            if floor >= generated_maps.len() {
                generated_maps.resize_with(floor + 1, || {
                    std::array::from_fn(|_| std::array::from_fn(|_| None))
                });
            }

            generated_maps[floor][x][y] = Some(assignment.map);
            drop(generated_maps);

            if x == 2 && y == 2 && floor == 0 {
                let stairs_pos = generated_maps_clone.lock().unwrap()
                    .get(0).and_then(|floor| floor[2][2].as_ref())
                    .and_then(|map| map.lock().unwrap().downstair_teleport);
                
                // This is the center map, setup adjacent maps
                if let Some(overworld_strong) = overworld_weak.upgrade() {
                    let mut o = overworld_strong.lock().unwrap();
                    o.setup_adjacent_maps(floor, x, y, stairs_pos.unwrap_or_else(|| panic!()));
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

    pub fn setup_adjacent_maps(&mut self, floor: usize, x: usize, y: usize, stairs_pos: Position) {
        for dx in -1i32..=1 {
            for dy in -1i32..=1 {
                // Skip diagonals
                if dx.abs() + dy.abs() != 1 { continue; }
                let new_x = x as i32 + dx;
                let new_y = y as i32 + dy;
                if new_x >= 0 && new_x < 5 && new_y >= 0 && new_y < 5 {
                    let opos = OverworldPos { floor, x: new_x as usize, y: new_y as usize };
                    // Check if this adjacent map is already generated
                    if self.get_generated_map_ptr(opos).is_none() {
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
        let opos = OverworldPos { floor: floor + 1, x: 2usize, y: 2usize };
        let mut gen_params = GenerationParams::default();
        gen_params.borders = BorderFlags::TOP | BorderFlags::BOTTOM | BorderFlags::LEFT | BorderFlags::RIGHT | BorderFlags::DOWN;
        gen_params.theme = MapTheme::Chasm;
        gen_params.force_regen = true;
        gen_params.predefined_start_pos = Some(stairs_pos);
        self.map_generator.request_generation(opos, gen_params);
    }

    pub fn get_generated_map_ptr(&self, opos: OverworldPos) -> Option<Arc<Mutex<GeneratedMap>>> {
        let shared_status_opt = self.map_generator.map_statuses.lock().ok()?.get(&opos).cloned();

        let shared_status = shared_status_opt?;
        let (lock, cvar) = &*shared_status;

        let mut status = lock.lock().ok()?;

        while let MapStatus::Requested = *status {
            status = cvar.wait(status).ok()?; // Wait until notified
        }

        match &*status {
            MapStatus::Ready(map_arc) => {
                let maps_guard = self.generated_maps.lock().ok()?;
                if opos.floor < maps_guard.len() && opos.x < 5 && opos.y < 5 {
                    maps_guard[opos.floor][opos.x][opos.y].as_ref().map(Arc::clone)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}