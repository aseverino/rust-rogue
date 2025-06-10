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
pub mod widget_bar;

use std::{cell::RefCell, rc::Weak, rc::Rc};

use macroquad::prelude::*;

use crate::ui::{quad_f::QuadF, size_f::SizeF, widget::{AnchorKind, Widget}, widget_bar::WidgetBar, widget_panel::WidgetPanel, widget_text::WidgetText};
use std::fmt::Debug;


static ROOT_ID: u32 = 0;

#[derive(Debug)]
pub struct Ui {
    player_hp: u32,
    player_max_hp: u32,
    player_mp: u32,
    player_max_mp: u32,

    left_panel_id: u32,
    right_panel_id: u32,
    character_sheet_id: u32,
    hp_bar_id: u32,
    mp_bar_id: u32,
    
    id_counter: u32,
    widgets: Vec<Rc<RefCell<dyn Widget>>>,

    pub is_focused: bool,
}

impl Ui {
    pub fn new() -> Self {
        let mut ui = Ui {
            player_hp: 0,
            player_max_hp: 0,
            player_mp: 0,
            player_max_mp: 0,
            is_focused: false,
            id_counter: 1,
            left_panel_id: u32::MAX,
            right_panel_id: u32::MAX,
            character_sheet_id: u32::MAX,
            hp_bar_id: u32::MAX,
            mp_bar_id: u32::MAX,
            widgets: Vec::new(),
        };

        let root = WidgetPanel::new(&mut ui, 0, None);
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

    pub fn set_player_hp(&mut self, hp: u32, max_hp: u32) {
        self.player_hp = hp;
        self.player_max_hp = max_hp;

        if let Some(hp_bar) = self.widgets.get(self.hp_bar_id as usize) {
            let mut bar_ref = hp_bar.borrow_mut();
            if let Some(bar) = bar_ref.as_any_mut().downcast_mut::<WidgetBar>() {
                bar.set_text(&format!("{}/{}", self.player_hp, self.player_max_hp));
                bar.set_bar_percentage(self.player_hp as f32 / self.player_max_hp as f32);
            }
        }
    }

    pub fn set_player_mp(&mut self, mp: u32, max_mp: u32) {
        self.player_mp = mp;
        self.player_max_mp = max_mp;

        if let Some(mp_bar) = self.widgets.get(self.mp_bar_id as usize) {
            let mut bar_ref = mp_bar.borrow_mut();
            if let Some(bar) = bar_ref.as_any_mut().downcast_mut::<WidgetBar>() {
                bar.set_text(&format!("{}/{}", self.player_mp, self.player_max_mp));
                bar.set_bar_percentage(self.player_mp as f32 / self.player_max_mp as f32);
            }
        }
    }

    pub fn toggle_character_sheet(&mut self) {
        let is_visible = self.widgets[self.character_sheet_id as usize].borrow().is_visible();
        let mut cs = self.widgets[self.character_sheet_id as usize].borrow_mut();
        cs.set_visible(!is_visible)
    }

    pub fn hide(&mut self) {
        self.widgets[self.character_sheet_id as usize].borrow_mut().set_visible(false);
        self.is_focused = false;
    }

    // fn add_widget(&mut self, widget: Rc<RefCell<dyn Widget>>) {
    //     self.widgets.push(widget);
    //     self.id_counter += 1;

    //     if let Some(w) = self.widgets.last() {
    //         let new_children: Vec<_> = w.borrow().get_children()
    //             .iter()
    //             .filter_map(|c| c.upgrade())
    //             .collect();
    //         for child in new_children {
    //             self.widgets.push(child);
    //             self.id_counter += 1;
    //         }
    //     }
    // }

    // fn add_widget_recursive(&mut self, widget: Rc<RefCell<dyn Widget>>) {
    //     self.widgets.push(widget.clone());
    //     self.id_counter += 1;

    //     let children: Vec<_> = widget.borrow().get_children()
    //         .iter()
    //         .filter_map(|c| c.upgrade())
    //         .collect();

    //     for child in children {
    //         self.add_widget_recursive(child);
    //     }
    // }

    // fn create_widget<T: Widget + 'static>(&mut self, parent: Option<Weak<RefCell<dyn Widget>>>) -> Rc<RefCell<T>> {
    //     let widget = T::new(self, self.id_counter, parent);
    //     let widget_dyn: Rc<RefCell<dyn Widget>> = widget.clone();

    //     self.add_widget_recursive(widget_dyn);

    //     widget
    // }

    fn add_widget(&mut self, widget: Rc<RefCell<dyn Widget>>) {
        self.widgets.push(widget);
        self.id_counter += 1;
    }


    fn create_widget<T: Widget + 'static>(&mut self, parent: Option<Weak<RefCell<dyn Widget>>>) -> Rc<RefCell<T>> {
        let widget = T::new(self, self.id_counter, parent);
        let widget_dyn: Rc<RefCell<dyn Widget>> = widget.clone();

        self.add_widget(widget_dyn);

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

        self.hp_bar_id = self.id_counter;
        let hp_bar = self.create_widget::<WidgetBar>(
            Some(Rc::downgrade(&parent_dyn))
        );
        {
            let mut bar = hp_bar.borrow_mut();
            bar.set_size(SizeF::new(200.0, 20.0));
            bar.add_anchor(AnchorKind::Top, hp_label.borrow().get_id(), AnchorKind::Top);
            bar.add_anchor(AnchorKind::Left, hp_label.borrow().get_id(), AnchorKind::Right);
            bar.set_text(&format!("{}/{}", self.player_hp, self.player_max_hp));
            bar.set_background_color(Color { r:0.0, g:0.0, b:0.0, a:1.0 });
            bar.set_bar_color(Color { r:1.0, g:0.0, b:0.0, a:1.0 });

            if self.player_max_hp > 0 {
                bar.set_bar_percentage(self.player_hp as f32 / self.player_max_hp as f32);
            }
        }

        let mp_label = self.create_widget::<WidgetText>(
            Some(Rc::downgrade(&parent_dyn))
        );
        {
            let mut lbl = mp_label.borrow_mut();
            lbl.set_text(&"MP".to_string());
            lbl.set_color(BLUE);
            lbl.set_margin_top(10.0);
            lbl.add_anchor(AnchorKind::Top, hp_label.borrow().get_id(), AnchorKind::Bottom);
            lbl.add_anchor(AnchorKind::Left, hp_label.borrow().get_id(), AnchorKind::Left);
        }

        self.mp_bar_id = self.id_counter;
        let mp_bar = self.create_widget::<WidgetBar>(
            Some(Rc::downgrade(&parent_dyn))
        );
        {
            let mut bar = mp_bar.borrow_mut();
            bar.set_size(SizeF::new(200.0, 20.0));
            bar.add_anchor(AnchorKind::Top, mp_label.borrow().get_id(), AnchorKind::Top);
            bar.add_anchor(AnchorKind::Left, mp_label.borrow().get_id(), AnchorKind::Right);
            bar.set_text(&format!("{}/{}", self.player_mp, self.player_max_mp));
            bar.set_background_color(Color { r:0.0, g:0.0, b:0.0, a:1.0 });
            bar.set_bar_color(Color { r:0.0, g:0.0, b:1.0, a:1.0 });

            if self.player_max_mp > 0 {
                bar.set_bar_percentage(self.player_mp as f32 / self.player_max_mp as f32);
            }
        }
    }

