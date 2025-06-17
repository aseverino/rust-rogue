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
use crate::items::collection::Items;
use crate::lua_interface::LuaInterface;
use crate::maps::overworld::{self, Overworld, OverworldPos};
use crate::maps::{GRID_HEIGHT, GRID_WIDTH};
use crate::maps::{map::Map, TILE_SIZE, map::PlayerEvent};
use crate::ui::point_f::PointF;
use crate::ui::size_f::SizeF;
use crate::ui::manager::Ui;
use crate::player::Player;
use crate::input::{Input, KeyboardAction};
use crate::position::Position;

use crate::{spell_type};
use macroquad::time::get_time;

use std::rc::Rc;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

pub struct GameState {
    pub player: Player,
    pub overworld: Arc<Mutex<Overworld>>,
    pub ui: Ui,
    pub items: Items,
    pub lua_interface: LuaInterface
}

#[derive(PartialEq, Debug)]
enum PlayerOverworldEvent {
    None,
    BorderCross,
    ClimbDown,
}

impl GameState {
    pub fn get_player_hp(&self) -> (u32, u32) {
        (self.player.hp, self.player.max_hp)
    }

    pub fn get_player_mp(&self) -> (u32, u32) {
        (self.player.mp, self.player.max_mp)
    }
}

fn draw(game: &mut GameState, map: &mut Map, game_interface_offset: PointF) {
    if !game.ui.is_focused {
        map.draw(&game.player, game_interface_offset);
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
        overworld: Overworld::new().await,
        ui: Ui::new(),
        items: Items::new(),
        lua_interface: LuaInterface::new(),
    };

    game.items.load_holdable_items().await;

    let mut overworld_pos = OverworldPos { floor: 0, x: 2, y: 2 };
    let mut current_downstair_teleport_pos: Option<Position>;

    let map_arc = if let Some(map_arc) = game.overworld.lock().unwrap().get_map_ptr(overworld_pos) {
        map_arc
    } else {
        panic!("Failed to get map pointer from overworld");
    };
    let mut current_map_arc = map_arc;
    {
        let mut map = current_map_arc.lock().unwrap();
        map.add_player_first_map(&mut game.player);
    }

    let mut last_move_time = 0.0;
    let move_interval = 0.15; // seconds between auto steps
    let mut goal_position: Option<Position> = None;
    let game_interface_offset = PointF::new(410.0, 10.0);
    let chest_action: Rc<RefCell<Option<u32>>> = Rc::new(RefCell::new(None));
    let mut map_update = PlayerOverworldEvent::None;

    loop {
        if map_update != PlayerOverworldEvent::None {
            // Determine player's current border position
            let mut player_pos = game.player.position;
            let mut new_opos = overworld_pos.clone();

            if map_update == PlayerOverworldEvent::BorderCross {
                if player_pos.x == 0 {
                    new_opos.x -= 1;
                }
                else if player_pos.x == GRID_WIDTH - 1 {
                    new_opos.x += 1;
                }
                if player_pos.y == 0 {
                    new_opos.y -= 1;
                }
                else if player_pos.y == GRID_HEIGHT - 1 {
                    new_opos.y += 1;
                }
            }
            else {
                new_opos.floor += 1; // Climbing down
            }

            {
                let mut overworld = game.overworld.lock().unwrap();
                if let Some(new_map_arc) = overworld.get_map_ptr(new_opos) {
                    {
                        let mut map = current_map_arc.lock().unwrap();
                        map.remove_creature(&mut game.player);
                    }

                    current_map_arc = new_map_arc;
                    
                    current_downstair_teleport_pos = {
                        let map = current_map_arc.lock().unwrap();
                        map.downstair_teleport.clone()
                    };
                    println!("Current downstairs: {:?}", current_downstair_teleport_pos);

                    // Setting up the map adjacencies has to be done before locking it
                    overworld.setup_adjacent_maps(new_opos.floor, new_opos.x, new_opos.y, current_downstair_teleport_pos.unwrap());
                    let mut map = current_map_arc.lock().unwrap();

                    if map_update == PlayerOverworldEvent::BorderCross {
                        if player_pos.x == 0 {
                            player_pos.x = GRID_WIDTH - 2;
                        }
                        else if player_pos.x == GRID_WIDTH - 1 {
                            player_pos.x = 1;
                        }
                        if player_pos.y == 0 {
                            player_pos.y = GRID_HEIGHT - 2;
                        }
                        else if player_pos.y == GRID_HEIGHT - 1 {
                            player_pos.y = 1;
                        }
                    }
                    
                    map.add_player(&mut game.player, player_pos);
                    println!("Player moved to new map at position: {:?}", new_opos);
                    
                    overworld_pos = new_opos;
                } else {
                    panic!("Failed to get map pointer from overworld");
                }
            }
            map_update = PlayerOverworldEvent::None;
        }
        else if let Some(item_id) = chest_action.take() {
            let item = game.items.items[item_id as usize].clone();
            game.player.add_item(item);
            {
                let mut map = current_map_arc.lock().unwrap();
                map.remove_chest(game.player.position);
            }
            game.ui.hide();
        }

        let now = get_time();
        if now - last_move_time < move_interval {
            {
                let mut map = current_map_arc.lock().unwrap();
                draw(&mut game, &mut map, game_interface_offset);
            }
            next_frame().await;
            continue;
        }
        clear_background(BLACK);

        let player_event = { current_map_arc.lock().unwrap().last_player_event.clone() };

        if player_event == Some(PlayerEvent::Death) {
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

        {
            let mut map = current_map_arc.lock().unwrap();
            map.hovered_tile_changed = map.hovered_tile != Some(current_tile);
            map.hovered_tile = Some(current_tile);
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
                    map.update(&mut game.player, &mut game.lua_interface, input.keyboard_action, input.direction, input.spell, goal_position);
                    let player_pos = { game.player.position };

                    if player_event == Some(PlayerEvent::OpenChest) {
                        if let Some(items_vec) = map.get_chest_items(&player_pos) {
                            
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

                            let chest_action_clone = chest_action.clone();

                            game.ui.show_chest_view(&actual_items, Box::new(move |item_id| {
                                *chest_action_clone.borrow_mut() = Some(item_id);
                            }));
                        }
                    }
                    else if player_event == Some(PlayerEvent::ReachBorder) {
                        map_update = PlayerOverworldEvent::BorderCross;
                    }
                    else if player_event == Some(PlayerEvent::ClimbDown) {
                        map_update = PlayerOverworldEvent::ClimbDown;
                    }
                }
            }

            if map.last_player_event == Some(PlayerEvent::AutoMove) {
                last_move_time = now; // Update last move time for auto step
            } else {
                goal_position = None;
            }

            draw(&mut game, &mut map, game_interface_offset);
        }
        next_frame().await;
    }
}