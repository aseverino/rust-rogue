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
use crate::items::item::Item;
use crate::items::Items;
use crate::map::{Map, TILE_SIZE, PlayerEvent};
use crate::ui::point_f::PointF;
use crate::ui::size_f::SizeF;
use crate::ui::Ui;
use crate::player::Player;
use crate::input::{Input, KeyboardAction};
use crate::position::Position;

use crate::{spell_type};
use crate::map_generator;
use macroquad::time::get_time;

use std::rc::Rc;
use std::cell::RefCell;

pub struct GameState {
    pub player: Player,
    pub map: Map,
    pub ui: Ui,
    pub items: Items,
}

impl GameState {
    pub fn get_player_hp(&self) -> (u32, u32) {
        (self.player.hp, self.player.max_hp)
    }

    pub fn get_player_mp(&self) -> (u32, u32) {
        (self.player.mp, self.player.max_mp)
    }
}

fn draw(game: &mut GameState, game_interface_offset: PointF) {
    if !game.ui.is_focused {
        game.map.draw(&game.player, game_interface_offset);
    }
    
    game.ui.update_geometry(SizeF::new(screen_width(), screen_height()));

    let (hp, max_hp) = game.get_player_hp();
    let (mp, max_mp) = game.get_player_mp();
    game.ui.set_player_hp(hp, max_hp);
    game.ui.set_player_mp(mp, max_mp);
    game.ui.set_player_sp(game.player.sp);
    game.ui.set_player_str(game.player.strength);

    game.ui.draw();
}

pub async fn run() {
    let spell_types = spell_type::load_spell_types().await;
    spell_type::set_global_spell_types(spell_types);

    let mut game = GameState {
        player: Player::new(Position::new(1, 1)),
        map: map_generator::generate(),
        ui: Ui::new(),
        items: Items::new()
    };

    game.items.load_holdable_items().await;

    game.map.init(&mut game.player).await;
    game.map.set_player_random_position(&mut game.player);
    
    let mut last_move_time = 0.0;
    let move_interval = 0.15; // seconds between auto steps
    let mut goal_position: Option<Position> = None;
    let game_interface_offset = PointF::new(410.0, 10.0);

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

        let global_mouse_pos = PointF::new(input.mouse.x, input.mouse.y);
        let map_mouse_pos = PointF::new(input.mouse.x - game_interface_offset.x, input.mouse.y - game_interface_offset.y);
        let map_hover_x = (map_mouse_pos.x / TILE_SIZE) as usize;
        let map_hover_y = ((map_mouse_pos.y) / TILE_SIZE) as usize;
        let current_tile = Position { x: map_hover_x, y: map_hover_y };

        game.map.hovered_tile_changed = game.map.hovered_tile != Some(current_tile);
        game.map.hovered_tile = Some(current_tile);
        game.ui.update_mouse_position(global_mouse_pos);

        if let Some(_click) = input.click {
            if game.ui.is_focused {
                game.ui.handle_click(global_mouse_pos);
            }
            else {
                goal_position = Some(current_tile)
            }
        };

        if game.ui.is_focused {
            if input.keyboard_action == KeyboardAction::Cancel {
                game.ui.hide();
            }
        }
        else {
            if input.keyboard_action == KeyboardAction::OpenCharacterSheet {
                game.ui.toggle_character_sheet();
            }
            else {
                game.map.update(&mut game.player, input.keyboard_action, input.direction, input.spell, goal_position);

                if game.map.last_player_event == Some(PlayerEvent::OpenChest) {
                    if let Some(items_vec) = game.map.get_chest_items(game.player.position) {
                        
                        let actual_items: Vec<(u32, String)> = items_vec.iter()
                            .filter_map(|item_id| {
                                game.items.items.iter()
                                    .find(|item| item.borrow().get_id() == *item_id)
                                    .map(|item_rc| {
                                        let item_ref = item_rc.borrow();
                                        (item_ref.get_id(), item_ref.get_name().to_string())
                                    })
                            })
                            .collect();

                        game.ui.show_chest_view(&actual_items);
                    }
                }
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