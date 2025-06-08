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

pub mod point_f;
pub mod size_f;
mod quad_f;

#[macro_use]
mod widget;

pub mod widget_panel;
pub mod widget_text;

use std::{cell::RefCell, rc::Weak, rc::Rc, sync::Mutex};

use macroquad::prelude::*;
use once_cell::sync::OnceCell;

use crate::ui::{point_f::PointF, quad_f::QuadF, size_f::SizeF, widget::{Anchor, AnchorKind, Widget}, widget_panel::WidgetPanel, widget_text::WidgetText};
use std::fmt::Debug;

// trait WidgetDebug: Widget + Debug {}
// impl<T: Widget + Debug> WidgetDebug for T {}

static ROOT_ID: u32 = 0;

#[derive(Debug)]
pub struct Ui {
    player_hp: i32,
    player_max_hp: i32,
    player_sp: u32,

    pub is_character_sheet_visible: bool,
    pub is_focused: bool,

    pub left_panel_id: u32,
    pub right_panel_id: u32,
    pub character_sheet_id: u32,

    id_counter: u32,
    pub widgets: Vec<Rc<RefCell<dyn Widget>>>,
}

impl Ui {
    pub fn new() -> Self {
        let root = WidgetPanel::new(0, None);
        let mut ui = Ui {
            player_hp: 0,
            player_max_hp: 0,
            player_sp: 0,
            is_character_sheet_visible: false,
            is_focused: false,
            id_counter: 1,
            left_panel_id: u32::MAX,
            right_panel_id: u32::MAX,
            character_sheet_id: u32::MAX,
            widgets: Vec::new(),
        };

        ui.widgets.push(root);

        ui.create_left_panel();
        ui.create_right_panel();
        ui.create_character_sheet();
        ui
    }

    pub fn update_geometry(&self, resolution: SizeF) {
        self.widgets[ROOT_ID as usize].borrow_mut().set_size(SizeF::new(resolution.w, resolution.h));
        
        for widget in &self.widgets {
            // This only needs `&self`, not `&mut self`.
            // Each call to `update_drawing_coords` will write into that widget's Cell.
            widget.borrow_mut().get_drawing_coords(self);
        }
    }

    pub fn set_player_hp(&mut self, hp: i32, max_hp: i32) {
        self.player_hp = hp;
        self.player_max_hp = max_hp;
    }

    pub fn set_player_sp(&mut self, sp: u32) {
        self.player_sp = sp;
    }

    pub fn set_last_action(&mut self, _action: &str) {

    }

    pub fn show_character_sheet(&mut self) {
        self.is_character_sheet_visible = true;
        self.is_focused = true;
    }

    pub fn hide(&mut self) {
        self.is_character_sheet_visible = false;
        self.is_focused = false;
    }

    fn create_widget<T: Widget + 'static>(&mut self, parent: Option<Weak<RefCell<dyn Widget>>>) -> Rc<RefCell<T>> {
        let widget = T::new(self.id_counter, parent);
        let widget_dyn: Rc<RefCell<dyn Widget>> = widget.clone();

        self.widgets.push(widget_dyn);
        self.id_counter += 1;

