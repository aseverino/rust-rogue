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

use crate::{position::Direction, ui::point_f::PointF};
use macroquad::prelude::*;
use once_cell::sync::Lazy;
use std::sync::Mutex;

#[derive(Clone, PartialEq)]
pub enum KeyboardAction {
    None,
    Move,
    Wait,
    Cancel,
    Confirm,
    SpellSelect,
    AttackChooseTarget,
    AttackConfirm,
    OpenCharacterSheet,
}

pub struct Input {
    keyboard_action: KeyboardAction,
    direction_intention: Direction,
    spell_action: i32,
    mouse_position: PointF,
    clicked_position: Option<PointF>,

    mouse_press_position: Option<PointF>,
}

pub struct InputSnapshot {
    pub keyboard_action: KeyboardAction,
    pub direction: Direction,
    pub spell: i32,
    pub click: Option<PointF>,
    pub mouse: PointF,
}

impl Input {
    fn handle_keyboard_input(&mut self) {
        let mut keyboard_action = KeyboardAction::None;
        let mut direction = Direction::None;
        let mut spell_action = 0;

        if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::Kp6) {
            keyboard_action = KeyboardAction::Move;
            direction = Direction::Right;
        }
        if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::Kp4) {
            keyboard_action = KeyboardAction::Move;
            direction = Direction::Left;
        }
        if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::Kp8) {
            keyboard_action = KeyboardAction::Move;
            direction = Direction::Up;
        }
        if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::Kp2) {
            keyboard_action = KeyboardAction::Move;
            direction = Direction::Down;
        }
        if is_key_pressed(KeyCode::Kp7) {
            keyboard_action = KeyboardAction::Move;
            direction = Direction::UpLeft;
        }
        if is_key_pressed(KeyCode::Kp9) {
            keyboard_action = KeyboardAction::Move;
            direction = Direction::UpRight;
        }
        if is_key_pressed(KeyCode::Kp1) {
            keyboard_action = KeyboardAction::Move;
            direction = Direction::DownLeft;
        }
        if is_key_pressed(KeyCode::Kp3) {
            keyboard_action = KeyboardAction::Move;
            direction = Direction::DownRight;
        }
        if is_key_pressed(KeyCode::Kp5) {
            keyboard_action = KeyboardAction::Wait;
        }
        if is_key_pressed(KeyCode::Escape) {
            keyboard_action = KeyboardAction::Cancel;
        }
        if is_key_pressed(KeyCode::Enter) {
            keyboard_action = KeyboardAction::Confirm;
        }
        if is_key_pressed(KeyCode::Key1) {
            keyboard_action = KeyboardAction::SpellSelect;
            spell_action = 1;
        }
        if is_key_pressed(KeyCode::A) {
            keyboard_action = KeyboardAction::AttackChooseTarget;
        }
        if is_key_pressed(KeyCode::C) {
            keyboard_action = KeyboardAction::OpenCharacterSheet;
        }

        self.keyboard_action = keyboard_action;
        self.direction_intention = direction;
        self.spell_action = spell_action;
    }

    fn handle_mouse_input(&mut self) {
        let mouse_pos_tuple = mouse_position();
        self.mouse_position = PointF::new(mouse_pos_tuple.0, mouse_pos_tuple.1);
        if is_mouse_button_pressed(MouseButton::Left) {
            self.mouse_press_position = Some(self.mouse_position);
        }
        if is_mouse_button_released(MouseButton::Left) {
            if let Some(press_pos) = self.mouse_press_position.take() {
                if (self.mouse_position.x - press_pos.x).abs() < 5.0
                    && (self.mouse_position.y - press_pos.y).abs() < 5.0
                {
                    self.clicked_position = Some(press_pos);
                } else {
                    self.clicked_position = None;
                }
            }
        }
    }

    pub fn poll() -> InputSnapshot {
        let mut input = INPUT.lock().unwrap();
        input.handle_keyboard_input();
        input.handle_mouse_input();
        InputSnapshot {
            keyboard_action: input.keyboard_action.clone(),
            direction: input.direction_intention.clone(),
            spell: input.spell_action,
            click: input.clicked_position.take(), // consumes click for this frame
            mouse: input.mouse_position,
        }
    }
}

static INPUT: Lazy<Mutex<Input>> = Lazy::new(|| {
    Mutex::new(Input {
        keyboard_action: KeyboardAction::None,
        direction_intention: Direction::None,
        spell_action: 0,
        mouse_position: PointF::new(0.0, 0.0),
        clicked_position: None,
        mouse_press_position: None,
    })
});
