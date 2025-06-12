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

use crate::ui::{point_f::PointF, quad_f::QuadF, size_f::SizeF, widget::{Anchor, AnchorKind, Widget, WidgetBase, WidgetBasicConstructor}, widget_text::WidgetText, Ui};

use std::{cell::RefCell, rc::{Weak, Rc}};

pub struct WidgetButton {
    pub base: WidgetText,
    pub click_callback: Option<Box<dyn FnMut(&mut Ui, PointF)>>
}

impl WidgetButton {
    pub fn draw(&self, _ui: &Ui) {
        let quad_opt = self.base.base.computed_quad;

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

        if let Some(drawing_coords) = self.base.base.computed_quad {
            draw_text(&self.base.text, drawing_coords.x, drawing_coords.y + self.base.text_size.h, 30.0, self.base.base.color);
        }
    }

    pub fn set_text(&mut self, text: &String) {
        self.base.text = text.to_string();
        let dim = measure_text(&text, None, 30, 1.0);
        self.base.text_size = SizeF::new(dim.width, dim.height);
        self.base.base.size = self.base.text_size;
        self.base.base.dirty = true;
    }
}

impl fmt::Debug for WidgetButton {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WidgetText").finish()
    }
}

impl WidgetBasicConstructor for WidgetButton {
    fn basic_constructor(id: u32, parent: Option<Weak<RefCell<dyn Widget>>>) -> Self {
        let mut w = WidgetButton {
            base: WidgetText {
                base: WidgetBase::new(id, parent),
                text: "".to_string(),
                text_size: SizeF::new(0.0, 0.0),
            },
            click_callback: None
        };

        w.base.base.color = WHITE;
        //w.base.size = SizeF::new(0.0, 0.0);
        //bar.background.set_parent(Some());
        //bar.background.set_color(Color::from_rgba(1, 0, 0, 1));
        //bar.background.fill_parent();
        w
    }
}

impl Widget for WidgetButton {
    impl_widget_fns!(WidgetButton, base.base);

    fn on_click(&mut self, ui: &mut Ui, pos: PointF) {
        if self.contains_point(ui, pos) {
            if let Some(cb) = self.click_callback.as_mut() {
                cb(ui, pos);
            }
        }
    }

    fn set_on_click(&mut self, f: Box<dyn FnMut(&mut Ui, PointF)>)
    {
        self.click_callback = Some(f);
    }
}

//impl_widget!(WidgetButton, base.base);