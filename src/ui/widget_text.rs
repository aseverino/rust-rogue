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

use crate::ui::{point_f::PointF, quad_f::QuadF, size_f::SizeF, widget::{Anchor, AnchorKind, Widget, WidgetBase, WidgetBasicConstructor}, Ui};

use std::{cell::RefCell, rc::{Weak, Rc}};

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
    pub fn draw(&self, _ui: &Ui) {
        if let Some(drawing_coords) = self.base.computed_quad {
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
        f.debug_struct("WidgetText").finish()
    }
}

impl WidgetBasicConstructor for WidgetText {
    fn basic_constructor(id: u32, parent: Option<Weak<RefCell<dyn Widget>>>) -> Self {
        let mut w = WidgetText {
            base: WidgetBase::new(id, parent),
            is_focused: false,
            is_enabled: true,
            is_hovered: false,
            is_visible: true,
            is_clickable: true,
            text: "".to_string(),
            position: PointF::zero(),
        };

        w.base.color = WHITE;

        w
    }
}

impl_widget!(WidgetText, base);