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

use std::sync::{Arc, RwLock};

use rand::{seq::SliceRandom, thread_rng};

use crate::{monster::{Monster, MonsterArc}, monster_type::MonsterType, position::Position, tile::Tile, tile_map::TileMap};

#[derive(Clone, Debug)]
pub struct GeneratedMap {
    pub tiles: TileMap,
    pub walkable_cache: Vec<Position>,
    pub available_walkable_cache: Vec<Position>,
    pub monsters: Vec<MonsterArc>,
    pub border_positions: [Vec<Position>; 4],
    pub downstair_teleport: Option<Position>,
}

impl GeneratedMap {
    pub fn new(tiles: Vec<Vec<Tile>>, walkable_cache: Vec<Position>, available_walkable_cache: Vec<Position>) -> Self {
        Self {
            tiles: TileMap::new(tiles),
            walkable_cache,
            available_walkable_cache,
            monsters: Vec::new(),
            border_positions: [
                Vec::new(), // Up border
                Vec::new(), // Right border
                Vec::new(), // Down border
                Vec::new(), // Left border
            ],
            downstair_teleport: None,
        }
    }

    pub(crate) fn add_random_monsters(
        &mut self,
        monster_types: &Vec<Arc<MonsterType>>,
        count: usize,
    ) {
        let mut rng = thread_rng();

        // 3. Pick up to `count` positions
        let len = self.available_walkable_cache.len();
        let positions: Vec<Position> = self.available_walkable_cache
            .drain(len.saturating_sub(count)..)
            .collect();

        for pos in positions {
            let kind = monster_types
                .choose(&mut rng)
                .expect("Monster type list is empty")
                .clone();

            let monster = Arc::new(RwLock::new(Monster::new(pos.clone(), kind)));
            if let Ok(monster_guard) = monster.read() {
                self.tiles[pos].creature = monster_guard.id;
            } else {
                println!("Failed to read monster data due to poisoning.");
            }
            // Wrap the monster in Rc and push to creatures
            self.monsters.push(monster);
        }

        self.walkable_cache.shuffle(&mut rng);
    }
}