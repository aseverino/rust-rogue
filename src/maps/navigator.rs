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

use pathfinding::prelude::astar;
use std::collections::HashSet;

use crate::{
    maps::{GRID_HEIGHT, GRID_WIDTH},
    position::Position,
    tile::TileKind,
    tile_map::TileMap,
};

pub struct Navigator {}

impl Navigator {
    pub fn find_path(
        start_pos: Position,
        goal_pos: Position,
        is_walkable: impl Fn(Position) -> bool,
    ) -> Option<Vec<Position>> {
        let start = (start_pos.x, start_pos.y);
        let goal = (goal_pos.x, goal_pos.y);
        let path = astar(
            &start,
            |&(x, y)| {
                let mut neighbors = Vec::new();
                let directions = [
                    (0isize, -1),
                    (1, 0),
                    (0, 1),
                    (-1, 0), // cardinal
                    (-1, -1),
                    (1, -1),
                    (1, 1),
                    (-1, 1), // diagonals
                ];

                for (dx, dy) in directions {
                    let nx = x as isize + dx;
                    let ny = y as isize + dy;
                    if nx >= 0 && ny >= 0 {
                        let (ux, uy) = (nx as usize, ny as usize);
                        if is_walkable(Position { x: ux, y: uy }) {
                            // Diagonal steps have slightly higher cost
                            let cost = if dx != 0 && dy != 0 { 14 } else { 10 };
                            neighbors.push(((ux, uy), cost));
                        }
                    }
                }

                neighbors
            },
            |&(x, y)| {
                let dx = (goal.0 as isize - x as isize).abs();
                let dy = (goal.1 as isize - y as isize).abs();
                10 * (dx + dy) - 6 * dx.min(dy) // diagonal heuristic
            },
            |&pos| pos == goal,
        )
        .map(|(path, _)| path);

        path.map(|p| p.into_iter().map(|(x, y)| Position { x, y }).collect())
    }

    fn is_opaque(tiles: &TileMap, x: isize, y: isize) -> bool {
        if x < 0 || y < 0 || x as usize >= GRID_WIDTH || y as usize >= GRID_HEIGHT {
            true // treat out-of-bounds as walls
        } else {
            matches!(
                tiles[Position::new(x as usize, y as usize)].kind(),
                TileKind::Wall
            )
        }
    }

    fn cast_light(
        tiles: &TileMap,
        visible: &mut HashSet<Position>,
        cx: isize,
        cy: isize,
        row: isize,
        mut start_slope: f64,
        end_slope: f64,
        radius: isize,
        octant: u8,
    ) {
        if start_slope < end_slope {
            return;
        }

        let mut next_start_slope = start_slope;

        for i in row..=radius {
            let mut blocked = false;
            let mut dx = -i;
            while dx <= 0 {
                let dy = -i;
                let (nx, ny) = match octant {
                    0 => (cx + dx, cy + dy),
                    1 => (cx + dy, cy + dx),
                    2 => (cx + dy, cy - dx),
                    3 => (cx + dx, cy - dy),
                    4 => (cx - dx, cy - dy),
                    5 => (cx - dy, cy - dx),
                    6 => (cx - dy, cy + dx),
                    7 => (cx - dx, cy + dy),
                    _ => (cx, cy),
                };

                let l_slope = (dx as f64 - 0.5) / (dy as f64 + 0.5);
                let r_slope = (dx as f64 + 0.5) / (dy as f64 - 0.5);

                if r_slope > start_slope {
                    dx += 1;
                    continue;
                } else if l_slope < end_slope {
                    break;
                }

                let distance = dx * dx + dy * dy;
                if nx >= 0 && ny >= 0 && nx < GRID_WIDTH as isize && ny < GRID_HEIGHT as isize {
                    let is_blocking = Self::is_opaque(tiles, nx, ny);

                    // Only insert tiles that are not opaque
                    if !is_blocking && distance <= radius * radius {
                        visible.insert(Position {
                            x: nx as usize,
                            y: ny as usize,
                        });
                    }
                }

                if blocked {
                    if Self::is_opaque(tiles, nx, ny) {
                        next_start_slope = r_slope;
                        dx += 1;
                        continue;
                    } else {
                        blocked = false;
                        start_slope = next_start_slope;
                    }
                } else {
                    if Self::is_opaque(tiles, nx, ny) {
                        blocked = true;
                        Self::cast_light(
                            tiles,
                            visible,
                            cx,
                            cy,
                            i + 1,
                            next_start_slope,
                            l_slope,
                            radius,
                            octant,
                        );
                        next_start_slope = r_slope;
                    }
                }

                dx += 1;
            }

            if blocked {
                break;
            }
        }
    }

    pub fn compute_fov(tiles: &TileMap, origin: Position, max_radius: usize) -> HashSet<Position> {
        let mut visible = HashSet::new();
        visible.insert(origin); // Always see self

        for octant in 0..8 {
            Self::cast_light(
                tiles,
                &mut visible,
                origin.x as isize,
                origin.y as isize,
                1,
                1.0,
                0.0,
                max_radius as isize,
                octant,
            );
        }

        visible
    }
}

pub fn find_path<F>(
    start_pos: Position,
    goal_pos: Position,
    is_walkable: F,
) -> Option<Vec<Position>>
where
    F: Fn(Position) -> bool,
{
    let start = (start_pos.x, start_pos.y);
    let goal = (goal_pos.x, goal_pos.y);
    let path = astar(
        &start,
        |&(x, y)| {
            let mut neighbors = Vec::new();
            let directions = [
                (0isize, -1),
                (1, 0),
                (0, 1),
                (-1, 0), // cardinal
                (-1, -1),
                (1, -1),
                (1, 1),
                (-1, 1), // diagonals
            ];

            for (dx, dy) in directions {
                let nx = x as isize + dx;
                let ny = y as isize + dy;
                if nx >= 0 && ny >= 0 {
                    let (ux, uy) = (nx as usize, ny as usize);
                    if is_walkable(Position { x: ux, y: uy }) {
                        // Diagonal steps have slightly higher cost
                        let cost = if dx != 0 && dy != 0 { 14 } else { 10 };
                        neighbors.push(((ux, uy), cost));
                    }
                }
            }

            neighbors
        },
        |&(x, y)| {
            let dx = (goal.0 as isize - x as isize).abs();
            let dy = (goal.1 as isize - y as isize).abs();
            10 * (dx + dy) - 6 * dx.min(dy) // diagonal heuristic
        },
        |&pos| pos == goal,
    )
    .map(|(path, _)| path);

    path.map(|p| p.into_iter().map(|(x, y)| Position { x, y }).collect())
}
