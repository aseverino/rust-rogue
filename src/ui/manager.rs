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

use std::{cell::RefCell, collections::VecDeque, rc::{Rc, Weak}};

use macroquad::prelude::*;

use crate::ui::{point_f::PointF, quad_f::QuadF, size_f::SizeF, widget::{AnchorKind, Widget}, widget_bar::WidgetBar, widget_button::WidgetButton, widget_panel::WidgetPanel, widget_text::WidgetText};
use std::fmt::Debug;


static ROOT_ID: u32 = 0;

#[derive(Debug, Clone)]
pub enum UiEvent {
    IncStrength,
    IncDexterity,
    IncIntelligence,
}

#[derive(Debug)]
pub struct Ui {
    player_hp: u32,
    player_max_hp: u32,
    player_mp: u32,
    player_max_mp: u32,
    player_sp: u32,
    player_str: u32,
    player_dex: u32,
    player_int: u32,

    left_panel_id: u32,
    right_panel_id: u32,
    character_sheet_id: u32,
    chest_view_id: u32,
    hp_bar_id: u32,
    mp_bar_id: u32,
    sp_value_id: u32,
    str_area_button_id: u32,
    dex_area_button_id: u32,
    int_area_button_id: u32,
    str_value_bound_ids: Vec<u32>,
    dex_value_bound_ids: Vec<u32>,
    int_value_bound_ids: Vec<u32>,

    pub events: VecDeque<UiEvent>,
    
    id_counter: u32,
    pub widgets: Vec<Rc<RefCell<dyn Widget>>>,

    pub is_focused: bool,
}

