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

use std::hash::{Hash, Hasher};

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
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
}