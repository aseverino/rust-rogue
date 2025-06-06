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

use std::fmt;

use macroquad::prelude::*;

use std::sync::{RwLock, atomic::{AtomicBool, Ordering}};

use crate::ui::{Ui, point_f::PointF, size_f::SizeF, quad_f::QuadF, widget::{Widget, WidgetBase, Anchor, AnchorKind}};

pub struct WidgetPanel {
    pub base: WidgetBase,
    pub border_color: Option<Color>,
    pub border_thickness: Option<f32>,
}

impl WidgetPanel {
    pub fn new(id: u32, parent_id: Option<u32>) -> Self {
        WidgetPanel {
            base: WidgetBase::new(id, parent_id),
            border_color: None,
            border_thickness: None,
        }
    }

    pub fn draw(&self, ui: &Ui) {
        // 1) Acquire a read lock on computed_quad
        let quad_opt = self.base.computed_quad.read().unwrap();

        // 2) Early‐exit if the widget is invisible
        if !self.base.visible {
            return;
        }

        if let Some(drawing_coords) = *quad_opt {
            if let Some(border_color) = self.border_color {
                if let Some(border_thickness) = self.border_thickness {
                    if self.base.color != BLANK {
                        draw_rectangle(
                            drawing_coords.x,
                            drawing_coords.y,
                            drawing_coords.w,
                            drawing_coords.h,
                            self.base.color,
                        );
                    }
                    draw_rectangle_lines(
                        drawing_coords.x,
                        drawing_coords.y,
                        drawing_coords.w,
                        drawing_coords.h,
                        border_thickness,
                        border_color,
                    );
                    return;
                }
            }
            
            if self.base.color != BLANK {
                draw_rectangle(
                    drawing_coords.x,
                    drawing_coords.y,
                    drawing_coords.w,
                    drawing_coords.h,
                    self.base.color,
                );
            }
        }
    }

    pub fn set_border(&mut self, color: Color, thickness: f32) {
        self.border_color = Some(color);
        self.border_thickness = Some(thickness);
    }
}

impl fmt::Debug for WidgetPanel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WidgetPanel")
            // You can add fields here if you want more detailed debug output
            .finish()
    }
}

impl_widget!(WidgetPanel, base);