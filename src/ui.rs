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

use macroquad::prelude::*;

pub struct Ui {
    player_hp: i32,
    player_max_hp: i32,
}

impl Ui {
    pub fn new() -> Self {
        Ui {
            player_hp: 0,
            player_max_hp: 0,
        }
    }

    pub fn set_player_hp(&mut self, hp: i32, max_hp: i32) {
        self.player_hp = hp;
        self.player_max_hp = max_hp;
    }

    pub fn set_last_action(&mut self, _action: &str) {

    }

    pub fn draw(&mut self, resolution: (f32, f32)) {
        draw_rectangle_lines(
            0.0,
            0.0,
            400.0,
            resolution.1,
            2.0,
            WHITE,
        );

        let mut offset: (f32, f32) = (10.0, 30.0);

        let mut text: String = "HP".to_string();
        let true_red = Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0};
        draw_text(&text, offset.0, offset.1, 30.0, true_red);
        let mut text_offset = measure_text(&text, None, 30, 1.0).width;
        text_offset += offset.0 + 10.0;
        let start_of_health = text_offset;

        text = format!("{}", self.player_hp);
        draw_text(&text, text_offset, offset.1, 30.0, if self.player_hp < self.player_max_hp / 2 { true_red } else { WHITE });
        text_offset += measure_text(&text, None, 30, 1.0).width;

        text = format!("/{}", self.player_max_hp);
        draw_text(&text,text_offset,offset.1,30.0,WHITE);

        offset.1 += 30.0;
        text = "SP".to_string();
        draw_text(&text,offset.0,offset.1,30.0,YELLOW);
        text_offset = start_of_health;

        text = "0".to_string();
        draw_text(&text,text_offset,offset.1,30.0,YELLOW);

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
}