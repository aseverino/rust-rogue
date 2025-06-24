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

use std::hash::Hash;

#[derive(Clone)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
    UpRight,
    DownRight,
    DownLeft,
    UpLeft,
    None,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    pub fn is_valid(&self, width: usize, height: usize) -> bool {
        self.x < width && self.y < height
    }

    pub fn distance_to(&self, other: &Position) -> usize {
        let dx = (self.x as isize - other.x as isize) as f64;
        let dy = (self.y as isize - other.y as isize) as f64;
        (dx * dx + dy * dy).sqrt().floor() as usize
    }

    pub fn in_range(&self, other: &Position, range: usize) -> bool {
        let dx = (self.x as isize - other.x as isize) as f64;
        let dy = (self.y as isize - other.y as isize) as f64;
        (dx * dx + dy * dy).sqrt() <= range as f64
    }
    pub fn north(&self) -> Option<Self> {
        if self.y == 0 {
            None
        } else {
            Some(Self {
                x: self.x,
                y: self.y - 1,
            })
        }
    }
    pub fn east(&self) -> Self {
        Self {
            x: self.x + 1,
            y: self.y,
        }
    }
    pub fn south(&self) -> Self {
        Self {
            x: self.x,
            y: self.y + 1,
        }
    }
    pub fn west(&self) -> Option<Self> {
        if self.x == 0 {
            None
        } else {
            Some(Self {
                x: self.x - 1,
                y: self.y,
            })
        }
    }
    pub fn north_east(&self) -> Option<Self> {
        if self.x == 0 {
            None
        } else {
            Some(Self {
                x: self.x - 1,
                y: self.y - 1,
            })
        }
    }
    pub fn south_east(&self) -> Self {
        Self {
            x: self.x + 1,
            y: self.y + 1,
        }
    }
    pub fn south_west(&self) -> Option<Self> {
        if self.y == 0 {
            None
        } else {
            Some(Self {
                x: self.x - 1,
                y: self.y + 1,
            })
        }
    }
    pub fn north_west(&self) -> Option<Self> {
        if self.x == 0 || self.y == 0 {
            None
        } else {
            Some(Self {
                x: self.x - 1,
                y: self.y - 1,
            })
        }
    }

    pub fn positions_around(&self) -> Vec<Position> {
        let mut positions = Vec::new();
        if let Some(north) = self.north() {
            positions.push(north);
        }
        if let Some(west) = self.west() {
            positions.push(west);
        }
        positions.push(self.east());
        positions.push(self.south());
        if let Some(south_west) = self.south_west() {
            positions.push(south_west);
        }
        if let Some(north_east) = self.north_east() {
            positions.push(north_east);
        }

        positions.push(self.south_east());

        if let Some(north_west) = self.north_west() {
            positions.push(north_west);
        }
        positions
    }
}

pub static POSITION_INVALID: Position = Position {
    x: usize::MAX,
    y: usize::MAX,
};