    fn create_right_panel(&mut self) {
        self.right_panel_id = self.id_counter;
        let right_panel_rc = self.create_widget::<WidgetPanel>(
            Some(Rc::downgrade(&self.widgets[ROOT_ID as usize])));
        {
            let mut right_panel = right_panel_rc.borrow_mut();
            right_panel.set_size(SizeF::new(400.0, 0.0));
            right_panel.set_border(WHITE, 2.0);
            right_panel.add_anchor_to_parent(AnchorKind::Top, AnchorKind::Top);
            right_panel.add_anchor_to_parent(AnchorKind::Bottom, AnchorKind::Bottom);
            right_panel.add_anchor_to_parent(AnchorKind::Right, AnchorKind::Right);
        }
        
        // self.widgets.push(right_panel_rc);
        // self.id_counter += 1;
    }

    fn create_character_sheet(&mut self) {
        self.character_sheet_id = self.id_counter;
        let character_sheet_rc = self.create_widget::<WidgetPanel>(Some(Rc::downgrade(&self.widgets[ROOT_ID as usize])));
        {
            let mut character_sheet = character_sheet_rc.borrow_mut();
            character_sheet.set_border(WHITE, 2.0);
            character_sheet.add_anchor_to_parent(AnchorKind::Top, AnchorKind::Top);
            character_sheet.add_anchor_to_parent(AnchorKind::Bottom, AnchorKind::Bottom);
            //character_sheet.add_anchor_to_parent(AnchorKind::Left, AnchorKind::Right);
            //character_sheet.add_anchor_to_parent(AnchorKind::Right, AnchorKind::Left);
            character_sheet.add_anchor(AnchorKind::Left, self.left_panel_id, AnchorKind::Right);
            character_sheet.add_anchor(AnchorKind::Right, self.right_panel_id, AnchorKind::Left);
            character_sheet.set_color(BLACK);
            character_sheet.set_visible(false);
        }

        let parent_dyn = Rc::clone(&self.widgets[self.character_sheet_id as usize]);

        let spell_title_id = self.id_counter;
        let spells_title = self.create_widget::<WidgetText>(
            Some(Rc::downgrade(&parent_dyn))
        );
        {
            let mut lbl = spells_title.borrow_mut();
            lbl.set_text(&"Spells".to_string());
            lbl.set_margin(QuadF::new(10.0, 30.0, 0.0, 0.0));
            lbl.add_anchor_to_parent(AnchorKind::Top, AnchorKind::Top);
            lbl.add_anchor_to_parent(AnchorKind::Left, AnchorKind::Left);
        }
        
        let spells_learn = self.create_widget::<WidgetText>(
            Some(Rc::downgrade(&parent_dyn))
        );
        {
            let mut lbl = spells_learn.borrow_mut();
            lbl.set_text(&"Learn New Spell (S)".to_string());
            lbl.set_margin_top(30.0);
            lbl.add_anchor(AnchorKind::Top, spell_title_id, AnchorKind::Top);
            lbl.add_anchor(AnchorKind::Left, spell_title_id, AnchorKind::Left);
        }

        let skills_title_id = self.id_counter;
        let skills_title = self.create_widget::<WidgetText>(
            Some(Rc::downgrade(&parent_dyn))
        );
        {
            let mut lbl = skills_title.borrow_mut();
            lbl.set_text(&"Skills".to_string());
            lbl.set_margin(QuadF::new(10.0, 30.0, 0.0, 0.0));
            lbl.add_anchor_to_parent(AnchorKind::Top, AnchorKind::Top);
            lbl.add_anchor_to_parent(AnchorKind::Left, AnchorKind::HorizontalCenter);
        }

        let skills_learn = self.create_widget::<WidgetText>(
            Some(Rc::downgrade(&parent_dyn))
        );
        {
            let mut lbl = skills_learn.borrow_mut();
            lbl.set_text(&"Learn New Skill (K)".to_string());
            lbl.set_margin_top(30.0);
            lbl.add_anchor(AnchorKind::Top, skills_title_id, AnchorKind::Top);
            lbl.add_anchor(AnchorKind::Left, skills_title_id, AnchorKind::Left);
        }
    }

    pub fn draw(&mut self) {
        let ui_ref = self as &Ui;

        self.widgets[ROOT_ID as usize].borrow_mut().draw(ui_ref);
    }
}
