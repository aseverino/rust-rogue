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

use std::{cell::RefCell, rc::Rc, sync::Mutex};

use macroquad::prelude::*;
use once_cell::sync::OnceCell;

use crate::ui::{point_f::PointF, size_f::SizeF, widget::{Anchor, AnchorKind, Widget}, widget_panel::WidgetPanel};
use std::fmt::Debug;

trait WidgetDebug: Widget + Debug {}
impl<T: Widget + Debug> WidgetDebug for T {}

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
    pub widgets: Vec<Box<dyn WidgetDebug>>,
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

        ui.widgets.push(Box::new(root));

        ui.create_left_panel();
        ui.create_right_panel();
        ui.create_character_sheet();
        ui
    }

    pub fn update_geometry(&mut self, resolution: SizeF) {
        // Step 1: resize the root widget by &mut borrow
        if let Some(root) = self
            .widgets
            .iter()
            .find(|w| w.get_id() == ROOT_ID)
        {
            // We know `root` is a &Widget, not &mut Widget. But we want &mut Widget to set_size.
            // In practice, you can retrieve the index or split_mut. Easiest is:
            let root_idx = self
                .widgets
                .iter()
                .position(|w| w.get_id() == ROOT_ID)
                .unwrap();
            self.widgets[root_idx].set_size(SizeF::new(resolution.w, resolution.h));
        }

        // Step 2: recompute drawing_coords for every widget, using only &self
        // (interior mutability takes care of writing into computed_quad).
        for widget in &self.widgets {
            // This only needs `&self`, not `&mut self`.
            // Each call to `update_drawing_coords` will write into that widget's Cell.
            widget.get_drawing_coords(self);
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

    fn create_left_panel(&mut self) {
        self.left_panel_id = self.id_counter;
        let mut left_panel: WidgetPanel = WidgetPanel::new(self.id_counter, Some(ROOT_ID));
        left_panel.set_size(SizeF::new(400.0, 0.0));
        left_panel.set_border(WHITE, 2.0);
        left_panel.add_anchor(AnchorKind::Top, ROOT_ID, AnchorKind::Top);
        left_panel.add_anchor(AnchorKind::Bottom, ROOT_ID, AnchorKind::Bottom);
        left_panel.add_anchor(AnchorKind::Left, ROOT_ID, AnchorKind::Left);
        // left_panel.set_visible(false);
        self.widgets.push(Box::new(left_panel));
        self.id_counter += 1;


        // self.widgets.push(Box::new(WidgetPanel::new(
        //     (0.0, 0.0),
        //     (400.0, -1.0),
        //     WHITE,
        // )));

        // draw_rectangle_lines(
        //     0.0,
        //     0.0,
        //     400.0,
        //     resolution.1,
        //     2.0,
        //     WHITE,
        // );

        // let mut offset: (f32, f32) = (10.0, 30.0);

        // let mut text: String = "HP".to_string();
        // let true_red = Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0};
        // draw_text(&text, offset.0, offset.1, 30.0, true_red);
        // let mut text_offset = measure_text(&text, None, 30, 1.0).width;
        // text_offset += offset.0 + 10.0;
        // let start_of_health = text_offset;

        // text = format!("{}", self.player_hp);
        // draw_text(&text, text_offset, offset.1, 30.0, if self.player_hp < self.player_max_hp / 2 { true_red } else { WHITE });
        // text_offset += measure_text(&text, None, 30, 1.0).width;

        // text = format!("/{}", self.player_max_hp);
        // draw_text(&text,text_offset,offset.1,30.0,WHITE);

        // offset.1 += 30.0;
        // text = "SP".to_string();
        // draw_text(&text,offset.0,offset.1,30.0,YELLOW);
        // text_offset = start_of_health;

        // text = format!("{}", self.player_sp);
        // draw_text(&text,text_offset,offset.1,30.0,YELLOW);
    }

    fn create_right_panel(&mut self) {
        self.right_panel_id = self.id_counter;
        let mut right_panel: WidgetPanel = WidgetPanel::new(self.id_counter, Some(ROOT_ID));
        right_panel.set_size(SizeF::new(400.0, 0.0));
        right_panel.set_border(WHITE, 2.0);
        right_panel.add_anchor(AnchorKind::Top, ROOT_ID, AnchorKind::Top);
        right_panel.add_anchor(AnchorKind::Bottom, ROOT_ID, AnchorKind::Bottom);
        right_panel.add_anchor(AnchorKind::Right, ROOT_ID, AnchorKind::Right);
        // right_panel.set_visible(false);
        self.widgets.push(Box::new(right_panel));
        self.id_counter += 1;
    }

    fn create_character_sheet(&mut self) {
        self.character_sheet_id = self.id_counter;
        let mut character_sheet: WidgetPanel = WidgetPanel::new(self.id_counter, Some(ROOT_ID));
        character_sheet.set_size(SizeF::new(400.0, 0.0));
        character_sheet.set_border(WHITE, 2.0);
        character_sheet.add_anchor(AnchorKind::Top, ROOT_ID, AnchorKind::Top);
        character_sheet.add_anchor(AnchorKind::Bottom, ROOT_ID, AnchorKind::Bottom);
        character_sheet.add_anchor(AnchorKind::Left, self.left_panel_id, AnchorKind::Right);
        character_sheet.add_anchor(AnchorKind::Right, self.right_panel_id, AnchorKind::Left);
        character_sheet.set_color(BLACK);
        self.widgets.push(Box::new(character_sheet));
        self.id_counter += 1;
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
        let widgets = self.widgets.iter().collect::<Vec<_>>();

        for widget in widgets {
            widget.draw(ui_ref);
        }
    }
}

// pub static UI: OnceCell<Mutex<Ui>> = OnceCell::new();

// pub fn ui_mut() -> std::sync::MutexGuard<'static, Ui> {
//     UI.get().expect("UI not initialized").lock().unwrap()
// }