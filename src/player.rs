// SPDX-License-Identifier: MIT
//
// Copyright (c) 2025 Your Name
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

use macroquad::prelude::*;
use crate::map::{TILE_SIZE, GRID_WIDTH, GRID_HEIGHT, Map};
use crate::creature::Creature;
use crate::position::Position;
use crate::map::Direction;

#[derive(PartialEq)]
pub enum KeyboardAction {
    None,
    Move,
    Wait,
    Cancel
}

pub struct Player {
    pub name: String,
    pub position: Position,
    pub keyboard_action: KeyboardAction,
    pub goal_position: Option<Position>,
}

impl Player {
    pub fn new(x: usize, y: usize) -> Self {
        Self {
            name: "Player".into(),
            position: Position {
                x,
                y,
            },
            keyboard_action: KeyboardAction::None,
            goal_position: None,
        }
    }

    pub fn handle_input(&self, map: &Map) -> (KeyboardAction, Direction) {
        let mut keyboard_action = KeyboardAction::None;
        let mut direction = Direction::None;

        if (is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::Kp6)) && self.position.x < GRID_WIDTH - 1 && map.is_walkable(self.position.x + 1, self.position.y) {
            keyboard_action = KeyboardAction::Move;
            direction = Direction::Right;
        }
        if (is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::Kp4)) && self.position.x > 0 && map.is_walkable(self.position.x - 1, self.position.y) {
            keyboard_action = KeyboardAction::Move;
            direction = Direction::Left;
        }
        if (is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::Kp8)) && self.position.y > 0 && map.is_walkable(self.position.x, self.position.y - 1) {
            keyboard_action = KeyboardAction::Move;
            direction = Direction::Up;
        }
        if (is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::Kp2)) && self.position.y < GRID_HEIGHT - 1 && map.is_walkable(self.position.x, self.position.y + 1) {
            keyboard_action = KeyboardAction::Move;
            direction = Direction::Down;
        }
        if is_key_pressed(KeyCode::Kp7) && self.position.x > 0 && self.position.y > 0 && map.is_walkable(self.position.x - 1, self.position.y - 1) {
            keyboard_action = KeyboardAction::Move;
            direction = Direction::UpLeft;
        }
        if is_key_pressed(KeyCode::Kp9) && self.position.x < GRID_WIDTH - 1 && self.position.y > 0 && map.is_walkable(self.position.x + 1, self.position.y - 1) {
            keyboard_action = KeyboardAction::Move;
            direction = Direction::UpRight;
        }
        if is_key_pressed(KeyCode::Kp1) && self.position.x > 0 && self.position.y < GRID_HEIGHT - 1 && map.is_walkable(self.position.x - 1, self.position.y + 1) {
            keyboard_action = KeyboardAction::Move;
            direction = Direction::DownLeft;
        }
        if is_key_pressed(KeyCode::Kp3) && self.position.x < GRID_WIDTH - 1 && self.position.y < GRID_HEIGHT - 1 && map.is_walkable(self.position.x + 1, self.position.y + 1) {
            keyboard_action = KeyboardAction::Move;
            direction = Direction::DownRight;
        }
        if is_key_pressed(KeyCode::Kp5) {
            keyboard_action = KeyboardAction::Wait;
        }
        if is_key_pressed(KeyCode::Escape) {
            keyboard_action = KeyboardAction::Cancel;
        }

        (keyboard_action, direction)
    }
}

impl Creature for Player {
    fn name(&self) -> &str {
        &self.name
    }

    fn pos(&self) -> Position {
        self.position
    }

    fn set_pos(&mut self, pos: Position) {
        self.position = pos;
    }

    fn draw(&self) {
        // Base colored rectangle
        draw_rectangle(
            self.position.x as f32 * TILE_SIZE + 4.0,
            self.position.y as f32 * TILE_SIZE + 44.0,
            TILE_SIZE - 8.0,
            TILE_SIZE - 8.0,
            BLUE,
        );

        // Glyph overlay
        draw_text(
            "@",
            self.position.x as f32 * TILE_SIZE + 10.0,
            self.position.y as f32 * TILE_SIZE + 60.0,
            18.0,
            WHITE,
        );
    }

    fn is_player(&self) -> bool {
        true
    }
}