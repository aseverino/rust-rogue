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
use std::fmt::Debug;

use std::{cell::RefCell, rc::{Weak, Rc}};
use macroquad::color::{Color, BLANK};

use crate::ui::{Ui};

use crate::ui::{point_f::PointF, size_f::SizeF, quad_f::QuadF};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum AnchorKind {
    Top,
    Left,
    Right,
    Bottom,
    HorizontalCenter,
    VerticalCenter,
}

#[derive(Debug)]
pub struct Anchor {
    pub anchor_this: AnchorKind,
    pub anchor_to: AnchorKind,
    pub anchor_widget_id: u32, // ID of the widget to anchor to
}

#[derive(Debug)]
pub struct WidgetBase {
    pub id: u32,
    pub parent: Option<Weak<RefCell<dyn Widget>>>,
    pub parent_id: u32,
    pub children: Vec<Weak<RefCell<dyn Widget>>>,
    pub children_ids: Vec<u32>,
    pub anchors: Vec<Anchor>,
    pub position: PointF,
    pub size: SizeF,
    pub margin: QuadF,
    pub computed_quad: Option<QuadF>,
    pub dirty: bool,
    pub visible: bool,
    pub color: Color,
}

impl WidgetBase {
    pub fn new(id: u32, parent: Option<Weak<RefCell<dyn Widget>>>) -> Self {
        let mut w = WidgetBase {
            id,
            parent: parent,
            parent_id: u32::MAX,
            children: Vec::new(),
            children_ids: Vec::new(),
            anchors: Vec::new(),
            position: PointF::zero(),
            size: SizeF::zero(),
            margin: QuadF::zero(),
            computed_quad: None,
            dirty: true,
            visible: true,
            color: BLANK,
        };

        if let Some(ref parent) = w.parent {
            if let Some(parent_rc) = parent.upgrade() {
                w.parent_id = parent_rc.borrow().get_id();
            }
        }

        w
    }
}

pub trait WidgetBasicConstructor: Debug + 'static {
    fn basic_constructor(id: u32, parent: Option<Weak<RefCell<dyn Widget>>>) -> Self where Self: Sized;
}

