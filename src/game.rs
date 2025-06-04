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
use crate::map::{Map, TILE_SIZE, PlayerEvent};
use crate::player::Player;
use crate::input::{Input, KeyboardAction};
use crate::position::Position;
use crate::monster_type::load_monster_types;
use crate::spell_type;
use macroquad::time::get_time;

use std::rc::Rc;
use std::cell::RefCell;

pub struct GameState {
    pub map: Map,
}

pub async fn run() {
    let spell_types = spell_type::load_spell_types().await;
    spell_type::set_global_spell_types(spell_types);

    let monster_types = load_monster_types().await;
    let player = Player::new(Position::new(1, 1));
    let mut game = GameState {
        map: Map::generate(player, &monster_types),
    };
    
    let mut last_move_time = 0.0;
    let move_interval = 0.15; // seconds between auto steps
    let mut goal_position: Option<Position> = None;

    loop {
        let now = get_time();
        if now - last_move_time < move_interval {
            // If the last move was too recent, skip this frame
            game.map.draw();
            next_frame().await;
            continue;
        }
        clear_background(BLACK);
        let input = Input::poll();

        let mouse_pos = input.mouse;
        let hover_x = (mouse_pos.0 / TILE_SIZE) as usize;
        let hover_y = ((mouse_pos.1 - 40.0) / TILE_SIZE) as usize;
        let current_tile = Position { x: hover_x, y: hover_y };

        game.map.hovered_changed = game.map.hovered != Some(current_tile);
        game.map.hovered = Some(current_tile);

        if let Some(_click) = input.click {
            goal_position = Some(current_tile)
        };

        game.map.update(input.keyboard_action, input.direction, input.spell, goal_position);

        if game.map.last_player_event == Some(PlayerEvent::AutoMove) {
            last_move_time = now; // Update last move time for auto step
        } else {
            goal_position = None;
        }

        game.map.draw();

        next_frame().await;
    }
}