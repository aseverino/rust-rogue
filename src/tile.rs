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

use macroquad::prelude::*;

use crate::{
    items::{base_item::ItemKind, orb::Orb, teleport::Teleport},
    maps::{GRID_HEIGHT, GRID_WIDTH, TILE_SIZE},
    position::Position,
    ui::point_f::PointF,
};

pub const NO_CREATURE: u32 = 0;
pub const PLAYER_CREATURE_ID: u32 = u32::MAX; // or any large unique value

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum TileKind {
    Chasm,
    Wall,
    Floor,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct EdgeKind: u16 {
        const NONE         = 0b0000_0000_0000;
        const TOP          = 0b0000_0000_0001;
        const RIGHT        = 0b0000_0000_0010;
        const BOTTOM       = 0b0000_0000_0100;
        const LEFT         = 0b0000_0000_1000;
        const TOP_LEFT     = 0b0000_0001_0000;
        const TOP_RIGHT    = 0b0000_0010_0000;
        const BOTTOM_LEFT  = 0b0000_0100_0000;
        const BOTTOM_RIGHT = 0b0000_1000_0000;
    }
}

pub struct TileFactory {
    chasm: Tile,
    floor: Tile,
    wall: Tile,
}

impl TileFactory {
    pub fn new() -> Self {
        Self {
            chasm: Tile::new(TileKind::Chasm),
            floor: Tile::new(TileKind::Floor),
            wall: Tile::new(TileKind::Wall),
        }
    }
    pub async fn init(&mut self) {
        {
            let texture = load_texture("assets/sprites/scenario/chasm.png")
                .await
                .unwrap();
            texture.set_filter(FilterMode::Nearest);
            self.chasm.sprite = Some(Arc::new(RwLock::new(texture)));
        }
        {
            let texture_result = load_texture("assets/sprites/scenario/floor.png").await;
            let mut texture = match texture_result {
                Ok(tex) => tex,
                Err(e) => {
                    eprintln!("Failed to load floor texture: {}", e);
                    return;
                }
            };

            texture.set_filter(FilterMode::Nearest);
            self.floor.sprite = Some(Arc::new(RwLock::new(texture)));
        }
    }
    pub fn create_tile(&self, kind: TileKind) -> Tile {
        match kind {
            TileKind::Chasm => self.chasm.clone(),
            TileKind::Wall => self.wall.clone(),
            TileKind::Floor => self.floor.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Tile {
    kind: TileKind,
    pub edge: EdgeKind,
    pub creature: u32,
    pub items: Vec<ItemKind>,
    pub sprite: Option<Arc<RwLock<Texture2D>>>,
}

impl Tile {
    pub(in crate::tile) fn new(kind: TileKind) -> Self {
        Self {
            kind,
            creature: NO_CREATURE,
            items: Vec::new(),
            edge: EdgeKind::NONE,
            sprite: None,
        }
    }

    pub fn kind(&self) -> TileKind {
        self.kind
    }

    pub fn has_enemy(&self) -> bool {
        self.creature != NO_CREATURE && self.creature != PLAYER_CREATURE_ID
    }

    pub fn is_walkable(&self) -> bool {
        self.kind == TileKind::Floor
            && (self.creature == NO_CREATURE || self.creature == PLAYER_CREATURE_ID)
    }

    pub fn is_blocking(&self) -> bool {
        self.kind == TileKind::Wall
            || (self.creature != NO_CREATURE && self.creature != PLAYER_CREATURE_ID)
    }

    pub fn is_solid_blocking(&self) -> bool {
        self.kind == TileKind::Wall
    }

    pub fn has_container(&self) -> bool {
        self.items
            .iter()
            .any(|item| matches!(item, ItemKind::Container(_)))
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

    fn has_edge(&self, kind: EdgeKind) -> bool {
        self.edge.contains(kind)
    }

    pub fn remove_item(&mut self, index: usize) -> Option<ItemKind> {
        if index < self.items.len() {
            Some(self.items.remove(index))
        } else {
            None
        }
    }

    pub fn is_border(&self, pos: &Position) -> bool {
        self.kind == TileKind::Floor
            && (pos.x == 0 || pos.y == 0 || pos.x == GRID_WIDTH - 1 || pos.y == GRID_HEIGHT - 1)
    }

    pub fn draw_edges(&self, pos: Position, offset: PointF) {
        const TILE_PX: f32 = 16.0;
        let mut px = 0.0 * TILE_PX;
        let mut py = 1.0 * TILE_PX;
        let dest = Vec2::new(32.0, 32.0);
        let tex = &self.sprite.as_ref().unwrap().read().unwrap();
        let mut drawn = EdgeKind::NONE;

        if self.has_edge(EdgeKind::TOP) && self.has_edge(EdgeKind::RIGHT) {
            let mut extra = PointF::new(0.0, 0.0);
            let mut size = dest.clone();
            let mut src = Rect {
                x: px,
                y: py + TILE_PX,
                w: TILE_PX,
                h: -TILE_PX,
            };

            if self.has_edge(EdgeKind::BOTTOM) && self.has_edge(EdgeKind::RIGHT) {
                src.h += 8.0;
                size.y -= 16.0;
                extra.y -= 8.0;
            }

            if self.has_edge(EdgeKind::TOP) && self.has_edge(EdgeKind::LEFT) {
                src.x += 8.0;
                extra.x += 16.0;
            }

            let params = DrawTextureParams {
                dest_size: Some(size),
                source: Some(src),
                ..Default::default()
            };

            let base_x = offset.x + pos.x as f32 * TILE_SIZE + (TILE_SIZE - size.x) / 2.0;
            let base_y = offset.y + pos.y as f32 * TILE_SIZE + (TILE_SIZE - size.y) / 2.0;

            draw_texture_ex(tex, base_x + extra.x, base_y + extra.y, WHITE, params);
            drawn |= EdgeKind::TOP | EdgeKind::RIGHT;
        }
        if self.has_edge(EdgeKind::TOP) && self.has_edge(EdgeKind::LEFT) {
            let mut extra = PointF::new(0.0, 0.0);
            let mut size = dest.clone();
            let mut src = Rect {
                x: px + TILE_PX,
                y: py + TILE_PX,
                w: -TILE_PX,
                h: -TILE_PX,
            };

            if self.has_edge(EdgeKind::BOTTOM) && self.has_edge(EdgeKind::LEFT) {
                src.h += 8.0; // -16 + 8 = -8
                size.y -= 16.0; // 32 → 16
                extra.y -= 8.0; // move up a bit
            }
            if self.has_edge(EdgeKind::TOP) && self.has_edge(EdgeKind::RIGHT) {
                src.w += 8.0; // -16 + 8 = -8
                size.x -= 16.0;
                extra.x -= 8.0;
            }

            let params = DrawTextureParams {
                dest_size: Some(size),
                source: Some(src),
                ..Default::default()
            };

            let base_x = offset.x + pos.x as f32 * TILE_SIZE + (TILE_SIZE - size.x) / 2.0;
            let base_y = offset.y + pos.y as f32 * TILE_SIZE + (TILE_SIZE - size.y) / 2.0;

            draw_texture_ex(tex, base_x + extra.x, base_y + extra.y, WHITE, params);
            drawn |= EdgeKind::TOP | EdgeKind::LEFT;
        }
        if self.has_edge(EdgeKind::BOTTOM) && self.has_edge(EdgeKind::RIGHT) {
            let mut extra = PointF::new(0.0, 0.0);
            let mut size = dest.clone();
            let mut src = Rect {
                x: px,
                y: py,
                w: TILE_PX,
                h: TILE_PX,
            };

            if self.has_edge(EdgeKind::TOP) && self.has_edge(EdgeKind::RIGHT) {
                src.y += 8.0;
                src.h -= 8.0;
                size.y -= 16.0;
                extra.y += 8.0;
            }
            if self.has_edge(EdgeKind::BOTTOM) && self.has_edge(EdgeKind::LEFT) {
                src.x += 8.0;
                src.w -= 8.0;
                size.x -= 16.0;
                extra.x += 8.0;
            }

            let params = DrawTextureParams {
                dest_size: Some(size),
                source: Some(src),
                ..Default::default()
            };

            let base_x = offset.x + pos.x as f32 * TILE_SIZE + (TILE_SIZE - size.x) / 2.0;
            let base_y = offset.y + pos.y as f32 * TILE_SIZE + (TILE_SIZE - size.y) / 2.0;

            draw_texture_ex(tex, base_x + extra.x, base_y + extra.y, WHITE, params);
            drawn |= EdgeKind::BOTTOM | EdgeKind::RIGHT;
        }
        if self.has_edge(EdgeKind::BOTTOM) && self.has_edge(EdgeKind::LEFT) {
            let mut extra = PointF::new(0.0, 0.0);
            let mut size = dest.clone();
            let mut src = Rect {
                x: px + TILE_PX,
                y: py,
                w: -TILE_PX,
                h: TILE_PX,
            };

            if self.has_edge(EdgeKind::TOP) && self.has_edge(EdgeKind::LEFT) {
                src.y += 8.0;
                src.h -= 8.0;
                size.y -= 16.0;
                extra.y += 8.0;
            }
            if self.has_edge(EdgeKind::BOTTOM) && self.has_edge(EdgeKind::RIGHT) {
                //src.x -= 8.0;
                src.w += 8.0;
                size.x -= 16.0;
                extra.x -= 8.0;
            }

            let params = DrawTextureParams {
                dest_size: Some(size),
                source: Some(src),
                ..Default::default()
            };

            let base_x = offset.x + pos.x as f32 * TILE_SIZE + (TILE_SIZE - size.x) / 2.0;
            let base_y = offset.y + pos.y as f32 * TILE_SIZE + (TILE_SIZE - size.y) / 2.0;

            draw_texture_ex(tex, base_x + extra.x, base_y + extra.y, WHITE, params);
            drawn |= EdgeKind::BOTTOM | EdgeKind::LEFT;
        }

        px = 1.0 * TILE_PX;
        py = 0.0 * TILE_PX;

        if self.has_edge(EdgeKind::TOP) && !drawn.contains(EdgeKind::TOP) {
            let extra = PointF::new(0.0, 0.0);
            let size = dest.clone();
            let src = Rect {
                x: px,
                y: py + TILE_PX,
                w: TILE_PX,
                h: -TILE_PX,
            };

            let params = DrawTextureParams {
                dest_size: Some(size),
                source: Some(src),
                ..Default::default()
            };

            let base_x = offset.x + pos.x as f32 * TILE_SIZE + (TILE_SIZE - size.x) / 2.0;
            let base_y = offset.y + pos.y as f32 * TILE_SIZE + (TILE_SIZE - size.y) / 2.0;

            draw_texture_ex(tex, base_x + extra.x, base_y + extra.y, WHITE, params);
        }

        if self.has_edge(EdgeKind::BOTTOM) && !drawn.contains(EdgeKind::BOTTOM) {
            let extra = PointF::new(0.0, 0.0);
            let size = dest.clone();
            let src = Rect {
                x: px,
                y: py,
                w: TILE_PX,
                h: TILE_PX,
            };

            let params = DrawTextureParams {
                dest_size: Some(size),
                source: Some(src),
                ..Default::default()
            };

            let base_x = offset.x + pos.x as f32 * TILE_SIZE + (TILE_SIZE - size.x) / 2.0;
            let base_y = offset.y + pos.y as f32 * TILE_SIZE + (TILE_SIZE - size.y) / 2.0;

            draw_texture_ex(tex, base_x + extra.x, base_y + extra.y, WHITE, params);
        }

        px = 1.0 * TILE_PX;
        py = 1.0 * TILE_PX;

        if self.has_edge(EdgeKind::RIGHT) && !drawn.contains(EdgeKind::RIGHT) {
            let extra = PointF::new(0.0, 0.0);
            let size = dest.clone();
            let src = Rect {
                x: px,
                y: py,
                w: TILE_PX,
                h: TILE_PX,
            };

            let params = DrawTextureParams {
                dest_size: Some(size),
                source: Some(src),
                ..Default::default()
            };

            let base_x = offset.x + pos.x as f32 * TILE_SIZE + (TILE_SIZE - size.x) / 2.0;
            let base_y = offset.y + pos.y as f32 * TILE_SIZE + (TILE_SIZE - size.y) / 2.0;

            draw_texture_ex(tex, base_x + extra.x, base_y + extra.y, WHITE, params);
        }

        if self.has_edge(EdgeKind::LEFT) && !drawn.contains(EdgeKind::LEFT) {
            let extra = PointF::new(0.0, 0.0);
            let size = dest.clone();
            let src = Rect {
                x: px + TILE_PX,
                y: py,
                w: -TILE_PX,
                h: TILE_PX,
            };

            let params = DrawTextureParams {
                dest_size: Some(size),
                source: Some(src),
                ..Default::default()
            };

            let base_x = offset.x + pos.x as f32 * TILE_SIZE + (TILE_SIZE - size.x) / 2.0;
            let base_y = offset.y + pos.y as f32 * TILE_SIZE + (TILE_SIZE - size.y) / 2.0;

            draw_texture_ex(tex, base_x + extra.x, base_y + extra.y, WHITE, params);
        }
    }

    pub fn draw(
        &self,
        pos: Position,
        offset: PointF,
        borders_locked: bool,
        animating_effect: Option<&Arc<RwLock<Texture2D>>>,
        animate_for: f32,
    ) {
        let color = match self.kind {
            TileKind::Floor => Color {
                r: 0.5,
                g: 0.5,
                b: 0.5,
                a: 1.0,
            },
            TileKind::Wall => Color {
                r: 0.3,
                g: 0.3,
                b: 0.3,
                a: 1.0,
            },
            TileKind::Chasm => Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
        };

        if self.creature == NO_CREATURE && self.items.is_empty() {
            if self.sprite.is_some() {
                if self.kind == TileKind::Chasm {
                    self.draw_edges(pos, offset);
                } else if self.kind == TileKind::Floor {
                    if let Some(sprite_arc) = &self.sprite {
                        let sprite = sprite_arc.read().unwrap();
                        let draw_params = DrawTextureParams {
                            dest_size: Some(Vec2::new(32.0, 32.0)),
                            source: Some(Rect {
                                x: 0.0,
                                y: 0.0,
                                w: 16.0,
                                h: 16.0,
                            }),
                            ..Default::default()
                        };

                        let x = offset.x + pos.x as f32 * TILE_SIZE;
                        let y = offset.y + pos.y as f32 * TILE_SIZE;

                        draw_texture_ex(&sprite, x, y, WHITE, draw_params);
                    }
                }
            } else {
                draw_rectangle(
                    offset.x + pos.x as f32 * TILE_SIZE,
                    offset.y + pos.y as f32 * TILE_SIZE,
                    TILE_SIZE - 1.0,
                    TILE_SIZE - 1.0,
                    color,
                );
            }

            if self.is_border(&pos) {
                // Draw border
                let border_color = if borders_locked {
                    Color {
                        r: 0.8,
                        g: 0.2,
                        b: 0.2,
                        a: 1.0,
                    } // Red for locked borders
                } else {
                    Color {
                        r: 0.2,
                        g: 0.8,
                        b: 0.2,
                        a: 1.0,
                    } // Green for unlocked borders
                };
                draw_rectangle(
                    offset.x + pos.x as f32 * TILE_SIZE,
                    offset.y + pos.y as f32 * TILE_SIZE,
                    TILE_SIZE - 1.0,
                    TILE_SIZE - 1.0,
                    border_color,
                );
            }
        }

        for item in &self.items {
            match item {
                ItemKind::Orb(_) => {
                    draw_circle(
                        offset.x + pos.x as f32 * TILE_SIZE + TILE_SIZE / 2.0,
                        offset.y + pos.y as f32 * TILE_SIZE + TILE_SIZE / 2.0,
                        TILE_SIZE / 4.0,
                        Color {
                            r: 0.0,
                            g: 0.0,
                            b: 1.0,
                            a: 1.0,
                        },
                    );
                }
                ItemKind::Teleport(_) => {
                    let teleport_color = if borders_locked {
                        Color {
                            r: 0.8,
                            g: 0.2,
                            b: 0.2,
                            a: 1.0,
                        } // Red for locked
                    } else {
                        Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        } // Black for open
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
                        Color {
                            r: 1.0,
                            g: 1.0,
                            b: 0.0,
                            a: 1.0,
                        },
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

        if let Some(effect) = animating_effect {
            let texture = effect.read().unwrap();
            let frame = ((0.25 - animate_for) * 12.0).floor() as usize;
            let draw_params = DrawTextureParams {
                dest_size: Some(Vec2::new(32.0, 32.0)),
                source: Some(Rect {
                    x: 0.0,
                    y: frame as f32 * 16.0,
                    w: 16.0,
                    h: 16.0,
                }),
                ..Default::default()
            };

            let x = offset.x + pos.x as f32 * TILE_SIZE;
            let y = offset.y + pos.y as f32 * TILE_SIZE;

            draw_texture_ex(&texture, x, y, WHITE, draw_params);
        }
    }
}
