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
use crate::ui::Ui;
use crate::player::Player;
use crate::input::{Input, KeyboardAction};
use crate::position::Position;

use crate::spell_type;
use crate::map_generator;
use macroquad::time::get_time;

use std::rc::Rc;
use std::cell::RefCell;

pub struct GameState {
    pub player: Player,
    pub map: Map,
    pub ui: Ui,
}

impl GameState {
    pub fn get_player_hp(&self) -> (i32, i32) {
        (self.player.hp, self.player.max_hp)
    }

    pub fn get_player_sp(&self) -> u32 {
        self.player.spell_points
    }
}

fn draw(game: &mut GameState, game_interface_offset: (f32, f32)) {
    if !game.ui.is_focused {
        game.map.draw(&game.player, game_interface_offset);
    }
    
    let resolution = (screen_width(), screen_height());

    let (hp, max_hp) = game.get_player_hp();
    let sp = game.get_player_sp();
    game.ui.set_player_hp(hp, max_hp);
    game.ui.set_player_sp(sp);

    game.ui.draw(resolution);
}

pub async fn run() {
    let spell_types = spell_type::load_spell_types().await;
    spell_type::set_global_spell_types(spell_types);

    let mut game = GameState {
        player: Player::new(Position::new(1, 1)),
        map: map_generator::generate(),
        ui: Ui::new(),
    };

    game.map.init(&mut game.player).await;
    game.map.set_player_random_position(&mut game.player);
    
    let mut last_move_time = 0.0;
    let move_interval = 0.15; // seconds between auto steps
    let mut goal_position: Option<Position> = None;
    let game_interface_offset = ( 410.0, 10.0 );

    loop {
        let now = get_time();
        if now - last_move_time < move_interval {
            draw(&mut game, game_interface_offset);
            next_frame().await;
            continue;
        }
        clear_background(BLACK);

        if game.map.last_player_event == Some(PlayerEvent::Death) {
            draw_text("Game Over!", 10.0, 20.0, 30.0, WHITE);
            next_frame().await;
            continue;
        }

        let input = Input::poll();

        let mouse_pos = (input.mouse.0 - game_interface_offset.0, input.mouse.1 - game_interface_offset.1);
        let hover_x = (mouse_pos.0 / TILE_SIZE) as usize;
        let hover_y = ((mouse_pos.1) / TILE_SIZE) as usize;
        let current_tile = Position { x: hover_x, y: hover_y };

        game.map.hovered_changed = game.map.hovered != Some(current_tile);
        game.map.hovered = Some(current_tile);

        if let Some(_click) = input.click {
            goal_position = Some(current_tile)
        };

        if game.ui.is_focused {
            if input.keyboard_action == KeyboardAction::Cancel {
                game.ui.hide();
            }
        }
        else {
            if input.keyboard_action == KeyboardAction::OpenCharacterSheet {
                game.ui.show_character_sheet();
            }
            else {
                game.map.update(&mut game.player, input.keyboard_action, input.direction, input.spell, goal_position);
            }   
        }

        if game.map.last_player_event == Some(PlayerEvent::AutoMove) {
            last_move_time = now; // Update last move time for auto step
        } else {
            goal_position = None;
        }

        draw(&mut game, game_interface_offset);
        next_frame().await;
    }
}