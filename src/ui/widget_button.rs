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

use crate::ui::{
    manager::Ui,
    point_f::PointF,
    quad_f::QuadF,
    size_f::SizeF,
    widget::{Anchor, AnchorKind, Widget, WidgetBase, WidgetBasicConstructor},
    widget_text::WidgetText,
};

use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

pub struct WidgetButton {
    pub base: WidgetBase,
    pub text: Option<Weak<RefCell<WidgetText>>>,
    pub click_callback: Option<Box<dyn FnMut(&mut Ui, &mut WidgetButton, PointF)>>,
    pub hovered: bool,
    pub hovered_color: Color,
    pub toggled: bool,
    pub toggled_color: Color,
}

impl WidgetButton {
    pub fn draw(&self, ui: &Ui) {
        if !self.is_visible() {
            return;
        }
        let quad_opt = self.base.computed_quad;

        if let Some(drawing_coords) = quad_opt {
            if self.hovered {
                if self.hovered_color != BLANK {
                    draw_rectangle(
                        drawing_coords.x,
                        drawing_coords.y,
                        drawing_coords.w,
                        drawing_coords.h,
                        self.hovered_color,
                    );
                }
            } else if self.toggled {
                if self.toggled_color != BLANK {
                    draw_rectangle(
                        drawing_coords.x,
                        drawing_coords.y,
                        drawing_coords.w,
                        drawing_coords.h,
                        self.toggled_color,
                    );
                }
            }

            for child in &self.base.children {
                if let Some(child_widget) = child.upgrade() {
                    child_widget.borrow().draw(ui);
                }
            }
        }
    }

    pub fn set_text(&mut self, text: &String) {
        if let Some(text_weak) = &self.text {
            if let Some(text_rc) = text_weak.upgrade() {
                text_rc.borrow_mut().set_text(text);
            }
        }
    }

    // pub fn set_text(&mut self, text: &String) {
    //     if let Some(text_weak) = &self.text {
    //         if let Some(text_rc) = text_weak.upgrade() {
    //             {
    //                 let mut text_area = text_rc.borrow_mut();
    //                 text_area.set_text(text);
    //                 let offset_y = text_area.offset_y;
    //                 let text_size_h = text_area.text_size.h;
    //                 text_area.set_margin_top((offset_y - text_size_h) / 2.0);
    //             }
    //         }
    //     }
    // }
}

impl fmt::Debug for WidgetButton {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WidgetText").finish()
    }
}

impl WidgetBasicConstructor for WidgetButton {
    fn basic_constructor(id: u32, parent: Option<Weak<RefCell<dyn Widget>>>) -> Self {
        let mut w = WidgetButton {
            base: WidgetBase::new(id, parent),
            click_callback: None,
            hovered: false,
            hovered_color: Color::new(0.5, 0.5, 0.5, 1.0),
            text: None,
            toggled: false,
            toggled_color: Color::new(0.3, 0.3, 0.3, 1.0),
        };

        w.base.size = SizeF::new(100.0, 30.0);
        w
    }
}

impl Widget for WidgetButton {
    impl_widget_fns!(WidgetButton, base);

    fn new(ui: &mut Ui, id: u32, parent: Option<Weak<RefCell<dyn Widget>>>) -> Rc<RefCell<Self>>
    where
        Self: Sized,
    {
        let w = Self::new_default(id, parent);
        let w_dyn: Rc<RefCell<dyn Widget>> = w.clone();

        ui.widgets.push(w.clone());
        w.borrow_mut().set_manually_added();

        let text = &mut ui.create_widget::<WidgetText>(Some(Rc::downgrade(&w_dyn)));
        w.borrow_mut().text = Some(Rc::downgrade(text));

        if let Some(text_weak) = &w.borrow().text {
            if let Some(text) = text_weak.upgrade() {
                //bg_rc.borrow_mut().set_color(Color::from_rgba(255, 0, 0, 255));
                {
                    let mut t = text.borrow_mut();
                    t.center_parent();
                    t.base.color = WHITE;
                }
            }
        }

        w
    }

    fn on_mouse_position_update(&mut self, ui: &mut Ui, pos: PointF) {
        self.hovered = self.contains_point(ui, pos);
    }

    fn on_click(&mut self, ui: &mut Ui, pos: PointF) {
        if self.is_visible() && self.contains_point(ui, pos) {
            let mut cb_opt = self.click_callback.take();
            if let Some(ref mut cb) = cb_opt {
                cb(ui, self, pos);
            }
            self.click_callback = cb_opt;
        }
    }

    fn set_on_click(&mut self, f: Box<dyn FnMut(&mut Ui, &mut WidgetButton, PointF)>) {
        self.click_callback = Some(f);
    }
}
