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

use crate::ui::{Ui, point_f::PointF, size_f::SizeF, quad_f::QuadF, widget::{Widget, WidgetBase, Anchor, AnchorKind}};

use std::sync::{RwLock, atomic::{AtomicBool, Ordering}};

pub struct WidgetText {
    base: WidgetBase,
    
    pub is_focused: bool,
    pub is_enabled: bool,
    pub is_hovered: bool,
    pub is_visible: bool,
    pub is_clickable: bool,

    pub text: String,
    pub position: PointF,
}

impl WidgetText {
    pub fn new(id: u32, parent_id: u32) -> Self {
        WidgetText {
            base: WidgetBase::new(id, Some(parent_id)),
            is_focused: false,
            is_enabled: true,
            is_hovered: false,
            is_visible: true,
            is_clickable: true,
            text: "".to_string(),
            position: PointF::zero(),
        }
    }

    pub fn draw(&self, _ui: &Ui) {
        let quad_opt = self.base.computed_quad.read().unwrap();

        // 2) Early‐exit if the widget is invisible
        // if !self.base.visible {
        //     return;
        // }

        if let Some(drawing_coords) = *quad_opt {
            draw_text(&self.text, drawing_coords.x, drawing_coords.y, 30.0, self.base.color);
        }
    }

    pub fn set_text(&mut self, text: &String) {
        self.text = text.to_string();
        let dim = measure_text(&text, None, 30, 1.0);
        self.base.size = SizeF::new(dim.width, dim.height);
    }
}

impl fmt::Debug for WidgetText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WidgetText")
            // You can add fields here if you want more detailed debug output
            .finish()
    }
}

impl_widget!(WidgetText, base);