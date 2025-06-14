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
use std::{cell::RefCell, fmt, rc::{Rc, Weak}};

use macroquad::color::Color;

use crate::ui::{widget::{AnchorKind, Widget, WidgetBase, WidgetBasicConstructor}, widget_panel::WidgetPanel, widget_text::WidgetText, manager::Ui};

pub struct WidgetBar {
    pub base: WidgetBase,
    pub background: Option<Weak<RefCell<WidgetPanel>>>,
    pub foreground: Option<Weak<RefCell<WidgetPanel>>>,
    pub text: Option<Weak<RefCell<WidgetText>>>,
}

impl WidgetBar {
    pub fn draw(&self, ui: &Ui) {
        // 2) Early‐exit if the widget is invisible
        if !self.base.visible {
            return;
        }

        // Draw the background panel if it exists
        for child in &self.base.children {
            if let Some(child_widget) = child.upgrade() {
                child_widget.borrow().draw(ui);
            }
        }
    }

    pub fn set_background_color(&mut self, color: Color) {
        if let Some(bg_weak) = &self.background {
            if let Some(bg_rc) = bg_weak.upgrade() {
                bg_rc.borrow_mut().set_color(color);
            }
        }
    }

    pub fn set_bar_color(&mut self, color: Color) {
        if let Some(fg_weak) = &self.foreground {
            if let Some(fg_rc) = fg_weak.upgrade() {
                fg_rc.borrow_mut().set_color(color);
            }
        }
    }

    pub fn set_bar_percentage(&mut self, percentage: f32) {
        if let Some(fg_weak) = &self.foreground {
            if let Some(fg_rc) = fg_weak.upgrade() {
                let mut fg = fg_rc.borrow_mut();
                let width = if let Some(bg_weak) = &self.background {
                    if let Some(bg_rc) = bg_weak.upgrade() {
                        bg_rc.borrow().base.computed_quad
                            .as_ref()
                            .map_or(0.0, |quad| quad.w * percentage)
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };
                fg.base.size.w = width;
                fg.base.computed_quad.as_mut().map(|quad| {
                    quad.w = width;
                });
            }
        }
    }

    pub fn set_text(&mut self, text: &str) {
        if let Some(text_weak) = &self.text {
            if let Some(text_rc) = text_weak.upgrade() {
                text_rc.borrow_mut().set_text(&text.to_string());
            }
        }
    }
}

impl fmt::Debug for WidgetBar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WidgetBar")
            // You can add fields here if you want more detailed debug output
            .finish()
    }
}

impl WidgetBasicConstructor for WidgetBar {
    fn basic_constructor(id: u32, parent: Option<Weak<RefCell<dyn Widget>>>) -> Self {
        WidgetBar {
            base: WidgetBase::new(id, parent),
            background: None,
            foreground: None,
            text: None,
            
        }
    }
}

impl Widget for WidgetBar {
    impl_widget_fns!(WidgetBar, base);
    fn new(ui: &mut Ui, id: u32, parent: Option<Weak<RefCell<dyn Widget>>>) -> Rc<RefCell<Self>> where Self: Sized {
        let w = Self::new_default(id, parent);
        ui.add_widget(w.clone());
        
        let background = &mut ui.create_widget::<WidgetPanel>(
            Some(Rc::downgrade(&ui.widgets[id as usize])));
        w.borrow_mut().background = Some(Rc::downgrade(background));

        if let Some(bg_weak) = &w.borrow().background {
            if let Some(bg_rc) = bg_weak.upgrade() {
                //bg_rc.borrow_mut().set_color(Color::from_rgba(255, 0, 0, 255));
                bg_rc.borrow_mut().fill_parent();
            }
        }

        let foreground = &mut ui.create_widget::<WidgetPanel>(
            Some(Rc::downgrade(&ui.widgets[id as usize])));
        w.borrow_mut().foreground = Some(Rc::downgrade(foreground));

        if let Some(fg_weak) = &w.borrow().foreground {
            if let Some(fg_rc) = fg_weak.upgrade() {
                //bg_rc.borrow_mut().set_color(Color::from_rgba(255, 0, 0, 255));
                
                {
                    let mut fg = fg_rc.borrow_mut();
                    fg.add_anchor_to_parent(AnchorKind::Top, AnchorKind::Top);
                    fg.add_anchor_to_parent(AnchorKind::Left, AnchorKind::Left);
                    fg.add_anchor_to_parent(AnchorKind::Bottom, AnchorKind::Bottom);
                };
            }
        }

        let text = &mut ui.create_widget::<WidgetText>(
            Some(Rc::downgrade(&ui.widgets[id as usize])));
        w.borrow_mut().text = Some(Rc::downgrade(text));

        if let Some(text_weak) = &w.borrow().text {
            if let Some(text_rc) = text_weak.upgrade() {
                //bg_rc.borrow_mut().set_color(Color::from_rgba(255, 0, 0, 255));
                
                {
                    let mut text = text_rc.borrow_mut();
                    text.add_anchor_to_parent(AnchorKind::VerticalCenter, AnchorKind::VerticalCenter);
                    text.add_anchor_to_parent(AnchorKind::HorizontalCenter, AnchorKind::HorizontalCenter);
                };
            }
        }

        w
    }
}