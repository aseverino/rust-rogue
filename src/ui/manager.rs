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

use std::{
    cell::RefCell,
    collections::VecDeque,
    rc::{Rc, Weak},
    sync::Arc,
};

use macroquad::prelude::*;

use crate::{
    items::holdable::HoldableGroupKind,
    spell_type::SpellType,
    ui::{
        point_f::PointF,
        quad_f::QuadF,
        size_f::SizeF,
        widget::{AnchorKind, Widget},
        widget_bar::WidgetBar,
        widget_button::WidgetButton,
        widget_panel::WidgetPanel,
        widget_text::WidgetText,
    },
};
use std::fmt::Debug;

static ROOT_ID: u32 = 0;

pub enum AttrKind {
    Strength,
    Dexterity,
    Intelligence,
}

#[derive(Debug, Clone)]
pub enum UiEvent {
    IncStrength,
    IncDexterity,
    IncIntelligence,
    ChestAction(u32),
    SkillPurchase(u8),
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
    player_weapon: String,
    player_armor: String,
    player_shield: String,
    player_helmet: String,
    player_boots: String,

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
    weapon_value_bound_ids: Vec<u32>,
    armor_value_bound_ids: Vec<u32>,
    shield_value_bound_ids: Vec<u32>,
    helmet_value_bound_ids: Vec<u32>,
    boots_value_bound_ids: Vec<u32>,

    pub events: VecDeque<UiEvent>,

    id_counter: u32,
    pub widgets: Vec<Rc<RefCell<dyn Widget>>>,

    pub is_focused: bool,
}

impl Ui {
    pub fn new(spell_types: &Vec<Option<Arc<SpellType>>>) -> Self {
        let mut ui = Ui {
            player_hp: 1,
            player_max_hp: 1,
            player_mp: 0,
            player_max_mp: 0,
            player_sp: 0,
            player_str: 0,
            player_dex: 0,
            player_int: 0,
            player_weapon: String::new(),
            player_armor: String::new(),
            player_shield: String::new(),
            player_helmet: String::new(),
            player_boots: String::new(),
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
            weapon_value_bound_ids: Vec::new(),
            armor_value_bound_ids: Vec::new(),
            shield_value_bound_ids: Vec::new(),
            helmet_value_bound_ids: Vec::new(),
            boots_value_bound_ids: Vec::new(),

            events: VecDeque::new(),

            widgets: Vec::new(),
        };

        let root = WidgetPanel::new(&mut ui, 0, None);
        ui.widgets.push(root);

        ui.create_left_panel();
        ui.create_right_panel();
        ui.create_character_sheet(spell_types);
        ui.create_chest_view();
        ui
    }

    pub fn update_geometry(&self, resolution: SizeF) {
        self.widgets[ROOT_ID as usize]
            .borrow_mut()
            .set_size(SizeF::new(resolution.w, resolution.h));

        for widget in &self.widgets {
            // This only needs `&self`, not `&mut self`.
            // Each call to `update_drawing_coords` will write into that widget's Cell.
            widget.borrow_mut().get_drawing_coords(self);
        }
    }

