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
use crate::map::{Map, GRID_WIDTH, GRID_HEIGHT, TILE_SIZE};
use crate::player::{KeyboardAction, Player};
use crate::position::Position;
use crate::monster_type::load_monster_types;
use crate::spell_type;
use macroquad::time::get_time;

use std::rc::Rc;
use std::cell::RefCell;

pub struct GameState {
    pub map: Map,
    pub player: Rc<RefCell<Player>>,
}

pub async fn run() {
    let spell_types = spell_type::load_spell_types().await;
    spell_type::set_global_spell_types(spell_types);

    let monster_types = load_monster_types().await;
    let player = Rc::new(RefCell::new(Player::new(Position::new(1, 1))));
    let mut game = GameState {
        player: player.clone(),
        map: Map::generate(player.clone(), &monster_types),
    };

    let mut mouse_down_tile: Option<Position> = None;
    
    let mut last_move_time = 0.0;
    let move_interval = 0.15; // seconds between auto steps

    loop {
        clear_background(BLACK);

        let mouse_pos = mouse_position();
        let hover_x = (mouse_pos.0 / TILE_SIZE) as usize;
        let hover_y = ((mouse_pos.1 - 40.0) / TILE_SIZE) as usize;
        let current_tile = Position { x: hover_x, y: hover_y };
        let mut goal_position = game.player.borrow().goal_position;

        if is_mouse_button_pressed(MouseButton::Left) {
            mouse_down_tile = Some(current_tile);
        }

        let mut hover_changed = game.map.hovered != Some(current_tile);

        if hover_x < GRID_WIDTH && hover_y < GRID_HEIGHT {
            game.map.hovered = Some(current_tile);
        } else {
            if game.map.hovered == None {
                hover_changed = false;
            }
            else {
                game.map.hovered = None;
            }
        }

        game.map.hovered_changed = hover_changed;

        if is_mouse_button_released(MouseButton::Left) {
            if let Some(down_tile) = mouse_down_tile.take() {
                if down_tile == current_tile {
                    // A full click on the same tile — treat as a click!
                    // if down_tile.x < GRID_WIDTH && down_tile.y < GRID_HEIGHT
                        // && game.map.is_walkable(down_tile.x, down_tile.y) && down_tile != game.player.borrow().position
                    // {
                        goal_position = Some(down_tile);
                    // }
                }
            }
        }

        // draw_text("OpenRift - Procedural Map", 10.0, 20.0, 30.0, WHITE);
        game.player.borrow_mut().keyboard_action = KeyboardAction::None;
        let (keyboard_action, direction, spell_action) = game.player.borrow_mut().handle_input(&game.map);

        let mut do_update = false;

        if keyboard_action != KeyboardAction::None {
            do_update = true;
        } else if goal_position.is_some() {
            if game.map.selected_spell.is_some() {
                do_update = true;
            } else {
                let now = get_time();
                if now - last_move_time >= move_interval {
                    do_update = true;
                    last_move_time = now;
                }
            }
        }

        if do_update {
            println!("do_update");
            game.map.update(keyboard_action, direction, spell_action, goal_position);
        }

        game.map.draw();
        //game.player.draw();

        next_frame().await;
    }
}