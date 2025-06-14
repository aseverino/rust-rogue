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

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use rand::rngs::ThreadRng;
use rand::{thread_rng, Rng};

use crate::maps::{map::Map, GRID_WIDTH, GRID_HEIGHT};
use crate::tile::{Tile, TileKind};
use crate::position::Position;
use rand::seq::SliceRandom;

// fn generate_maps() {
//     // Shared state
//     let pair = Arc::new((Mutex::new(false), Condvar::new()));

//     // Clone for the thread
//     let pair_clone = Arc::clone(&pair);

//     // Spawn the thread
//     let handle = thread::spawn(move || {
//         loop {
//             let (lock, cvar) = &*pair_clone;
//             let mut should_poll = lock.lock().unwrap();

//             // Wait until `should_poll` becomes true
//             while !*should_poll {
//                 should_poll = cvar.wait(should_poll).unwrap();
//             }

//             // Do the work
//             println!("Polling something...");
//             thread::sleep(Duration::from_secs(1)); // Simulate polling

//             // Reset signal (optional)
//             *should_poll = false;
//         }
//     });

//     // Main thread controls when polling occurs
//     for i in 0..3 {
//         thread::sleep(Duration::from_secs(2));
//         println!("Signaling thread to poll, iteration {}", i);

//         let (lock, cvar) = &*pair;
//         let mut should_poll = lock.lock().unwrap();
//         *should_poll = true;
//         cvar.notify_one(); // Wake the polling thread
//     }

//     // Optional: join if you want graceful shutdown (add exit condition to the loop)
//     handle.join().unwrap();
// }

// Commands for the background thread
enum Command {
    Generate,
    Stop,
}

pub struct MapGenerator {
    command_tx: Sender<Command>,
    result_rx: Arc<Mutex<Receiver<Map>>>,
    thread_handle: Option<JoinHandle<()>>,
}

impl MapGenerator {
    pub fn new() -> Self {
        let (command_tx, command_rx) = mpsc::channel::<Command>();
        let (result_tx, result_rx) = mpsc::channel::<Map>();
        let result_rx = Arc::new(Mutex::new(result_rx));
        let result_tx_clone = result_tx.clone();

        let handle = {
            let result_tx = result_tx_clone.clone();
            thread::spawn(move || {
                for command in command_rx {
                    match command {
                        Command::Generate => {
                            let map = Self::generate_map();
                            let _ = result_tx.send(map); // send result back
                        }
                        Command::Stop => {
                            println!("[MapGenerator] Stopping...");
                            break;
                        }
                    }
                }
            })
        };

        Self {
            command_tx,
            result_rx,
            thread_handle: Some(handle),
        }
    }

    pub fn request_generation(&self) {
        let _ = self.command_tx.send(Command::Generate);
    }

    // pub fn get_generated_map(&self) -> Option<Map> {
    //     // Try to receive without blocking
    //     self.result_rx.lock().unwrap().recv().ok()
    // }

    pub fn get_generated_map_blocking(&self) -> Option<Map> {
        self.result_rx.lock().unwrap().recv().ok()
    }

    pub fn stop(&mut self) {
        let _ = self.command_tx.send(Command::Stop);
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }

    fn carve_tile(tiles: &mut Vec<Vec<Tile>>, x: usize, y: usize, walkable_cache: &mut Vec<Position>) {
        if x >= GRID_WIDTH || y >= GRID_HEIGHT {
            return;
        }
        if tiles[x][y].kind != TileKind::Floor {
            tiles[x][y].kind = TileKind::Floor;
            walkable_cache.push(Position { x, y });
        }
    }

    fn carve_straight_path(
        tiles: &mut Vec<Vec<Tile>>,
        start: Position,
        end: Position,
        walkable_cache: &mut Vec<Position>,
    ) {
        let mut x = start.x;
        let mut y = start.y;

        while x != end.x {
            if tiles[x][y].kind != TileKind::Floor {
                tiles[x][y].kind = TileKind::Floor;
                walkable_cache.push(Position { x, y });
            }
            x = if end.x > x { x + 1 } else { x - 1 };
        }

        while y != end.y {
            if tiles[x][y].kind != TileKind::Floor {
                tiles[x][y].kind = TileKind::Floor;
                walkable_cache.push(Position { x, y });
            }
            y = if end.y > y { y + 1 } else { y - 1 };
        }
    }

