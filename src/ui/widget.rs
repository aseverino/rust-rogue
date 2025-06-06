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

use std::sync::{RwLock, atomic::{AtomicBool, Ordering}};
use std::{cell::RefCell, rc::Rc};
use macroquad::color::{Color, BLANK};

use crate::ui::{Ui};

use crate::ui::{point_f::PointF, size_f::SizeF, quad_f::QuadF};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum AnchorKind {
    Top,
    Left,
    Right,
    Bottom,
}

pub struct Anchor {
    pub anchor_this: AnchorKind,
    pub anchor_to: AnchorKind,
    pub anchor_widget_id: u32, // ID of the widget to anchor to
}

pub struct WidgetBase {
    pub id: u32,
    pub parent_id: Option<u32>,
    pub children: Vec<u32>,
    pub anchors: Vec<Anchor>,
    pub position: PointF,
    pub size: SizeF,
    pub margin: QuadF,
    pub computed_quad: RwLock<Option<QuadF>>,
    pub dirty: AtomicBool,
    pub visible: bool,
    pub color: Color,
}

impl WidgetBase {
    pub fn new(id: u32, parent_id: Option<u32>) -> Self {
        WidgetBase {
            id,
            parent_id,
            children: Vec::new(),
            anchors: Vec::new(),
            position: PointF::zero(),
            size: SizeF::zero(),
            margin: QuadF::zero(),
            computed_quad: RwLock::new(None),
            dirty: AtomicBool::new(true),
            visible: true,
            color: BLANK,
        }
    }
}

pub trait Widget: Send + Sync {
    fn parent(&self) -> Option<u32>;
    fn children(&self) -> &[u32];
    fn children_mut(&mut self) -> &mut Vec<u32>;
    fn draw(&self, ui: &Ui);
    // fn update(&mut self);
    // fn handle_input(&mut self);
    // fn is_visible(&self) -> bool;
    // fn set_visible(&mut self, visible: bool);
    fn get_position(&self) -> PointF;
    fn set_position(&mut self, position: PointF);
    fn set_color(&mut self, color: Color);
    fn get_size(&self) -> SizeF;
    fn set_size(&mut self, size: SizeF);

    fn get_top(&self, ui: &Ui) -> f32;
    fn get_left(&self, ui: &Ui) -> f32;
    fn get_right(&self, ui: &Ui) -> f32;
    fn get_bottom(&self, ui: &Ui) -> f32;

    fn get_margin_top(&self) -> f32;
    fn get_margin_left(&self) -> f32;
    fn get_margin_right(&self) -> f32;
    fn get_margin_bottom(&self) -> f32;
    fn get_margin(&self) -> QuadF;

    fn set_margin_top(&mut self, margin: f32);
    fn set_margin_left(&mut self, margin: f32);
    fn set_margin_right(&mut self, margin: f32);
    fn set_margin_bottom(&mut self, margin: f32);
    fn set_margin(&mut self, margin: QuadF);

    fn get_coords(&self) -> QuadF;

    fn set_visible(&mut self, visible: bool);
    fn is_visible(&self) -> bool;

    fn get_id(&self) -> u32;
    fn add_anchor(&mut self, this: AnchorKind, other_id: u32, other_side: AnchorKind);
    fn add_anchor_to_parent(&mut self, this: AnchorKind, other_side: AnchorKind);
    fn get_drawing_coords(&self, ui: &Ui) -> QuadF;
    fn recompute_quad(&self, ui: &Ui) -> QuadF;
}

// fn resolve_anchor_position<T: Widget>(anchor_kind: AnchorKind, widget: T) -> f32 {
//     match anchor_kind {
//         AnchorKind::Top => widget.get_top(),
//         AnchorKind::Left => widget.get_left(),
//         AnchorKind::Right => widget.get_right(),
//         AnchorKind::Bottom => widget.get_bottom(),
//     }
// }