impl Ui {
    pub fn new() -> Self {
        let mut ui = Ui {
            player_hp: 1,
            player_max_hp: 1,
            player_mp: 0,
            player_max_mp: 0,
            player_sp: 0,
            player_str: 0,
            player_dex: 0,
            player_int: 0,
            is_focused: false,
            id_counter: 0,
            left_panel_id: u32::MAX,
            right_panel_id: u32::MAX,
            character_sheet_id: u32::MAX,
            chest_view_id: u32::MAX,
            hp_bar_id: u32::MAX,
            mp_bar_id: u32::MAX,
            sp_value_id: u32::MAX,
            str_area_button_id: u32::MAX,
            dex_area_button_id: u32::MAX,
            int_area_button_id: u32::MAX,
            str_value_bound_ids: Vec::new(),
            dex_value_bound_ids: Vec::new(),
            int_value_bound_ids: Vec::new(),

            events: VecDeque::new(),

            widgets: Vec::new(),
        };

        let root = WidgetPanel::new(&mut ui, 0, None);
        ui.widgets.push(root);

        ui.create_left_panel();
        ui.create_right_panel();
        ui.create_character_sheet();
        ui.create_chest_view();
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

    pub fn update_mouse_position(&mut self, mouse_position: PointF) {
        let widgets: Vec<_> = self.widgets.iter().cloned().collect();
        for widget in widgets {
            widget.borrow_mut().on_mouse_position_update(self, mouse_position);
        }
    }

    pub fn handle_click(&mut self, mouse_position: PointF) {
        let widgets: Vec<_> = self.widgets.iter().cloned().collect();
        for widget in widgets {
            widget.borrow_mut().on_click(self, mouse_position);
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

    pub fn set_player_sp(&mut self, sp: u32) {
        self.player_sp = sp;

        if let Some(sp_value) = self.widgets.get(self.sp_value_id as usize) {
            let mut text_ref = sp_value.borrow_mut();
            if let Some(text) = text_ref.as_any_mut().downcast_mut::<WidgetText>() {
                text.set_text(&format!("{}", self.player_sp));
            }
        }
    }

    pub fn set_player_str(&mut self, str: u32) {
        self.player_str = str;

        for id in &self.str_value_bound_ids {
            if let Some(str_value) = self.widgets.get(*id as usize) {
                let mut text_ref = str_value.borrow_mut();
                if let Some(text) = text_ref.as_any_mut().downcast_mut::<WidgetText>() {
                    text.set_text(&format!("{}", self.player_str));
                }
            }
        }
    }

    pub fn set_player_dex(&mut self, dex: u32) {
        self.player_dex = dex;

        for id in &self.dex_value_bound_ids {
            if let Some(dex_value) = self.widgets.get(*id as usize) {
                let mut text_ref = dex_value.borrow_mut();
                if let Some(text) = text_ref.as_any_mut().downcast_mut::<WidgetText>() {
                    text.set_text(&format!("{}", self.player_dex));
                }
            }
        }
    }

    pub fn set_player_int(&mut self, int: u32) {
        self.player_int = int;

        for id in &self.int_value_bound_ids {
            if let Some(int_value) = self.widgets.get(*id as usize) {
                let mut text_ref = int_value.borrow_mut();
                if let Some(text) = text_ref.as_any_mut().downcast_mut::<WidgetText>() {
                    text.set_text(&format!("{}", self.player_int));
                }
            }
        }
    }

    pub fn set_chest_items(&mut self, items: &Vec<(u32, String)>, choice_cb: Box<dyn FnMut(u32)>) {
        use std::cell::RefCell;
        use std::rc::Rc;

        let chest_view_rc = if let Some(chest_view) = self.widgets.get(self.chest_view_id as usize) {
            chest_view.clone()
        } else {
            return; // early return if missing
        };
        
        let panel_children_len = {
            let mut chest_ref = chest_view_rc.borrow_mut();
            if let Some(panel) = chest_ref.as_any_mut().downcast_mut::<WidgetPanel>() {
                panel.get_children().len()
            } else {
                return; // not a panel → nothing to do
            }
        };

        let choice_cb = Rc::new(RefCell::new(choice_cb));

        if panel_children_len - 1 < items.len() {
            for &(item_id, ref item_name) in items.iter().skip(panel_children_len - 1) {
                let item_widget = self.create_widget::<WidgetButton>(
                    Some(Rc::downgrade(&chest_view_rc))
                );
                {
                    let mut item_text = item_widget.borrow_mut();
                    item_text.set_text(&format!("{}", item_name));
                    item_text.set_margin_top(10.0);
                    item_text.add_anchor_to_prev(AnchorKind::Top, AnchorKind::Bottom);
                    item_text.add_anchor_to_prev(AnchorKind::Left, AnchorKind::Left);
                    let choice_cb = Rc::clone(&choice_cb);
                    item_text.set_on_click(Box::new(move |_ui, _| {
                        // Handle item click
                        //println!("Clicked on item: {}", item_id);
                        (choice_cb.borrow_mut())(item_id);
                    }));
                }
            }
        }

        // Update existing items
        let mut chest_ref = chest_view_rc.borrow_mut();
        if let Some(panel) = chest_ref.as_any_mut().downcast_mut::<WidgetPanel>() {
            let panel_children = panel.get_children();

            for (index, (item_id, item_name)) in items.iter().enumerate() {
                if let Some(child_rc) = panel_children.get(index + 1).and_then(|c| c.upgrade()) {
                    let mut child_ref = child_rc.borrow_mut();
                    if let Some(item_text) = child_ref.as_any_mut().downcast_mut::<WidgetButton>() {
                        item_text.set_text(&format!("{}", item_name));
                    }
                }
            }
        }
    }

    pub fn toggle_character_sheet(&mut self) {
        let is_visible = self.widgets[self.character_sheet_id as usize].borrow().is_visible();
        let mut cs = self.widgets[self.character_sheet_id as usize].borrow_mut();
        cs.set_visible(!is_visible);
        self.is_focused = !is_visible;
    }

    pub fn show_chest_view(&mut self, items: &Vec<(u32, String)>, choice_cb: Box<dyn FnMut(u32)>) {
        self.set_chest_items(items, choice_cb);
        self.widgets[self.chest_view_id as usize].borrow_mut().set_visible(true);
        self.is_focused = true;
        
    }

    pub fn hide(&mut self) {
        self.widgets[self.character_sheet_id as usize].borrow_mut().set_visible(false);
        self.widgets[self.chest_view_id as usize].borrow_mut().set_visible(false);
        self.is_focused = false;
    }

    

    // pub fn add_widget(&mut self, widget: Rc<RefCell<dyn Widget>>) {
    //     self.id_counter += 1;
    //     self.widgets.push(widget);
    // }


    pub fn create_widget<T: Widget + 'static>(&mut self, parent: Option<Weak<RefCell<dyn Widget>>>) -> Rc<RefCell<T>> {
        self.id_counter += 1;
        let widget = T::new(self, self.id_counter, parent);
        let widget_dyn: Rc<RefCell<dyn Widget>> = widget.clone();

        if !widget.borrow().is_manually_added() {
            self.widgets.push(widget_dyn);    
        }

        if self.widgets.len() != self.id_counter as usize + 1 {
            panic!("Widget ID mismatch: expected {}, got {}", self.id_counter, self.widgets.len() - 1);
        }

        widget
    }

    pub fn create_attr_button(&mut self, event: UiEvent, label: &str, value: u32, parent: &Rc<RefCell<dyn Widget>>, margin_top: f32) -> (Rc<RefCell<dyn Widget>>, u32) {
        let button = self.create_widget::<WidgetButton>(
            Some(Rc::downgrade(parent))
        );
        {
            let mut attr_button = button.borrow_mut();
            attr_button.set_on_click(Box::new(move |ui, _| {
                ui.events.push_back(event.clone());
            }));
            //attr_panel.set_border(WHITE, 1.0);
            attr_button.set_size(SizeF::new(150.0, 50.0));
            attr_button.add_anchor_to_parent(AnchorKind::Top, AnchorKind::Top);
            attr_button.add_anchor_to_parent(AnchorKind::Left, AnchorKind::Left);
            attr_button.set_margin_top(margin_top);
        }

        let label_widget = self.create_widget::<WidgetText>(
                        Some(Rc::downgrade(&(button.clone() as Rc<RefCell<dyn Widget>>)))
                    );
        {
            let mut lbl = label_widget.borrow_mut();
            lbl.set_text(&label.to_string());
            lbl.set_margin_left(30.0);
            lbl.add_anchor_to_parent(AnchorKind::VerticalCenter, AnchorKind::VerticalCenter);
            lbl.add_anchor_to_parent(AnchorKind::Left, AnchorKind::Left);
        }

        let value_id = self.id_counter + 1;
        let value_widget = self.create_widget::<WidgetText>(
                        Some(Rc::downgrade(&(button.clone() as Rc<RefCell<dyn Widget>>)))
                    );
        {
            let mut val = value_widget.borrow_mut();
            val.set_text(&value.to_string());
            val.set_margin_right(30.0);
            val.add_anchor_to_prev(AnchorKind::Top, AnchorKind::Top);
            val.add_anchor_to_parent(AnchorKind::Right, AnchorKind::Right);
        }

        (button, value_id)
    }

    fn create_left_panel(&mut self) {
        self.left_panel_id = self.id_counter + 1;
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

        self.hp_bar_id = self.id_counter + 1;
        let hp_bar = self.create_widget::<WidgetBar>(
            Some(Rc::downgrade(&parent_dyn))
        );
        {
            let mut bar = hp_bar.borrow_mut();
            bar.set_size(SizeF::new(200.0, 20.0));
            bar.add_anchor_to_prev(AnchorKind::Top, AnchorKind::Top);
            bar.add_anchor_to_prev(AnchorKind::Left, AnchorKind::Right);
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

        self.mp_bar_id = self.id_counter + 1;
        let mp_bar = self.create_widget::<WidgetBar>(
            Some(Rc::downgrade(&parent_dyn))
        );
        {
            let mut bar = mp_bar.borrow_mut();
            bar.set_size(SizeF::new(200.0, 20.0));
            bar.add_anchor_to_prev(AnchorKind::Top, AnchorKind::Top);
            bar.add_anchor_to_prev(AnchorKind::Left, AnchorKind::Right);
            bar.set_text(&format!("{}/{}", self.player_mp, self.player_max_mp));
            bar.set_background_color(Color { r:0.0, g:0.0, b:0.0, a:1.0 });
            bar.set_bar_color(Color { r:0.0, g:0.0, b:1.0, a:1.0 });

            if self.player_max_mp > 0 {
                bar.set_bar_percentage(self.player_mp as f32 / self.player_max_mp as f32);
            }
        }

        let soul_label = self.create_widget::<WidgetText>(
            Some(Rc::downgrade(&parent_dyn))
        );
        {
            let mut lbl = soul_label.borrow_mut();
            lbl.set_text(&"SP".to_string());
            lbl.set_color(YELLOW);
            lbl.set_margin_top(10.0);
            lbl.add_anchor(AnchorKind::Top, mp_label.borrow().get_id(), AnchorKind::Bottom);
            lbl.add_anchor(AnchorKind::Left, mp_label.borrow().get_id(), AnchorKind::Left);
        }

        self.sp_value_id = self.id_counter + 1;
        let soul_value = self.create_widget::<WidgetText>(
            Some(Rc::downgrade(&parent_dyn))
        );
        {
            let mut lbl = soul_value.borrow_mut();
            lbl.set_color(YELLOW);
            lbl.add_anchor_to_prev(AnchorKind::Top, AnchorKind::Top);
            lbl.add_anchor_to_prev(AnchorKind::Left, AnchorKind::Right);
            lbl.set_text(&format!("{}", self.player_sp));
        }

        let str_label = self.create_widget::<WidgetText>(
            Some(Rc::downgrade(&parent_dyn))
        );
        {
            let mut lbl = str_label.borrow_mut();
            lbl.set_text(&"STR".to_string());
            lbl.set_color(GREEN);
            lbl.set_margin_top(10.0);
            lbl.add_anchor(AnchorKind::Top, soul_label.borrow().get_id(), AnchorKind::Bottom);
            lbl.add_anchor(AnchorKind::Left, soul_label.borrow().get_id(), AnchorKind::Left);
        }

        self.str_value_bound_ids.push(self.id_counter + 1);
        let str_value = self.create_widget::<WidgetText>(
            Some(Rc::downgrade(&parent_dyn))
        );
        {
            let mut lbl = str_value.borrow_mut();
            lbl.set_color(GREEN);
            lbl.add_anchor_to_prev(AnchorKind::Top, AnchorKind::Top);
            lbl.add_anchor_to_prev(AnchorKind::Left, AnchorKind::Right);
            lbl.set_text(&format!("{}", self.player_str));
        }
    }

    fn create_right_panel(&mut self) {
        self.right_panel_id = self.id_counter + 1;
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
    }

    fn create_character_sheet(&mut self) {
        self.character_sheet_id = self.id_counter + 1;
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

        let attributes_tab_id = self.id_counter + 1;
        let attributes_tab = self.create_widget::<WidgetButton>(
            Some(Rc::downgrade(&parent_dyn))
        );
        {
            let mut lbl = attributes_tab.borrow_mut();
            lbl.toggled = true;
            lbl.set_text(&"Attrib".to_string());
            lbl.set_margin(QuadF::new(10.0, 30.0, 0.0, 0.0));
            lbl.add_anchor_to_parent(AnchorKind::Top, AnchorKind::Top);
            lbl.add_anchor_to_parent(AnchorKind::Left, AnchorKind::Left);
        }

        let skills_tab_id = self.id_counter + 1;
        let skills_tab = self.create_widget::<WidgetButton>(
            Some(Rc::downgrade(&parent_dyn))
        );
        {
            let mut lbl = skills_tab.borrow_mut();
            lbl.set_text(&"Skills".to_string());
            lbl.set_margin(QuadF::new(30.0, 0.0, 0.0, 0.0));
            lbl.add_anchor_to_prev(AnchorKind::Top, AnchorKind::Top);
            lbl.add_anchor_to_prev(AnchorKind::Left, AnchorKind::Right);
        }

        let abilities_tab_id = self.id_counter + 1;
        let abilities_tab = self.create_widget::<WidgetButton>(
            Some(Rc::downgrade(&parent_dyn))
        );
        {
            let mut lbl = abilities_tab.borrow_mut();
            lbl.set_text(&"Abilit".to_string());
            lbl.set_margin(QuadF::new(30.0, 0.0, 0.0, 0.0));
            lbl.add_anchor_to_prev(AnchorKind::Top, AnchorKind::Top);
            lbl.add_anchor_to_prev(AnchorKind::Left, AnchorKind::Right);
        }

        let equipment_tab_id = self.id_counter + 1;
        let equipment_tab = self.create_widget::<WidgetButton>(
            Some(Rc::downgrade(&parent_dyn))
        );
        {
            let mut lbl = equipment_tab.borrow_mut();
            lbl.set_text(&"Equip".to_string());
            lbl.set_margin(QuadF::new(30.0, 0.0, 0.0, 0.0));
            lbl.add_anchor_to_prev(AnchorKind::Top, AnchorKind::Top);
            lbl.add_anchor_to_prev(AnchorKind::Left, AnchorKind::Right);
        }

        let inventory_tab_id = self.id_counter + 1;
        let inventory_tab = self.create_widget::<WidgetButton>(
            Some(Rc::downgrade(&parent_dyn))
        );
        {
            let mut lbl = inventory_tab.borrow_mut();
            lbl.set_text(&"Invent".to_string());
            lbl.set_margin(QuadF::new(30.0, 0.0, 0.0, 0.0));
            lbl.add_anchor_to_prev(AnchorKind::Top, AnchorKind::Top);
            lbl.add_anchor_to_prev(AnchorKind::Left, AnchorKind::Right);
        }

        let attributes_sheet_id = self.id_counter + 1;
        let attributes_sheet_rc = self.create_widget::<WidgetPanel>(Some(Rc::downgrade(&parent_dyn)));
        {
            let mut attr_sheet = attributes_sheet_rc.borrow_mut();
            attr_sheet.set_border(WHITE, 2.0);
            attr_sheet.add_anchor(AnchorKind::Top, attributes_tab_id, AnchorKind::Bottom);
            attr_sheet.add_anchor_to_parent(AnchorKind::Left, AnchorKind::Left);
            attr_sheet.add_anchor_to_parent(AnchorKind::Right, AnchorKind::Right);
            attr_sheet.add_anchor_to_parent(AnchorKind::Bottom, AnchorKind::Bottom);
        }

        let attr_as_parent_dyn = Rc::clone(&self.widgets[attributes_sheet_id as usize]);

        let (dex_area, dex_value_id) = self.create_attr_button(
            UiEvent::IncDexterity,
            "DEX",
            self.player_dex,
            &attr_as_parent_dyn,
            10.0
        );

        {
            let mut dex_area_button = dex_area.borrow_mut();
            dex_area_button.center_parent();
            self.dex_area_button_id = dex_area_button.get_id();
        }
        self.dex_value_bound_ids.push(dex_value_id);

        let (str_area, str_value_id) = self.create_attr_button(
            UiEvent::IncStrength,
            "STR",
            self.player_str,
            &attr_as_parent_dyn,
            0.0
        );

        {
            let mut str_area_button = str_area.borrow_mut();
            str_area_button.break_anchors();
            str_area_button.add_anchor_to_prev(AnchorKind::Top, AnchorKind::Top);
            str_area_button.add_anchor_to_prev(AnchorKind::Right, AnchorKind::Left);
            self.str_area_button_id = str_area_button.get_id();
        }
        self.str_value_bound_ids.push(str_value_id);

        let (int_area, int_value_id) = self.create_attr_button(
            UiEvent::IncIntelligence,
            "INT",
            self.player_int,
            &attr_as_parent_dyn,
            0.0
        );

        {
            let mut int_area_button = int_area.borrow_mut();
            int_area_button.break_anchors();
            int_area_button.add_anchor(AnchorKind::Top, dex_area.borrow().get_id(), AnchorKind::Top);
            int_area_button.add_anchor(AnchorKind::Left, dex_area.borrow().get_id(), AnchorKind::Right);
            self.int_area_button_id = int_area_button.get_id();
        }
        self.int_value_bound_ids.push(int_value_id);
        // let dex_title = self.create_widget::<WidgetText>(
        //     Some(Rc::downgrade(&attr_as_parent_dyn))
        // );
        // {
        //     let mut lbl = dex_title.borrow_mut();
        //     lbl.set_text(&"DEX".to_string());
        //     lbl.add_anchor_to_parent(AnchorKind::VerticalCenter, AnchorKind::VerticalCenter);
        //     lbl.add_anchor_to_parent(AnchorKind::HorizontalCenter, AnchorKind::HorizontalCenter);
        // }
        
        // let spells_learn = self.create_widget::<WidgetText>(
        //     Some(Rc::downgrade(&parent_dyn))
        // );
        // {
        //     let mut lbl = spells_learn.borrow_mut();
        //     lbl.set_text(&"Learn New Spell (S)".to_string());
        //     lbl.set_margin_top(30.0);
        //     lbl.add_anchor(AnchorKind::Top, spell_title_id, AnchorKind::Top);
        //     lbl.add_anchor(AnchorKind::Left, spell_title_id, AnchorKind::Left);
        // }

        // let skills_title_id = self.id_counter;
        // let skills_title = self.create_widget::<WidgetText>(
        //     Some(Rc::downgrade(&parent_dyn))
        // );
        // {
        //     let mut lbl = skills_title.borrow_mut();
        //     lbl.set_text(&"Skills".to_string());
        //     lbl.set_margin(QuadF::new(10.0, 30.0, 0.0, 0.0));
        //     lbl.add_anchor_to_parent(AnchorKind::Top, AnchorKind::Top);
        //     lbl.add_anchor_to_parent(AnchorKind::Left, AnchorKind::HorizontalCenter);
        // }

        // let skills_learn = self.create_widget::<WidgetText>(
        //     Some(Rc::downgrade(&parent_dyn))
        // );
        // {
        //     let mut lbl = skills_learn.borrow_mut();
        //     lbl.set_text(&"Learn New Skill (K)".to_string());
        //     lbl.set_margin_top(30.0);
        //     lbl.add_anchor(AnchorKind::Top, skills_title_id, AnchorKind::Top);
        //     lbl.add_anchor(AnchorKind::Left, skills_title_id, AnchorKind::Left);
        // }
    }

    fn create_chest_view(&mut self) {
        self.chest_view_id = self.id_counter + 1;
        let chest_view_rc = self.create_widget::<WidgetPanel>(Some(Rc::downgrade(&self.widgets[ROOT_ID as usize])));
        {
            let mut chest_view = chest_view_rc.borrow_mut();
            chest_view.set_border(WHITE, 2.0);
            chest_view.add_anchor_to_parent(AnchorKind::Top, AnchorKind::Top);
            chest_view.add_anchor_to_parent(AnchorKind::Bottom, AnchorKind::Bottom);
            chest_view.add_anchor(AnchorKind::Left, self.left_panel_id, AnchorKind::Right);
            chest_view.add_anchor(AnchorKind::Right, self.right_panel_id, AnchorKind::Left);
            chest_view.set_color(BLACK);
            chest_view.set_visible(false);
        }

        let parent_dyn = Rc::clone(&self.widgets[self.chest_view_id as usize]);

        let title_id = self.id_counter + 1;
        let title = self.create_widget::<WidgetText>(
            Some(Rc::downgrade(&parent_dyn))
        );
        {
            let mut lbl = title.borrow_mut();
            lbl.set_text(&"Chest".to_string());
            lbl.set_margin(QuadF::new(10.0, 30.0, 0.0, 0.0));
            lbl.add_anchor_to_parent(AnchorKind::Top, AnchorKind::Top);
            lbl.add_anchor_to_parent(AnchorKind::Left, AnchorKind::Left);
        }
    }

    // pub fn set_on_click_attr_button(&mut self, attr: &str, callback: Box<dyn FnMut(&mut Ui, PointF)>) {
    //     let button_id = match attr {
    //         "str" => self.str_area_button_id,
    //         "dex" => self.dex_area_button_id,
    //         "int" => self.int_area_button_id,
    //         _ => return, // Invalid attribute
    //     };

    //     if let Some(button) = self.widgets.get(button_id as usize) {
    //         let mut button_ref = button.borrow_mut();
    //         button_ref.set_on_click(callback);
    //     }
    // }

    pub fn draw(&mut self) {
        let ui_ref = self as &Ui;

        self.widgets[ROOT_ID as usize].borrow_mut().draw(ui_ref);
    }
}