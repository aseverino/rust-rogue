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

use macroquad::prelude::*;

use crate::{items::{base_item::ItemKind, orb::Orb, teleport::Teleport}, maps::{GRID_HEIGHT, GRID_WIDTH, TILE_SIZE}, position::Position, ui::point_f::PointF};

pub const NO_CREATURE: i32 = -1;
pub const PLAYER_CREATURE_ID: i32 = i32::MAX; // or any large unique value

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum TileKind {
    Chasm,
    Wall,
    Floor,
}

#[derive(Clone, Debug)]
pub struct Tile {
    pub kind: TileKind,
    pub creature: i32, // Index of creatures on this tile
    pub items: Vec<ItemKind>
}

impl Tile {
    pub fn new(kind: TileKind) -> Self {
        Self { kind, creature: NO_CREATURE, items: Vec::new() }
    }

    pub fn has_enemy(&self) -> bool {
        self.creature != NO_CREATURE && self.creature != PLAYER_CREATURE_ID
    }

    pub fn is_walkable(&self) -> bool {
        self.kind == TileKind::Floor && (self.creature == NO_CREATURE || self.creature == PLAYER_CREATURE_ID)
    }

    pub fn has_container(&self) -> bool {
        self.items.iter().any(|item| matches!(item, ItemKind::Container(_)))
    }

    pub fn get_top_item(&self) -> Option<&ItemKind> {
        self.items.last()
    }

    pub fn add_orb(&mut self) {
        let orb = Orb {};
        self.items.push(ItemKind::Orb(orb));
    }

    pub fn add_teleport(&mut self) {
        let teleport = Teleport {};
        self.items.push(ItemKind::Teleport(teleport));
    }

    pub fn remove_item(&mut self, index: usize) -> Option<ItemKind> {
        if index < self.items.len() {
            Some(self.items.remove(index))
        } else {
            None
        }
    }

    pub fn is_border(&self, pos: &Position) -> bool {
        self.kind == TileKind::Floor && (pos.x == 0 || pos.y == 0 || pos.x == GRID_WIDTH - 1 || pos.y == GRID_HEIGHT - 1)
    }

    pub fn draw(&self, pos: Position, offset: PointF, borders_locked: bool) {
        let color = match self.kind {
            TileKind::Floor => Color { r: 0.5, g: 0.5, b: 0.5, a: 1.0 },
            TileKind::Wall => Color { r: 0.3, g: 0.3, b: 0.3, a: 1.0 },
            TileKind::Chasm => Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 },
        };

        draw_rectangle(
            offset.x + pos.x as f32 * TILE_SIZE,
            offset.y + pos.y as f32 * TILE_SIZE,
            TILE_SIZE - 1.0,
            TILE_SIZE - 1.0,
            color,
        );

        if self.is_border(&pos) {
            // Draw border
            let border_color = if borders_locked {
                Color { r: 0.8, g: 0.2, b: 0.2, a: 1.0 } // Red for locked borders
            } else {
                Color { r: 0.2, g: 0.8, b: 0.2, a: 1.0 } // Green for unlocked borders
            };
            draw_rectangle(
                offset.x + pos.x as f32 * TILE_SIZE,
                offset.y + pos.y as f32 * TILE_SIZE,
                TILE_SIZE - 1.0,
                TILE_SIZE - 1.0,
                border_color
            );
        }

        for item in &self.items {
            match item {
                ItemKind::Orb(_) => {
                    draw_circle(
                        offset.x + pos.x as f32 * TILE_SIZE + TILE_SIZE / 2.0,
                        offset.y + pos.y as f32 * TILE_SIZE + TILE_SIZE / 2.0,
                        TILE_SIZE / 4.0,
                        Color { r: 0.0, g: 0.0, b: 1.0, a: 1.0 },
                    );
                }
                ItemKind::Teleport(_) => {
                    let teleport_color = if borders_locked {
                        Color { r: 0.8, g: 0.2, b: 0.2, a: 1.0 } // Red for locked
                    } else {
                        Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 } // Black for open
                    };
                    draw_circle(
                        offset.x + pos.x as f32 * TILE_SIZE + TILE_SIZE / 2.0,
                        offset.y + pos.y as f32 * TILE_SIZE + TILE_SIZE / 2.0,
                        TILE_SIZE / 4.0,
                        teleport_color,
                    );
                }
                ItemKind::Container(_) => {
                    draw_rectangle(
                        offset.x + pos.x as f32 * TILE_SIZE + TILE_SIZE / 4.0,
                        offset.y + pos.y as f32 * TILE_SIZE + TILE_SIZE / 4.0,
                        TILE_SIZE / 2.0,
                        TILE_SIZE / 2.0,
                        Color { r: 1.0, g: 1.0, b: 0.0, a: 1.0 },
                    );
                }
                // ItemKind::Portal(_) => {
                //     draw_rectangle(
                //         offset.0 + pos.x as f32 * TILE_SIZE + TILE_SIZE / 4.0,
                //         offset.1 + pos.y as f32 * TILE_SIZE + TILE_SIZE / 4.0,
                //         TILE_SIZE / 2.0,
                //         TILE_SIZE / 2.0,
                //         Color { r: 0.5, g: 0.5, b: 1.0, a: 1.0 },
                //     );
                // }
                _ => {}
            }
        }

        // if let Some(item) = &self.item {
        //     match item {
        //         ItemKind::Orb(_) => {
        //             draw_circle(
        //                 offset.0 + pos.x as f32 * TILE_SIZE + TILE_SIZE / 2.0,
        //                 offset.1 + pos.y as f32 * TILE_SIZE + TILE_SIZE / 2.0,
        //                 TILE_SIZE / 4.0,
        //                 Color { r: 0.0, g: 0.0, b: 1.0, a: 1.0 },
        //             );
        //         }
        //         _ => {

        //         }
        //         // ItemKind::Portal(_) => {
        //         //     draw_rectangle(
        //         //         offset.0 + pos.x as f32 * TILE_SIZE + TILE_SIZE / 4.0,
        //         //         offset.1 + pos.y as f32 * TILE_SIZE + TILE_SIZE / 4.0,
        //         //         TILE_SIZE / 2.0,
        //         //         TILE_SIZE / 2.0,
        //         //         Color { r: 0.5, g: 0.5, b: 1.0, a: 1.0 },
        //         //     );
        //         // }
        //         //ItemKind::Holdable(holdable) => {
        //             // Assuming Holdable has a draw method
        //             //holdable.draw(offset, pos);
        //         //}
        //     }
        // }
    }
}