    fn carve_jagged_path(
        tiles: &mut Vec<Vec<Tile>>,
        mut current: Position,
        goal: Position,
        walkable_cache: &mut Vec<Position>,
        rng: &mut ThreadRng,
        radius: usize, // 0 = 1x1, 1 = 3x3, or even 2 = 5x5
    ) {
        while current != goal {
            if radius == 0 {
                // Exact 1x1
                Self::carve_tile(tiles, current.x, current.y, walkable_cache);
            } else if radius == 1 {
                // Exact 2x2 (square)
                Self::carve_tile(tiles, current.x,     current.y,     walkable_cache);
                Self::carve_tile(tiles, current.x + 1, current.y,     walkable_cache);
                Self::carve_tile(tiles, current.x,     current.y + 1, walkable_cache);
                Self::carve_tile(tiles, current.x + 1, current.y + 1, walkable_cache);
            } else {
                // Circular area
                for dx in -(radius as isize)..=(radius as isize) {
                    for dy in -(radius as isize)..=(radius as isize) {
                        let nx = current.x as isize + dx;
                        let ny = current.y as isize + dy;
                        if nx >= 0 && ny >= 0 && nx < GRID_WIDTH as isize && ny < GRID_HEIGHT as isize {
                            Self::carve_tile(tiles, nx as usize, ny as usize, walkable_cache);
                        }
                    }
                }
            }

            // Movement logic (same as before)
            let dx = goal.x as isize - current.x as isize;
            let dy = goal.y as isize - current.y as isize;

            let mut step_x = 0;
            let mut step_y = 0;

            if rng.gen_bool(0.6) && dx != 0 {
                step_x = dx.signum();
            }
            if rng.gen_bool(0.6) && dy != 0 {
                step_y = dy.signum();
            }

            if step_x == 0 && dx != 0 {
                step_x = dx.signum();
            }
            if step_y == 0 && dy != 0 {
                step_y = dy.signum();
            }

            let new_x = (current.x as isize + step_x).clamp(1, GRID_WIDTH as isize - 2) as usize;
            let new_y = (current.y as isize + step_y).clamp(1, GRID_HEIGHT as isize - 2) as usize;
            current = Position { x: new_x, y: new_y };
        }
    }

    pub fn generate_map() -> Map {
        let mut rng = thread_rng();

        let tile_type = if rng.gen_bool(0.5) {
            TileKind::Chasm
        } else {
            TileKind::Wall
        };

        let mut tiles = vec![vec![Tile::new(tile_type); GRID_HEIGHT]; GRID_WIDTH];
        let mut walkable_cache = Vec::new();
        let mut visited = vec![vec![false; GRID_HEIGHT]; GRID_WIDTH];

        const NUM_WALKS: usize = 6;
        const WALK_LENGTH: usize = 80;
        const MIN_DISTANCE_BETWEEN_STARTS: usize = 6;

        let mut start_positions = Vec::new();

        for _ in 0..NUM_WALKS {
            let mut x;
            let mut y;
            // Ensure new walk starts away from previous walks
            loop {
                x = rng.gen_range(3..GRID_WIDTH - 3);
                y = rng.gen_range(3..GRID_HEIGHT - 3);

                let too_close = start_positions.iter().any(|&pos: &Position| {
                    let dx = pos.x as isize - x as isize;
                    let dy = pos.y as isize - y as isize;
                    dx.abs() + dy.abs() < MIN_DISTANCE_BETWEEN_STARTS as isize
                });

                if !too_close {
                    break;
                }
            }

            start_positions.push(Position { x, y });

            // Apply random walk
            for _ in 0..WALK_LENGTH {
                if x < 1 || x >= GRID_WIDTH - 1 || y < 1 || y >= GRID_HEIGHT - 1 {
                    break;
                }

                if !visited[x][y] {
                    tiles[x][y].kind = TileKind::Floor;
                    walkable_cache.push(Position { x, y });
                    visited[x][y] = true;
                }

                match rng.gen_range(0..8) {
                    0 if x > 1 => x -= 1,
                    1 if x < GRID_WIDTH - 2 => x += 1,
                    2 if y > 1 => y -= 1,
                    3 if y < GRID_HEIGHT - 2 => y += 1,
                    4 if x > 1 && y > 1 => {
                        x -= 1;
                        y -= 1;
                    }
                    5 if x < GRID_WIDTH - 2 && y > 1 => {
                        x += 1;
                        y -= 1;
                    }
                    6 if x > 1 && y < GRID_HEIGHT - 2 => {
                        x -= 1;
                        y += 1;
                    }
                    7 if x < GRID_WIDTH - 2 && y < GRID_HEIGHT - 2 => {
                        x += 1;
                        y += 1;
                    }
                    _ => {}
                }
            }
        }

        start_positions.sort_by_key(|pos| pos.x + pos.y);

        for i in 1..start_positions.len() {
            let prev = start_positions[i - 1];
            let current = start_positions[i];

            //carve_path_between(&mut tiles, prev, current, &mut walkable_cache);
            Self::carve_jagged_path(&mut tiles, prev, current, &mut walkable_cache, &mut rng, 1);
        }

        // Place player at one of the center-most islands
        // let player_pos = *start_positions.first().unwrap_or(&Position { x: GRID_WIDTH / 2, y: GRID_HEIGHT / 2 });
        // let mut player = player;
        // player.set_pos(player_pos);
        // tiles[player_pos.x][player_pos.y].kind = TileKind::Floor;
        // tiles[player_pos.x][player_pos.y].creature = PLAYER_CREATURE_ID;

        let mut available_walkable_cache = walkable_cache.clone();
        available_walkable_cache.shuffle(&mut rng);

        let map = Map::new(tiles, walkable_cache, available_walkable_cache);

        map
    }
}

impl Drop for MapGenerator {
    fn drop(&mut self) {
        self.stop();
    }
}