        widget
    }

    fn create_left_panel(&mut self) {
        self.left_panel_id = self.id_counter;
        let left_panel_rc = self.create_widget::<WidgetPanel>(
            Some(Rc::downgrade(&self.widgets[ROOT_ID as usize]))
        );

        {
            let mut left_panel = left_panel_rc.borrow_mut();
            left_panel.set_size(SizeF::new(400.0, 0.0));
            left_panel.set_border(WHITE, 2.0);
            left_panel.add_anchor_to_parent(AnchorKind::Top, AnchorKind::Top);
            left_panel.add_anchor_to_parent(AnchorKind::Bottom, AnchorKind::Bottom);
            left_panel.add_anchor_to_parent(AnchorKind::Left, AnchorKind::Left);
        } // ← mutable borrow of `left_panel_rc` ends here

        let parent_dyn = Rc::clone(&self.widgets[self.left_panel_id as usize]);

        // HP label
        let hp_label = self.create_widget::<WidgetText>(
            Some(Rc::downgrade(&parent_dyn))
        );
        {
            let mut lbl = hp_label.borrow_mut();
            lbl.set_text(&"HP".to_string());
            lbl.set_color(Color { r:1.0, g:0.0, b:0.0, a:1.0 });
            lbl.set_margin(QuadF::new(10.0, 30.0, 0.0, 0.0));
            lbl.add_anchor_to_parent(AnchorKind::Top,    AnchorKind::Top);
            lbl.add_anchor_to_parent(AnchorKind::Left,   AnchorKind::Left);
        }

        // Current‐HP value
        let hp_value = self.create_widget::<WidgetText>(
            Some(Rc::downgrade(&parent_dyn))
        );
        {
            let mut val = hp_value.borrow_mut();
            val.set_text(&format!("{}", self.player_hp));
            val.set_margin_left(10.0);
            // Anchor it relative to the "HP" label we just made
            val.add_anchor(AnchorKind::Top,  hp_label.borrow().get_id(), AnchorKind::Top);
            val.add_anchor(AnchorKind::Left, hp_label.borrow().get_id(), AnchorKind::Right);
        }

        // Max‐HP value
        let hp_max_value = self.create_widget::<WidgetText>(
            Some(Rc::downgrade(&parent_dyn))
        );
        {
            let mut val = hp_max_value.borrow_mut();
            val.set_text(&format!("/{}", self.player_max_hp));
            // Anchor it relative to the "HP" label we just made
            val.add_anchor(AnchorKind::Top,  hp_value.borrow().get_id(), AnchorKind::Top);
            val.add_anchor(AnchorKind::Left, hp_value.borrow().get_id(), AnchorKind::Right);
        }

        let sp_label = self.create_widget::<WidgetText>(
            Some(Rc::downgrade(&parent_dyn))
        );
        {
            let mut lbl = sp_label.borrow_mut();
            lbl.set_text(&"SP".to_string());
            lbl.set_color(YELLOW);
            lbl.set_margin_top(10.0);
            lbl.add_anchor(AnchorKind::Top, hp_label.borrow().get_id(), AnchorKind::Bottom);
            lbl.add_anchor(AnchorKind::Left, hp_label.borrow().get_id(), AnchorKind::Left);
        }

        // Current‐SP value
        let sp_value = self.create_widget::<WidgetText>(
            Some(Rc::downgrade(&parent_dyn))
        );
        {
            let mut val = sp_value.borrow_mut();
            val.set_text(&format!("{}", self.player_sp));
            val.set_margin_left(10.0);
            // Anchor it relative to the "HP" label we just made
            val.add_anchor(AnchorKind::Top,  sp_label.borrow().get_id(), AnchorKind::Top);
            val.add_anchor(AnchorKind::Left, sp_label.borrow().get_id(), AnchorKind::Right);
        }
    }

    fn create_right_panel(&mut self) {
        self.right_panel_id = self.id_counter;
        let right_panel_rc = WidgetPanel::new(self.id_counter, Some(Rc::downgrade(&self.widgets[ROOT_ID as usize])));
        {
            let mut right_panel = right_panel_rc.borrow_mut();
            right_panel.set_size(SizeF::new(400.0, 0.0));
            right_panel.set_border(WHITE, 2.0);
            right_panel.add_anchor_to_parent(AnchorKind::Top, AnchorKind::Top);
            right_panel.add_anchor_to_parent(AnchorKind::Bottom, AnchorKind::Bottom);
            right_panel.add_anchor_to_parent(AnchorKind::Right, AnchorKind::Right);
        }
        
        // right_panel.set_visible(false);
        self.widgets.push(right_panel_rc);
        self.id_counter += 1;
    }

    fn create_character_sheet(&mut self) {
        self.character_sheet_id = self.id_counter;
        let character_sheet = self.create_widget::<WidgetPanel>(Some(Rc::downgrade(&self.widgets[ROOT_ID as usize])));
        let mut character_sheet = character_sheet.borrow_mut();
        character_sheet.set_size(SizeF::new(400.0, 0.0));
        character_sheet.set_border(WHITE, 2.0);
        character_sheet.add_anchor_to_parent(AnchorKind::Top, AnchorKind::Top);
        character_sheet.add_anchor_to_parent(AnchorKind::Bottom, AnchorKind::Bottom);
        character_sheet.add_anchor(AnchorKind::Left, self.left_panel_id, AnchorKind::Right);
        character_sheet.add_anchor(AnchorKind::Right, self.right_panel_id, AnchorKind::Left);
        character_sheet.set_color(BLACK);
        character_sheet.set_visible(false);
    }

    fn draw_right_panel(&mut self, resolution: (f32, f32)) {
        draw_rectangle_lines(
            resolution.0 - 400.0,
            0.0,
            400.0,
            resolution.1,
            2.0,
            WHITE,
        );

        draw_text("UI 2", resolution.0 - 400.0, 20.0, 30.0, WHITE);
    }

    fn draw_character_sheet(&mut self, resolution: (f32, f32)) {
        draw_rectangle_lines(
            400.0,
            0.0,
            resolution.0 - 400.0,
            resolution.1,
            2.0,
            WHITE,
        );

        let mut offset = (420.0, 30.0);

        draw_text("Spells", offset.0, offset.1, 30.0, WHITE);
        offset.1 += 30.0;
        draw_text("Learn New Spell (S)", offset.0, offset.1, 30.0, WHITE);

        offset = (resolution.0 - resolution.0 / 2.0, 30.0);
        draw_text("Skills", offset.0, offset.1, 30.0, WHITE);
        offset.1 += 30.0;
        draw_text("Learn New Skill (K)", offset.0, offset.1, 30.0, WHITE);
    }

    pub fn draw(&mut self) {
        let ui_ref = self as &Ui;

        // PRE-EXTRACT widgets to avoid borrow overlap:
        // let widgets = self.widgets.iter().collect::<Vec<_>>();

        // for widget in widgets {
        //     widget.draw(ui_ref);
        // }

        self.widgets[ROOT_ID as usize].borrow_mut().draw(ui_ref);
    }
}

// pub static UI: OnceCell<Mutex<Ui>> = OnceCell::new();

// pub fn ui_mut() -> std::sync::MutexGuard<'static, Ui> {
//     UI.get().expect("UI not initialized").lock().unwrap()
// }