    pub fn update_mouse_position(&mut self, mouse_position: PointF) {
        let widgets: Vec<_> = self.widgets.iter().cloned().collect();
        for widget in widgets {
            widget
                .borrow_mut()
                .on_mouse_position_update(self, mouse_position);
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

    pub fn set_player_weapon(&mut self, weapon: String) {
        self.player_weapon = weapon;

        for id in &self.weapon_value_bound_ids {
            if let Some(weapon_value) = self.widgets.get(*id as usize) {
                let mut text_ref = weapon_value.borrow_mut();
                if let Some(text) = text_ref.as_any_mut().downcast_mut::<WidgetText>() {
                    text.set_text(&self.player_weapon);
                }
            }
        }
    }

    pub fn set_player_armor(&mut self, armor: String) {
        self.player_armor = armor;

        for id in &self.armor_value_bound_ids {
            if let Some(armor_value) = self.widgets.get(*id as usize) {
                let mut text_ref = armor_value.borrow_mut();
                if let Some(text) = text_ref.as_any_mut().downcast_mut::<WidgetText>() {
                    text.set_text(&self.player_armor);
                }
            }
        }
    }

    pub fn set_player_shield(&mut self, shield: String) {
        self.player_shield = shield;

        for id in &self.shield_value_bound_ids {
            if let Some(shield_value) = self.widgets.get(*id as usize) {
                let mut text_ref = shield_value.borrow_mut();
                if let Some(text) = text_ref.as_any_mut().downcast_mut::<WidgetText>() {
                    text.set_text(&self.player_shield);
                }
            }
        }
    }

    pub fn set_player_helmet(&mut self, helmet: String) {
        self.player_helmet = helmet;

        for id in &self.helmet_value_bound_ids {
            if let Some(helmet_value) = self.widgets.get(*id as usize) {
                let mut text_ref = helmet_value.borrow_mut();
                if let Some(text) = text_ref.as_any_mut().downcast_mut::<WidgetText>() {
                    text.set_text(&self.player_helmet);
                }
            }
        }
    }

    pub fn set_player_boots(&mut self, boots: String) {
        self.player_boots = boots;

        for id in &self.boots_value_bound_ids {
            if let Some(boots_value) = self.widgets.get(*id as usize) {
                let mut text_ref = boots_value.borrow_mut();
                if let Some(text) = text_ref.as_any_mut().downcast_mut::<WidgetText>() {
                    text.set_text(&self.player_boots);
                }
            }
        }
    }

    pub fn set_chest_items(&mut self, items: &Vec<(u32, String)>) {
        use std::rc::Rc;

        let chest_view_rc = if let Some(chest_view) = self.widgets.get(self.chest_view_id as usize)
        {
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

        //let choice_cb = Rc::new(RefCell::new(choice_cb));

        if panel_children_len - 1 < items.len() {
            for &(item_id, ref item_name) in items.iter().skip(panel_children_len - 1) {
                let item_widget =
                    self.create_widget::<WidgetButton>(Some(Rc::downgrade(&chest_view_rc)));
                {
                    let mut item_button = item_widget.borrow_mut();
                    item_button.set_text(&format!("{}", item_name));
                    item_button.set_margin_top(10.0);
                    item_button.add_anchor_to_prev(AnchorKind::Top, AnchorKind::Bottom);
                    item_button.add_anchor_to_prev(AnchorKind::Left, AnchorKind::Left);
                    item_button.add_anchor_to_parent(AnchorKind::Right, AnchorKind::Right);

                    item_button.set_on_click(Box::new(move |ui, _, _| {
                        ui.events.push_back(UiEvent::ChestAction(item_id));
                    }));
                }
            }
        }

        // Update existing items
        {
            let mut chest_ref = chest_view_rc.borrow_mut();
            if let Some(panel) = chest_ref.as_any_mut().downcast_mut::<WidgetPanel>() {
                let panel_children = panel.get_children();

                for (index, (item_id, item_name)) in items.iter().enumerate() {
                    if let Some(child_rc) = panel_children.get(index + 1).and_then(|c| c.upgrade())
                    {
                        let mut child_ref = child_rc.borrow_mut();
                        if let Some(item_button) =
                            child_ref.as_any_mut().downcast_mut::<WidgetButton>()
                        {
                            item_button.set_visible(true);
                            item_button.set_text(&format!("{}", item_name));

                            let item_id = *item_id;
                            item_button.set_on_click(Box::new(move |ui, _, _| {
                                ui.events.push_back(UiEvent::ChestAction(item_id));
                            }));
                        }
                    }
                }
            }
        }

        if panel_children_len > items.len() {
            // Remove excess items
            let mut chest_ref = chest_view_rc.borrow_mut();
            if let Some(panel) = chest_ref.as_any_mut().downcast_mut::<WidgetPanel>() {
                let panel_children = panel.get_children();
                for child in panel_children.iter().skip(items.len() + 1) {
                    if let Some(child_rc) = child.upgrade() {
                        child_rc.borrow_mut().set_visible(false);
                    }
                }
            }
        }
    }

    pub fn toggle_character_sheet(&mut self) {
        let is_visible = self.widgets[self.character_sheet_id as usize]
            .borrow()
            .is_visible();
        let mut cs = self.widgets[self.character_sheet_id as usize].borrow_mut();
        cs.set_visible(!is_visible);
        self.is_focused = !is_visible;
    }

    pub fn show_chest_view(&mut self, items: &Vec<(u32, String)>) {
        self.set_chest_items(items);
        self.widgets[self.chest_view_id as usize]
            .borrow_mut()
            .set_visible(true);
        self.is_focused = true;
    }

    pub fn hide(&mut self) {
        self.widgets[self.character_sheet_id as usize]
            .borrow_mut()
            .set_visible(false);
        self.widgets[self.chest_view_id as usize]
            .borrow_mut()
            .set_visible(false);
        self.is_focused = false;
    }

    // pub fn add_widget(&mut self, widget: Rc<RefCell<dyn Widget>>) {
    //     self.id_counter += 1;
    //     self.widgets.push(widget);
    // }

    pub fn create_widget<T: Widget + 'static>(
        &mut self,
        parent: Option<Weak<RefCell<dyn Widget>>>,
    ) -> Rc<RefCell<T>> {
        self.id_counter += 1;
        let widget = T::new(self, self.id_counter, parent);
        let widget_dyn: Rc<RefCell<dyn Widget>> = widget.clone();

        if !widget.borrow().is_manually_added() {
            self.widgets.push(widget_dyn);
        }

        if self.widgets.len() != self.id_counter as usize + 1 {
            panic!(
                "Widget ID mismatch: expected {}, got {}",
                self.id_counter,
                self.widgets.len() - 1
            );
        }

        widget
    }

    fn create_skill_button(
        &mut self,
        event: UiEvent,
        skill_name: &str,
        sp_cost: u32,
        parent: &Rc<RefCell<dyn Widget>>,
        first: bool,
    ) -> (Rc<RefCell<dyn Widget>>, u32) {
        let button = self.create_widget::<WidgetButton>(Some(Rc::downgrade(parent)));
        {
            let mut attr_button = button.borrow_mut();
            attr_button.set_on_click(Box::new(move |ui, _, _| {
                ui.events.push_back(event.clone());
            }));
            //attr_panel.set_border(WHITE, 1.0);
            attr_button.set_size(SizeF::new(150.0, 50.0));
            // attr_button.add_anchor_to_parent(AnchorKind::Top, AnchorKind::Top);
            // attr_button.add_anchor_to_parent(AnchorKind::Left, AnchorKind::Left);
            // attr_button.set_margin_top(margin_top);

            if first {
                attr_button.add_anchor_to_parent(AnchorKind::Top, AnchorKind::Top);
            } else {
                attr_button.add_anchor_to_prev(AnchorKind::Top, AnchorKind::Bottom);
            }
            attr_button.add_anchor_to_parent(AnchorKind::Left, AnchorKind::Left);
            attr_button.add_anchor_to_parent(AnchorKind::Right, AnchorKind::Right);
        }

        let label_widget = self.create_widget::<WidgetText>(Some(Rc::downgrade(
            &(button.clone() as Rc<RefCell<dyn Widget>>),
        )));
        {
            let mut lbl = label_widget.borrow_mut();
            lbl.set_text(&skill_name.to_string());
            lbl.set_margin_left(30.0);
            lbl.add_anchor_to_parent(AnchorKind::VerticalCenter, AnchorKind::VerticalCenter);
            lbl.add_anchor_to_parent(AnchorKind::Left, AnchorKind::Left);
        }

        let value_id = self.id_counter + 1;
        let value_widget = self.create_widget::<WidgetText>(Some(Rc::downgrade(
            &(button.clone() as Rc<RefCell<dyn Widget>>),
        )));
        {
            let mut val = value_widget.borrow_mut();
            val.set_text(&sp_cost.to_string());
            val.set_margin_right(30.0);
            val.add_anchor_to_prev(AnchorKind::Top, AnchorKind::Top);
            val.add_anchor_to_parent(AnchorKind::Right, AnchorKind::Right);
        }

        (button, value_id)
    }

    fn create_attr_button(
        &mut self,
        event: UiEvent,
        label: &str,
        value: u32,
        parent: &Rc<RefCell<dyn Widget>>,
        margin_top: f32,
    ) -> (Rc<RefCell<dyn Widget>>, u32) {
        let button = self.create_widget::<WidgetButton>(Some(Rc::downgrade(parent)));
        {
            let mut attr_button = button.borrow_mut();
            attr_button.set_on_click(Box::new(move |ui, _, _| {
                ui.events.push_back(event.clone());
            }));
            //attr_panel.set_border(WHITE, 1.0);
            attr_button.set_size(SizeF::new(150.0, 50.0));
            attr_button.add_anchor_to_parent(AnchorKind::Top, AnchorKind::Top);
            attr_button.add_anchor_to_parent(AnchorKind::Left, AnchorKind::Left);
            attr_button.set_margin_top(margin_top);
        }

        let label_widget = self.create_widget::<WidgetText>(Some(Rc::downgrade(
            &(button.clone() as Rc<RefCell<dyn Widget>>),
        )));
        {
            let mut lbl = label_widget.borrow_mut();
            lbl.set_text(&label.to_string());
            lbl.set_margin_left(30.0);
            lbl.add_anchor_to_parent(AnchorKind::VerticalCenter, AnchorKind::VerticalCenter);
            lbl.add_anchor_to_parent(AnchorKind::Left, AnchorKind::Left);
        }

        let value_id = self.id_counter + 1;
        let value_widget = self.create_widget::<WidgetText>(Some(Rc::downgrade(
            &(button.clone() as Rc<RefCell<dyn Widget>>),
        )));
        {
            let mut val = value_widget.borrow_mut();
            val.set_text(&value.to_string());
            val.set_margin_right(30.0);
            val.add_anchor_to_prev(AnchorKind::Top, AnchorKind::Top);
            val.add_anchor_to_parent(AnchorKind::Right, AnchorKind::Right);
        }

        (button, value_id)
    }

    fn create_attr_label(&mut self, attr: AttrKind, parent: &Rc<RefCell<dyn Widget>>) {
        let str_label = self.create_widget::<WidgetText>(Some(Rc::downgrade(&parent)));
        let str_value = self.create_widget::<WidgetText>(Some(Rc::downgrade(&parent)));

        let (label, value_now, binding) = match attr {
            AttrKind::Strength => ("STR", self.player_str, &mut self.str_value_bound_ids),
            AttrKind::Dexterity => ("DEX", self.player_dex, &mut self.dex_value_bound_ids),
            AttrKind::Intelligence => ("INT", self.player_int, &mut self.int_value_bound_ids),
        };

        {
            let mut lbl = str_label.borrow_mut();
            lbl.set_text(&label.to_string());
            lbl.set_color(GREEN);
            lbl.set_margin_top(10.0);
            lbl.base.size = SizeF::new(50.0, lbl.base.size.h);
            lbl.add_anchor_to_prev(AnchorKind::Top, AnchorKind::Bottom);
            lbl.add_anchor(AnchorKind::Left, self.sp_value_id - 1, AnchorKind::Left);
        }

        binding.push(self.id_counter);

        {
            let mut lbl = str_value.borrow_mut();
            lbl.set_color(GREEN);
            lbl.add_anchor_to_prev(AnchorKind::Top, AnchorKind::Top);
            lbl.add_anchor_to_prev(AnchorKind::Left, AnchorKind::Right);
            lbl.set_text(&format!("{}", value_now));
        }
    }

    fn create_equip_label(
        &mut self,
        anchor_left_id: u32,
        group: HoldableGroupKind,
        parent: &Rc<RefCell<dyn Widget>>,
    ) {
        let str_label = self.create_widget::<WidgetText>(Some(Rc::downgrade(&parent)));
        let str_value = self.create_widget::<WidgetText>(Some(Rc::downgrade(&parent)));

        let (label, value_now, binding) = match group {
            HoldableGroupKind::Weapons => {
                ("Wpn", &self.player_weapon, &mut self.weapon_value_bound_ids)
            }
            HoldableGroupKind::Armor => {
                ("Arm", &self.player_armor, &mut self.armor_value_bound_ids)
            }
            HoldableGroupKind::Shields => {
                ("Shd", &self.player_shield, &mut self.shield_value_bound_ids)
            }
            HoldableGroupKind::Helmets => {
                ("Hel", &self.player_helmet, &mut self.helmet_value_bound_ids)
            }
            HoldableGroupKind::Boots => {
                ("Bts", &self.player_boots, &mut self.boots_value_bound_ids)
            }
        };

        {
            let mut lbl = str_label.borrow_mut();
            lbl.set_text(&label.to_string());
            lbl.set_color(GREEN);
            lbl.set_margin_top(10.0);
            lbl.base.size = SizeF::new(50.0, 20.0);
            lbl.add_anchor_to_prev(AnchorKind::Top, AnchorKind::Bottom);
            lbl.add_anchor(AnchorKind::Left, anchor_left_id, AnchorKind::Left);
        }

        binding.push(self.id_counter);

        {
            let mut lbl = str_value.borrow_mut();
            lbl.set_color(GREEN);
            lbl.add_anchor_to_prev(AnchorKind::Top, AnchorKind::Top);
            lbl.add_anchor_to_prev(AnchorKind::Left, AnchorKind::Right);
            lbl.set_text(value_now);
            lbl.base.size = SizeF::new(50.0, 20.0);
        }
    }

    fn create_left_panel(&mut self) {
        self.left_panel_id = self.id_counter + 1;
        let left_panel_rc =
            self.create_widget::<WidgetPanel>(Some(Rc::downgrade(&self.widgets[ROOT_ID as usize])));

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
        let hp_label = self.create_widget::<WidgetText>(Some(Rc::downgrade(&parent_dyn)));
        {
            let mut lbl = hp_label.borrow_mut();
            lbl.set_text(&"HP".to_string());
            lbl.set_color(Color {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            });
            lbl.set_margin(QuadF::new(10.0, 30.0, 0.0, 0.0));
            lbl.add_anchor_to_parent(AnchorKind::Top, AnchorKind::Top);
            lbl.add_anchor_to_parent(AnchorKind::Left, AnchorKind::Left);
        }

        self.hp_bar_id = self.id_counter + 1;
        let hp_bar = self.create_widget::<WidgetBar>(Some(Rc::downgrade(&parent_dyn)));
        {
            let mut bar = hp_bar.borrow_mut();
            bar.set_size(SizeF::new(200.0, 20.0));
            bar.add_anchor_to_prev(AnchorKind::Top, AnchorKind::Top);
            bar.add_anchor_to_prev(AnchorKind::Left, AnchorKind::Right);
            bar.set_text(&format!("{}/{}", self.player_hp, self.player_max_hp));
            bar.set_background_color(Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            });
            bar.set_bar_color(Color {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            });

            if self.player_max_hp > 0 {
                bar.set_bar_percentage(self.player_hp as f32 / self.player_max_hp as f32);
            }
        }

        let mp_label = self.create_widget::<WidgetText>(Some(Rc::downgrade(&parent_dyn)));
        {
            let mut lbl = mp_label.borrow_mut();
            lbl.set_text(&"MP".to_string());
            lbl.set_color(BLUE);
            lbl.set_margin_top(10.0);
            lbl.add_anchor(
                AnchorKind::Top,
                hp_label.borrow().get_id(),
                AnchorKind::Bottom,
            );
            lbl.add_anchor(
                AnchorKind::Left,
                hp_label.borrow().get_id(),
                AnchorKind::Left,
            );
        }

        self.mp_bar_id = self.id_counter + 1;
        let mp_bar = self.create_widget::<WidgetBar>(Some(Rc::downgrade(&parent_dyn)));
        {
            let mut bar = mp_bar.borrow_mut();
            bar.set_size(SizeF::new(200.0, 20.0));
            bar.add_anchor_to_prev(AnchorKind::Top, AnchorKind::Top);
            bar.add_anchor_to_prev(AnchorKind::Left, AnchorKind::Right);
            bar.set_text(&format!("{}/{}", self.player_mp, self.player_max_mp));
            bar.set_background_color(Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            });
            bar.set_bar_color(Color {
                r: 0.0,
                g: 0.0,
                b: 1.0,
                a: 1.0,
            });

            if self.player_max_mp > 0 {
                bar.set_bar_percentage(self.player_mp as f32 / self.player_max_mp as f32);
            }
        }

        let soul_label = self.create_widget::<WidgetText>(Some(Rc::downgrade(&parent_dyn)));
        {
            let mut lbl = soul_label.borrow_mut();
            lbl.set_text(&"SP".to_string());
            lbl.set_color(YELLOW);
            lbl.set_margin_top(10.0);
            lbl.add_anchor(
                AnchorKind::Top,
                mp_label.borrow().get_id(),
                AnchorKind::Bottom,
            );
            lbl.add_anchor(
                AnchorKind::Left,
                mp_label.borrow().get_id(),
                AnchorKind::Left,
            );
        }

        self.sp_value_id = self.id_counter + 1;
        let soul_value = self.create_widget::<WidgetText>(Some(Rc::downgrade(&parent_dyn)));
        {
            let mut lbl = soul_value.borrow_mut();
            lbl.set_color(YELLOW);
            lbl.add_anchor_to_prev(AnchorKind::Top, AnchorKind::Top);
            lbl.add_anchor_to_prev(AnchorKind::Left, AnchorKind::Right);
            lbl.set_text(&format!("{}", self.player_sp));
        }

        for i in 0..3 {
            let attr_kind = match i {
                0 => AttrKind::Strength,
                1 => AttrKind::Dexterity,
                2 => AttrKind::Intelligence,
                _ => continue,
            };
            self.create_attr_label(attr_kind, &parent_dyn);
        }

        for i in 0..5 {
            let group = match i {
                0 => HoldableGroupKind::Weapons,
                1 => HoldableGroupKind::Armor,
                2 => HoldableGroupKind::Shields,
                3 => HoldableGroupKind::Helmets,
                4 => HoldableGroupKind::Boots,
                _ => continue,
            };
            self.create_equip_label(self.id_counter - 1, group, &parent_dyn);
        }

        //self.create_attr_label("DEX", dex_ids, self.player_dex, &parent_dyn);
        //self.create_attr_label("INT", dex_ids, self.player_int, &parent_dyn);

        // let str_label = self.create_widget::<WidgetText>(Some(Rc::downgrade(&parent_dyn)));
        // {
        //     let mut lbl = str_label.borrow_mut();
        //     lbl.set_text(&"STR".to_string());
        //     lbl.set_color(GREEN);
        //     lbl.set_margin_top(10.0);
        //     lbl.add_anchor(
        //         AnchorKind::Top,
        //         soul_label.borrow().get_id(),
        //         AnchorKind::Bottom,
        //     );
        //     lbl.add_anchor(
        //         AnchorKind::Left,
        //         soul_label.borrow().get_id(),
        //         AnchorKind::Left,
        //     );
        // }

        // self.str_value_bound_ids.push(self.id_counter + 1);
        // let str_value = self.create_widget::<WidgetText>(Some(Rc::downgrade(&parent_dyn)));
        // {
        //     let mut lbl = str_value.borrow_mut();
        //     lbl.set_color(GREEN);
        //     lbl.add_anchor_to_prev(AnchorKind::Top, AnchorKind::Top);
        //     lbl.add_anchor_to_prev(AnchorKind::Left, AnchorKind::Right);
        //     lbl.set_text(&format!("{}", self.player_str));
        // }
    }