#[macro_export]
macro_rules! impl_widget {
    ($t:ty, $base:ident) => {
        impl Widget for $t {
            fn parent(&self) -> Option<u32> {
                self.$base.parent_id
            }

            fn children(&self) -> &[u32] {
                &self.$base.children
            }

            fn children_mut(&mut self) -> &mut Vec<u32> {
                &mut self.$base.children
            }

            fn draw(&self, ui: &Ui) {
                self.draw(ui);
            }

            fn get_position(&self) -> PointF {
                self.$base.position
            }

            fn get_size(&self) -> SizeF {
                self.$base.size
            }

            fn get_left(&self, ui: &Ui) -> f32 {
                let q = self.get_drawing_coords(ui);
                q.x
            }
            fn get_right(&self, ui: &Ui) -> f32 {
                let q = self.get_drawing_coords(ui);
                q.x + q.w
            }
            fn get_top(&self, ui: &Ui) -> f32 {
                let q = self.get_drawing_coords(ui);
                q.y
            }
            fn get_bottom(&self, ui: &Ui) -> f32 {
                let q = self.get_drawing_coords(ui);
                q.y + q.h
            }

            fn get_margin_top(&self) -> f32 {
                self.$base.margin.y
            }
            fn get_margin_left(&self) -> f32 {
                self.$base.margin.x
            }
            fn get_margin_right(&self) -> f32 {
                self.$base.margin.x + self.$base.margin.w
            }
            fn get_margin_bottom(&self) -> f32 {
                self.$base.margin.y + self.$base.margin.h
            }
            fn get_margin(&self) -> QuadF {
                self.$base.margin
            }

            fn set_margin_top(&mut self, margin: f32) {
                self.$base.margin.y = margin;
                self.$base.dirty.store(true, Ordering::SeqCst);
            }
            fn set_margin_left(&mut self, margin: f32) {
                self.$base.margin.x = margin;
                self.$base.dirty.store(true, Ordering::SeqCst);
            }
            fn set_margin_right(&mut self, margin: f32) {
                self.$base.margin.w = margin;
                self.$base.dirty.store(true, Ordering::SeqCst);
            }
            fn set_margin_bottom(&mut self, margin: f32) {
                self.$base.margin.h = margin;
                self.$base.dirty.store(true, Ordering::SeqCst);
            }
            fn set_margin(&mut self, margin: QuadF) {
                self.$base.margin = margin;
                self.$base.dirty.store(true, Ordering::SeqCst);
            }

            fn get_coords(&self) -> QuadF {
                // Return a copy of the computed quad, or a zero quad if not set
                self.$base.computed_quad.read().unwrap().unwrap_or_else(QuadF::zero)
            }

            fn get_id(&self) -> u32 {
                self.$base.id
            }
            fn set_visible(&mut self, visible: bool) {
                self.$base.visible = visible;
            }

            fn is_visible(&self) -> bool {
                self.$base.visible
            }

            fn add_anchor(&mut self, this: AnchorKind, other_id: u32, other_side: AnchorKind) {
                self.$base.anchors.push(Anchor {
                    anchor_this: this,
                    anchor_widget_id: other_id,
                    anchor_to: other_side,
                });
                self.$base.dirty.store(true, Ordering::SeqCst);
            }

            fn add_anchor_to_parent(&mut self, this: AnchorKind, other_side: AnchorKind) {
                self.$base.anchors.push(Anchor {
                    anchor_this: this,
                    anchor_widget_id: self.$base.parent_id.unwrap_or(0),
                    anchor_to: other_side,
                });
                self.$base.dirty.store(true, Ordering::SeqCst);
            }

            fn set_size(&mut self, sz: SizeF) {
                self.$base.size = sz;
                self.$base.dirty.store(true, Ordering::SeqCst);
            }
            fn set_position(&mut self, pos: PointF) {
                self.$base.position = pos;
                self.$base.dirty.store(true, Ordering::SeqCst);
            }
            fn set_color(&mut self, color: Color) {
                self.$base.color = color;
            }

            fn get_drawing_coords(&self, ui: &Ui) -> QuadF {
                if self.$base.dirty.load(Ordering::SeqCst) {
                    let quad = self.recompute_quad(ui);
                    let mut lock = self.$base.computed_quad.write().unwrap();
                    *lock = Some(quad);
                    self.$base.dirty.store(false, Ordering::SeqCst);
                    quad
                } else {
                    // Already computed_QuadF must be Some(_).
                    self.$base.computed_quad.read().unwrap().unwrap()
                }
            }

            fn recompute_quad(&self, ui: &Ui) -> QuadF {
                let mut quad = QuadF::zero();

                // Gather any anchored sides
                let mut left = None;
                let mut right = None;
                let mut top = None;
                let mut bottom = None;

                for anchor in &self.base.anchors {
                    let anchor_widget = &ui.widgets[anchor.anchor_widget_id as usize];
                    let anchor_pos = match anchor.anchor_to {
                        AnchorKind::Left => anchor_widget.get_left(ui),
                        AnchorKind::Right => anchor_widget.get_right(ui),
                        AnchorKind::Top => anchor_widget.get_top(ui),
                        AnchorKind::Bottom => anchor_widget.get_bottom(ui),
                    };

                    match anchor.anchor_this {
                        AnchorKind::Left => left = Some(anchor_pos),
                        AnchorKind::Right => right = Some(anchor_pos),
                        AnchorKind::Top => top = Some(anchor_pos),
                        AnchorKind::Bottom => bottom = Some(anchor_pos),
                        _ => {}
                    }
                }

                // Horizontal logic
                if let (Some(lv), Some(rv)) = (left, right) {
                    quad.x = lv;
                    quad.w = rv - lv;
                } else if let Some(lv) = left {
                    quad.x = lv;
                    quad.w = self.$base.size.w;
                } else if let Some(rv) = right {
                    quad.w = self.$base.size.w;
                    quad.x = rv - quad.w;
                } else {
                    quad.x = self.$base.position.x;
                    quad.w = self.$base.size.w;
                }

                // Vertical logic
                if let (Some(tv), Some(bv)) = (top, bottom) {
                    quad.y = tv;
                    quad.h = bv - tv;
                } else if let Some(tv) = top {
                    quad.y = tv;
                    quad.h = self.$base.size.h;
                } else if let Some(bv) = bottom {
                    quad.h = self.$base.size.h;
                    quad.y = bv - quad.h;
                } else {
                    quad.y = self.$base.position.y;
                    quad.h = self.$base.size.h;
                }

                quad.x += self.$base.margin.x;
                quad.y += self.$base.margin.y;
                quad
            }
        }
    };
}

// #[macro_export]
// macro_rules! impl_widget {
//     ($t:ty, $base:ident, $position_getter:expr, $position_setter:expr, $draw_expr:expr) => {
//         impl Widget for $t {
//             fn parent(&self) -> Option<Rc<RefCell<dyn Widget>>> {
//                 self.$base.parent.as_ref().map(Rc::clone)
//             }

//             fn children(&self) -> &[Rc<RefCell<dyn Widget>>] {
//                 &self.$base.children
//             }

//             fn children_mut(&mut self) -> &mut Vec<Rc<RefCell<dyn Widget>>> {
//                 &mut self.$base.children
//             }

//             fn draw(&self, offset: (f32, f32)) {
//                 ($draw_expr)(self, offset);
//             }

//             fn get_position(&self) -> (f32, f32) {
//                 ($position_getter)(self)
//             }

//             fn set_position(&mut self, position: (f32, f32)) {
//                 ($position_setter)(self, position)
//             }
//         }
//     };
// }