pub trait Widget: WidgetBasicConstructor + Any + Debug + 'static {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn new(ui: &mut Ui, id: u32, parent: Option<Weak<RefCell<dyn Widget>>>) -> Rc<RefCell<Self>> where Self: Sized {
        Self::new_default(id, parent)
    }

    fn new_default(id: u32, parent: Option<Weak<RefCell<dyn Widget>>>) -> Rc<RefCell<Self>> where Self: Sized {
        let w = Rc::new(RefCell::new(Self::basic_constructor(id, parent.clone())));

        // Cast to trait object for correct type
        let w_dyn: Rc<RefCell<dyn Widget>> = w.clone();

        if let Some(parent_weak) = parent {
            if let Some(parent_rc) = parent_weak.upgrade() {
                parent_rc.borrow_mut().add_child(Rc::downgrade(&w_dyn), id);
            }
        }

        w
    }

    fn get_base(&self) -> &WidgetBase;
    fn get_base_mut(&mut self) -> &mut WidgetBase;

    fn set_parent(&mut self, parent: Option<Weak<RefCell<dyn Widget>>>)  {
        self.get_base_mut().parent = parent.clone();
    }

    fn get_parent(&self) -> &Option<Weak<RefCell<dyn Widget>>> {
        &self.get_base().parent
    }

    fn get_parent_mut(&mut self) -> &mut Option<Weak<RefCell<dyn Widget>>> {
        &mut self.get_base_mut().parent
    }

    fn get_children(&self) -> &[Weak<RefCell<dyn Widget>>] {
        &self.get_base().children
    }

    fn get_children_ids(&self) -> &[u32] {
        &self.get_base().children_ids
    }

    fn get_children_mut(&mut self) -> &mut Vec<Weak<RefCell<dyn Widget>>> {
        &mut self.get_base_mut().children
    }

    fn add_child(&mut self, child: Weak<RefCell<dyn Widget>>, id: u32) {
        self.get_children_mut().push(child);
        self.get_base_mut().children_ids.push(id);
    }

    fn draw(&self, ui: &Ui);

    fn get_position(&self) -> PointF {
        self.get_base().position
    }

    fn get_size(&self) -> SizeF {
        self.get_base().size
    }

    fn get_left(&mut self, ui: &Ui) -> f32 {
        let q = self.get_drawing_coords(ui);
        q.x
    }
    fn get_right(&mut self, ui: &Ui) -> f32 {
        let q = self.get_drawing_coords(ui);
        q.x + q.w
    }
    fn get_top(&mut self, ui: &Ui) -> f32 {
        let q = self.get_drawing_coords(ui);
        q.y
    }
    fn get_bottom(&mut self, ui: &Ui) -> f32 {
        let q = self.get_drawing_coords(ui);
        q.y + q.h
    }

    fn get_margin_top(&self) -> f32 {
        self.get_base().margin.y
    }
    fn get_margin_left(&self) -> f32 {
        self.get_base().margin.x
    }
    fn get_margin_right(&self) -> f32 {
        self.get_base().margin.x + self.get_base().margin.w
    }
    fn get_margin_bottom(&self) -> f32 {
        self.get_base().margin.y + self.get_base().margin.h
    }
    fn get_margin(&self) -> QuadF {
        self.get_base().margin
    }

    fn set_margin_top(&mut self, margin: f32) {
        self.get_base_mut().margin.y = margin;
        self.get_base_mut().dirty = true;
    }
    fn set_margin_left(&mut self, margin: f32) {
        self.get_base_mut().margin.x = margin;
        self.get_base_mut().dirty = true;
    }
    fn set_margin_right(&mut self, margin: f32) {
        self.get_base_mut().margin.w = margin;
        self.get_base_mut().dirty = true;
    }
    fn set_margin_bottom(&mut self, margin: f32) {
        self.get_base_mut().margin.h = margin;
        self.get_base_mut().dirty = true;
    }
    fn set_margin(&mut self, margin: QuadF) {
        self.get_base_mut().margin = margin;
        self.get_base_mut().dirty = true;
    }

    fn get_coords(&self) -> QuadF {
        // Return a copy of the computed quad, or a zero quad if not set
        self.get_base().computed_quad.unwrap_or_else(QuadF::zero)
    }

    fn get_id(&self) -> u32 {
        self.get_base().id
    }
    fn set_visible(&mut self, visible: bool) {
        self.get_base_mut().visible = visible;
    }

    fn is_visible(&self) -> bool {
        self.get_base().visible
    }

    fn add_anchor(&mut self, this: AnchorKind, other_id: u32, other_side: AnchorKind) {
        self.get_base_mut().anchors.push(Anchor {
            anchor_this: this,
            anchor_widget_id: other_id,
            anchor_to: other_side,
        });
        self.get_base_mut().dirty = true;
    }

    fn add_anchor_to_parent(&mut self, this: AnchorKind, other_side: AnchorKind) {
        let parent_id = self.get_base().parent_id;

        self.get_base_mut().anchors.push(Anchor {
            anchor_this: this,
            anchor_widget_id: parent_id,
            anchor_to: other_side,
        });

        self.get_base_mut().dirty = true;
    }

    fn add_anchor_to_prev(&mut self, this: AnchorKind, other_side: AnchorKind) {
        let c: Vec<u32> = if let Some(parent_weak) = self.get_parent() {
            if let Some(parent_rc) = parent_weak.upgrade() {
                parent_rc.borrow().get_children_ids().to_vec()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        let mut found_this = false;

        for i in (0..c.len()).rev() {
            if let Some(prev) = c.get(i) {
                if *prev == self.get_id() {
                    found_this = true;
                    continue; // Skip self
                }
                if found_this {
                    // We found the previous widget
                    let prev = *prev;
                    let this = this.clone();
                    self.add_anchor(this, prev, other_side);
                    return;
                }
            }
        }
    }
    
    fn fill_parent(&mut self) {
        self.add_anchor_to_parent(
            AnchorKind::Left,
            AnchorKind::Left
        );
        self.add_anchor_to_parent(
            AnchorKind::Right,
            AnchorKind::Right
        );
        self.add_anchor_to_parent(
            AnchorKind::Top,
            AnchorKind::Top
        );
        self.add_anchor_to_parent(
            AnchorKind::Bottom,
            AnchorKind::Bottom
        );
    }

    fn set_size(&mut self, sz: SizeF) {
        self.get_base_mut().size = sz;
        self.get_base_mut().dirty = true;
    }
    fn set_position(&mut self, pos: PointF) {
        self.get_base_mut().position = pos;
        self.get_base_mut().dirty = true;
    }
    fn set_color(&mut self, color: Color) {
        self.get_base_mut().color = color;
    }

    fn get_drawing_coords(&mut self, ui: &Ui) -> QuadF {
        if self.get_base().dirty || self.get_base().computed_quad.is_none() {
            let quad = self.recompute_quad(ui);
            self.get_base_mut().computed_quad = Some(quad);
            self.get_base_mut().dirty = false;
            quad
        } else {
            self.get_base().computed_quad.unwrap()
        }
    }

    fn recompute_quad(&self, ui: &Ui) -> QuadF {
        let mut quad = QuadF::zero();
        let size = self.get_base().size;

        let mut left = None;
        let mut right = None;
        let mut top = None;
        let mut bottom = None;

        for anchor in &self.get_base().anchors {
            let anchor_widget = &ui.widgets[anchor.anchor_widget_id as usize];
            let anchor_pos = match anchor.anchor_to {
                AnchorKind::Left => anchor_widget.borrow_mut().get_left(ui),
                AnchorKind::Right => anchor_widget.borrow_mut().get_right(ui),
                AnchorKind::Top => anchor_widget.borrow_mut().get_top(ui),
                AnchorKind::Bottom => anchor_widget.borrow_mut().get_bottom(ui),
                AnchorKind::HorizontalCenter => {
                    let left = anchor_widget.borrow_mut().get_left(ui);
                    let right = anchor_widget.borrow_mut().get_right(ui);
                    (left + right) / 2.0
                }
                AnchorKind::VerticalCenter => {
                    let top = anchor_widget.borrow_mut().get_top(ui);
                    let bottom = anchor_widget.borrow_mut().get_bottom(ui);
                    (top + bottom) / 2.0
                }
            };

            match anchor.anchor_this {
                AnchorKind::Left => left = Some(anchor_pos),
                AnchorKind::Right => right = Some(anchor_pos),
                AnchorKind::Top => top = Some(anchor_pos),
                AnchorKind::Bottom => bottom = Some(anchor_pos),

                AnchorKind::HorizontalCenter => {
                    quad.x = anchor_pos - size.w / 2.0;
                    quad.w = size.w;

                    // Continue collecting vertical anchors, but skip horizontal logic
                    // We'll break early if both vertical center and center-x are found.
                    continue;
                }

                AnchorKind::VerticalCenter => {
                    quad.y = anchor_pos - size.h / 2.0;
                    quad.h = size.h;

                    // Continue collecting horizontal anchors, but skip vertical logic
                    continue;
                }
            }
        }

        // Horizontal fallback
        if quad.w == 0.0 {
            if let (Some(l), Some(r)) = (left, right) {
                quad.x = l;
                quad.w = r - l;
            } else if let Some(l) = left {
                quad.x = l;
                quad.w = size.w;
            } else if let Some(r) = right {
                quad.w = size.w;
                quad.x = r - quad.w;
            } else {
                quad.x = self.get_base().position.x;
                quad.w = size.w;
            }
        }

        // Vertical fallback
        if quad.h == 0.0 {
            if let (Some(t), Some(b)) = (top, bottom) {
                quad.y = t;
                quad.h = b - t;
            } else if let Some(t) = top {
                quad.y = t;
                quad.h = size.h;
            } else if let Some(b) = bottom {
                quad.h = size.h;
                quad.y = b - quad.h;
            } else {
                quad.y = self.get_base().position.y;
                quad.h = size.h;
            }
        }

        // Apply margins
        quad.x += self.get_base().margin.x;
        quad.y += self.get_base().margin.y;

        quad
    }

}

#[macro_export]
macro_rules! impl_widget_fns {
    ($t:ty, $base:ident) => {
        fn get_base(&self) -> &WidgetBase {
            &self.$base
        }

        fn get_base_mut(&mut self) -> &mut WidgetBase {
            &mut self.$base
        }

        fn draw(&self, ui: &Ui) {
            self.draw(ui);
        }

        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
    };
}

#[macro_export]
macro_rules! impl_widget {
    ($t:ty, $base:ident) => {
        impl Widget for $t {
            impl_widget_fns!($t, $base);
        }
    };
}
