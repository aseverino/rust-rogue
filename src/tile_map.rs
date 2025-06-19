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

use crate::tile::Tile;
use crate::position::Position;
use std::ops::{Index, IndexMut};

#[derive(Clone, Debug)]
pub struct TileMap {
    tiles: Vec<Vec<Tile>>,
}

impl TileMap {
    pub fn new(tiles: Vec<Vec<Tile>>) -> Self {
        Self { tiles }
    }

    pub fn in_bounds(&self, pos: Position) -> bool {
        pos.x < self.tiles.len() && pos.y < self.tiles[0].len()
    }
}

impl Index<Position> for TileMap {
    type Output = Tile;

    fn index(&self, pos: Position) -> &Self::Output {
        &self.tiles[pos.x][pos.y]
    }
}

impl IndexMut<Position> for TileMap {
    fn index_mut(&mut self, pos: Position) -> &mut Self::Output {
        &mut self.tiles[pos.x][pos.y]
    }
}