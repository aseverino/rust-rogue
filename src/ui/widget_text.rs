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

use crate::ui::{point_f::PointF, quad_f::QuadF, size_f::SizeF, widget::{Anchor, AnchorKind, Widget, WidgetBase, WidgetBasicConstructor}, manager::Ui};

use std::{cell::RefCell, rc::{Weak, Rc}};

pub struct WidgetText {
    pub base: WidgetBase,
    pub text: String,
    pub text_size: SizeF,
}

impl WidgetText {
    pub fn draw(&self, _ui: &Ui) {
        let quad_opt = self.base.computed_quad;

        // if let Some(drawing_coords) = quad_opt {
        //     if self.base.color != BLANK {
        //         draw_rectangle(
        //             drawing_coords.x,
        //             drawing_coords.y,
        //             drawing_coords.w,
        //             drawing_coords.h,
        //             self.base.color,
        //         );
        //     }
        // }

        if let Some(drawing_coords) = self.base.computed_quad {
            draw_text(&self.text, drawing_coords.x, drawing_coords.y + self.text_size.h, 30.0, self.base.color);
        }
    }

    pub fn set_text(&mut self, text: &String) {
        self.text = text.to_string();
        let dim = measure_text(&text, None, 30, 1.0);
        self.text_size = SizeF::new(dim.width, dim.height);
        self.base.size = self.text_size;
        self.base.dirty = true;
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
            text: "".to_string(),
            text_size: SizeF::new(0.0, 0.0),
        };

        w.base.color = WHITE;
        //w.base.size = SizeF::new(0.0, 0.0);
        //bar.background.set_parent(Some());
        //bar.background.set_color(Color::from_rgba(1, 0, 0, 1));
        //bar.background.fill_parent();
        w
    }
}

impl_widget!(WidgetText, base);

