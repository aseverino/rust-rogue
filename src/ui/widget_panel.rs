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

use std::any::Any;
use std::fmt;

use macroquad::prelude::*;

use std::{cell::RefCell, rc::{Weak, Rc}};

use crate::ui::{point_f::PointF, quad_f::QuadF, size_f::SizeF, widget::{Anchor, AnchorKind, Widget, WidgetBase, WidgetBasicConstructor}, Ui};

pub struct WidgetPanel {
    pub base: WidgetBase,
    pub border_color: Option<Color>,
    pub border_thickness: Option<f32>,
}

impl WidgetPanel {
    pub fn draw(&self, ui: &Ui) {
        let quad_opt = self.base.computed_quad;

        // 2) Early‐exit if the widget is invisible
        if !self.base.visible {
            return;
        }

        if let Some(drawing_coords) = quad_opt {
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
                }
            }
            else if self.base.color != BLANK {
                draw_rectangle(
                    drawing_coords.x,
                    drawing_coords.y,
                    drawing_coords.w,
                    drawing_coords.h,
                    self.base.color,
                );
            }
        }
        
        for child in &self.base.children {
            if let Some(child_widget) = child.upgrade() {
                child_widget.borrow().draw(ui);
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

impl WidgetBasicConstructor for WidgetPanel {
    fn basic_constructor(id: u32, parent: Option<Weak<RefCell<dyn Widget>>>) -> Self {
        WidgetPanel {
            base: WidgetBase::new(id, parent),
            border_color: None,
            border_thickness: None,
        }
    }
}

impl_widget!(WidgetPanel, base);