    fn create_right_panel(&mut self) {
        self.right_panel_id = self.id_counter + 1;
        let right_panel_rc =
            self.create_widget::<WidgetPanel>(Some(Rc::downgrade(&self.widgets[ROOT_ID as usize])));
        {
            let mut right_panel = right_panel_rc.borrow_mut();
            right_panel.set_size(SizeF::new(400.0, 0.0));
            right_panel.set_border(WHITE, 2.0);
            right_panel.add_anchor_to_parent(AnchorKind::Top, AnchorKind::Top);
            right_panel.add_anchor_to_parent(AnchorKind::Bottom, AnchorKind::Bottom);
            right_panel.add_anchor_to_parent(AnchorKind::Right, AnchorKind::Right);
        }
    }

    fn create_character_sheet(&mut self, spell_types: &Vec<Option<Arc<SpellType>>>) {
        self.character_sheet_id = self.id_counter + 1;
        let character_sheet_rc =
            self.create_widget::<WidgetPanel>(Some(Rc::downgrade(&self.widgets[ROOT_ID as usize])));
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
        let attributes_tab = self.create_widget::<WidgetButton>(Some(Rc::downgrade(&parent_dyn)));
        {
            let mut btn = attributes_tab.borrow_mut();
            btn.toggled = true;
            btn.set_text(&"Attrib".to_string());
            btn.set_margin(QuadF::new(10.0, 30.0, 0.0, 0.0));
            btn.add_anchor_to_parent(AnchorKind::Top, AnchorKind::Top);
            btn.add_anchor_to_parent(AnchorKind::Left, AnchorKind::Left);
        }

        let skills_tab_id = self.id_counter + 1;
        let skills_tab = self.create_widget::<WidgetButton>(Some(Rc::downgrade(&parent_dyn)));
        {
            let mut btn = skills_tab.borrow_mut();
            btn.set_text(&"Skills".to_string());
            btn.set_margin(QuadF::new(30.0, 0.0, 0.0, 0.0));
            btn.add_anchor_to_prev(AnchorKind::Top, AnchorKind::Top);
            btn.add_anchor_to_prev(AnchorKind::Left, AnchorKind::Right);
        }

        let abilities_tab_id = self.id_counter + 1;
        let abilities_tab = self.create_widget::<WidgetButton>(Some(Rc::downgrade(&parent_dyn)));
        {
            let mut lbl = abilities_tab.borrow_mut();
            lbl.set_text(&"Abilit".to_string());
            lbl.set_margin(QuadF::new(30.0, 0.0, 0.0, 0.0));
            lbl.add_anchor_to_prev(AnchorKind::Top, AnchorKind::Top);
            lbl.add_anchor_to_prev(AnchorKind::Left, AnchorKind::Right);
        }

        let equipment_tab_id = self.id_counter + 1;
        let equipment_tab = self.create_widget::<WidgetButton>(Some(Rc::downgrade(&parent_dyn)));
        {
            let mut lbl = equipment_tab.borrow_mut();
            lbl.set_text(&"Equip".to_string());
            lbl.set_margin(QuadF::new(30.0, 0.0, 0.0, 0.0));
            lbl.add_anchor_to_prev(AnchorKind::Top, AnchorKind::Top);
            lbl.add_anchor_to_prev(AnchorKind::Left, AnchorKind::Right);
        }

        let inventory_tab_id = self.id_counter + 1;
        let inventory_tab = self.create_widget::<WidgetButton>(Some(Rc::downgrade(&parent_dyn)));
        {
            let mut lbl = inventory_tab.borrow_mut();
            lbl.set_text(&"Invent".to_string());
            lbl.set_margin(QuadF::new(30.0, 0.0, 0.0, 0.0));
            lbl.add_anchor_to_prev(AnchorKind::Top, AnchorKind::Top);
            lbl.add_anchor_to_prev(AnchorKind::Left, AnchorKind::Right);
        }

        let attributes_sheet_id = self.id_counter + 1;
        let attributes_sheet_rc =
            self.create_widget::<WidgetPanel>(Some(Rc::downgrade(&parent_dyn)));
        {
            let mut attr_sheet = attributes_sheet_rc.borrow_mut();
            attr_sheet.set_border(WHITE, 2.0);
            attr_sheet.add_anchor(AnchorKind::Top, attributes_tab_id, AnchorKind::Bottom);
            attr_sheet.add_anchor_to_parent(AnchorKind::Left, AnchorKind::Left);
            attr_sheet.add_anchor_to_parent(AnchorKind::Right, AnchorKind::Right);
            attr_sheet.add_anchor_to_parent(AnchorKind::Bottom, AnchorKind::Bottom);
        }

        let skills_sheet_id = self.id_counter + 1;
        let skills_sheet_rc = self.create_widget::<WidgetPanel>(Some(Rc::downgrade(&parent_dyn)));
        {
            let mut skills_sheet = skills_sheet_rc.borrow_mut();
            skills_sheet.set_border(WHITE, 2.0);
            skills_sheet.add_anchor(AnchorKind::Top, skills_tab_id, AnchorKind::Bottom);
            skills_sheet.add_anchor_to_parent(AnchorKind::Left, AnchorKind::Left);
            skills_sheet.add_anchor_to_parent(AnchorKind::Right, AnchorKind::Right);
            skills_sheet.add_anchor_to_parent(AnchorKind::Bottom, AnchorKind::Bottom);
            skills_sheet.set_visible(false);
        }

        let abilities_sheet_id = self.id_counter + 1;
        let abilities_sheet_rc =
            self.create_widget::<WidgetPanel>(Some(Rc::downgrade(&parent_dyn)));
        {
            let mut abilities_sheet = abilities_sheet_rc.borrow_mut();
            abilities_sheet.set_border(WHITE, 2.0);
            abilities_sheet.add_anchor(AnchorKind::Top, abilities_tab_id, AnchorKind::Bottom);
            abilities_sheet.add_anchor_to_parent(AnchorKind::Left, AnchorKind::Left);
            abilities_sheet.add_anchor_to_parent(AnchorKind::Right, AnchorKind::Right);
            abilities_sheet.add_anchor_to_parent(AnchorKind::Bottom, AnchorKind::Bottom);
            abilities_sheet.set_visible(false);
        }

        let equipment_sheet_id = self.id_counter + 1;
        let equipment_sheet_rc =
            self.create_widget::<WidgetPanel>(Some(Rc::downgrade(&parent_dyn)));
        {
            let mut equipment_sheet = equipment_sheet_rc.borrow_mut();
            equipment_sheet.set_border(WHITE, 2.0);
            equipment_sheet.add_anchor(AnchorKind::Top, equipment_tab_id, AnchorKind::Bottom);
            equipment_sheet.add_anchor_to_parent(AnchorKind::Left, AnchorKind::Left);
            equipment_sheet.add_anchor_to_parent(AnchorKind::Right, AnchorKind::Right);
            equipment_sheet.add_anchor_to_parent(AnchorKind::Bottom, AnchorKind::Bottom);
            equipment_sheet.set_visible(false);
        }

        let inventory_sheet_id = self.id_counter + 1;
        let inventory_sheet_rc =
            self.create_widget::<WidgetPanel>(Some(Rc::downgrade(&parent_dyn)));
        {
            let mut inventory_sheet = inventory_sheet_rc.borrow_mut();
            inventory_sheet.set_border(WHITE, 2.0);
            inventory_sheet.add_anchor(AnchorKind::Top, inventory_tab_id, AnchorKind::Bottom);
            inventory_sheet.add_anchor_to_parent(AnchorKind::Left, AnchorKind::Left);
            inventory_sheet.add_anchor_to_parent(AnchorKind::Right, AnchorKind::Right);
            inventory_sheet.add_anchor_to_parent(AnchorKind::Bottom, AnchorKind::Bottom);
            inventory_sheet.set_visible(false);
        }

        attributes_tab
            .borrow_mut()
            .set_on_click(Box::new(move |ui, this, _| {
                ui.widgets[skills_sheet_id as usize]
                    .borrow_mut()
                    .set_visible(false);
                if let Some(btn) = ui.widgets[skills_tab_id as usize]
                    .borrow_mut()
                    .as_button_mut()
                {
                    btn.toggled = false;
                }
                ui.widgets[abilities_sheet_id as usize]
                    .borrow_mut()
                    .set_visible(false);
                if let Some(btn) = ui.widgets[abilities_tab_id as usize]
                    .borrow_mut()
                    .as_button_mut()
                {
                    btn.toggled = false;
                }
                ui.widgets[equipment_sheet_id as usize]
                    .borrow_mut()
                    .set_visible(false);
                if let Some(btn) = ui.widgets[equipment_tab_id as usize]
                    .borrow_mut()
                    .as_button_mut()
                {
                    btn.toggled = false;
                }
                ui.widgets[inventory_sheet_id as usize]
                    .borrow_mut()
                    .set_visible(false);
                if let Some(btn) = ui.widgets[inventory_tab_id as usize]
                    .borrow_mut()
                    .as_button_mut()
                {
                    btn.toggled = false;
                }
                ui.widgets[attributes_sheet_id as usize]
                    .borrow_mut()
                    .set_visible(true);
                this.toggled = true;
            }));

        skills_tab
            .borrow_mut()
            .set_on_click(Box::new(move |ui, this, _| {
                ui.widgets[attributes_sheet_id as usize]
                    .borrow_mut()
                    .set_visible(false);
                if let Some(btn) = ui.widgets[attributes_tab_id as usize]
                    .borrow_mut()
                    .as_button_mut()
                {
                    btn.toggled = false;
                }
                ui.widgets[abilities_sheet_id as usize]
                    .borrow_mut()
                    .set_visible(false);
                if let Some(btn) = ui.widgets[abilities_tab_id as usize]
                    .borrow_mut()
                    .as_button_mut()
                {
                    btn.toggled = false;
                }
                ui.widgets[equipment_sheet_id as usize]
                    .borrow_mut()
                    .set_visible(false);
                if let Some(btn) = ui.widgets[equipment_tab_id as usize]
                    .borrow_mut()
                    .as_button_mut()
                {
                    btn.toggled = false;
                }
                ui.widgets[inventory_sheet_id as usize]
                    .borrow_mut()
                    .set_visible(false);
                if let Some(btn) = ui.widgets[inventory_tab_id as usize]
                    .borrow_mut()
                    .as_button_mut()
                {
                    btn.toggled = false;
                }
                ui.widgets[skills_sheet_id as usize]
                    .borrow_mut()
                    .set_visible(true);
                this.toggled = true;
            }));

        let attr_as_parent_dyn = Rc::clone(&self.widgets[attributes_sheet_id as usize]);

        let (dex_area, dex_value_id) = self.create_attr_button(
            UiEvent::IncDexterity,
            "DEX",
            self.player_dex,
            &attr_as_parent_dyn,
            10.0,
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
            0.0,
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
            0.0,
        );

        {
            let mut int_area_button = int_area.borrow_mut();
            int_area_button.break_anchors();
            int_area_button.add_anchor(
                AnchorKind::Top,
                dex_area.borrow().get_id(),
                AnchorKind::Top,
            );
            int_area_button.add_anchor(
                AnchorKind::Left,
                dex_area.borrow().get_id(),
                AnchorKind::Right,
            );
            self.int_area_button_id = int_area_button.get_id();
        }
        self.int_value_bound_ids.push(int_value_id);

        let skills_as_parent_dyn = Rc::clone(&self.widgets[skills_sheet_id as usize]);

        for (index, spell_type) in spell_types.iter().enumerate() {
            if let Some(spell_type) = spell_type {
                let (spell_id, spell_cost) = { (spell_type.index, spell_type.cost) };
                self.create_skill_button(
                    UiEvent::SkillPurchase(spell_id as u8),
                    &spell_type.name,
                    spell_cost,
                    &skills_as_parent_dyn,
                    index == 0,
                );
                // let skill_button_id = self.id_counter + 1;
                // let skill_button =
                //     self.create_widget::<WidgetButton>(Some(Rc::downgrade(&skills_as_parent_dyn)));
                // {
                //     let mut btn = skill_button.borrow_mut();
                //     btn.set_text(&spell_type.name);
                //     btn.set_margin_top(10.0);
                //     if index == 0 {
                //         btn.add_anchor_to_parent(AnchorKind::Top, AnchorKind::Top);
                //     } else {
                //         btn.add_anchor_to_prev(AnchorKind::Top, AnchorKind::Bottom);
                //     }
                //     btn.add_anchor_to_parent(AnchorKind::Left, AnchorKind::Left);
                //     btn.add_anchor_to_parent(AnchorKind::Right, AnchorKind::Right);
                //     // btn.set_on_click(Box::new(move |ui, _, _| {
                //     //     ui.events.push_back(UiEvent::CastSpell(spell_type.id));
                //     // }));
                // }
            }
        }
    }

    fn create_chest_view(&mut self) {
        self.chest_view_id = self.id_counter + 1;
        let chest_view_rc =
            self.create_widget::<WidgetPanel>(Some(Rc::downgrade(&self.widgets[ROOT_ID as usize])));
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
        let title = self.create_widget::<WidgetText>(Some(Rc::downgrade(&parent_dyn)));
        {
            let mut lbl = title.borrow_mut();
            lbl.set_text(&"Chest".to_string());
            lbl.set_margin(QuadF::new(10.0, 30.0, 0.0, 0.0));
            lbl.add_anchor_to_parent(AnchorKind::Top, AnchorKind::Top);
            lbl.add_anchor_to_parent(AnchorKind::Left, AnchorKind::Left);
        }
    }

    pub fn draw(&mut self) {
        let ui_ref = self as &Ui;

        self.widgets[ROOT_ID as usize].borrow_mut().draw(ui_ref);
    }
}
