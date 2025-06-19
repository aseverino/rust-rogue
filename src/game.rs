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
use crate::maps::overworld::{Overworld, OverworldGenerator, OverworldPos};
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
    pub overworld_generator: Arc<Mutex<OverworldGenerator>>,
    pub overworld: Overworld,
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

fn draw(game: &mut GameState, ui: &mut Ui, map: &mut Map, game_interface_offset: PointF)
{
    if !ui.is_focused {
        map.draw(&mut game.player, game_interface_offset);
    }
    
    ui.update_geometry(SizeF::new(screen_width(), screen_height()));

    let (hp, max_hp) = game.get_player_hp();
    let (mp, max_mp) = game.get_player_mp();
    ui.set_player_hp(hp, max_hp);
    ui.set_player_mp(mp, max_mp);

    ui.set_player_sp(game.player.sp);
    ui.set_player_str(game.player.strength);

    ui.draw();
}

fn get_map_ptr(game: &mut GameState, overworld_pos: OverworldPos) -> Rc<RefCell<Map>> {
    let current_map_rc = game.overworld.get_map_ptr(overworld_pos);

    if current_map_rc.is_none() {
        let generated_map_arc = if let Some(generated_map_arc) = game.overworld_generator.lock().unwrap().get_generated_map_ptr(overworld_pos) {
            generated_map_arc
        } else {
            panic!("Failed to get map pointer from overworld");
        };

        game.overworld.add_map(overworld_pos, generated_map_arc.clone())
    }
    else {
        current_map_rc.unwrap()
    }
}

pub async fn run() {
    let mut lua_interface = LuaInterface::new();
    lua_interface.init().unwrap();
    
    let spell_types = spell_type::load_spell_types().await;
    spell_type::set_global_spell_types(spell_types);

    let mut game = GameState {
        player: Player::new(Position::new(1, 1)),
        overworld_generator: OverworldGenerator::new(&mut lua_interface).await,
        overworld: Overworld::new(),
        items: Items::new(),
        lua_interface: lua_interface
    };

    game.items.load_holdable_items(&mut game.lua_interface).await;

    let mut overworld_pos = OverworldPos { floor: 0, x: 2, y: 2 };
    let mut current_downstair_teleport_pos: Option<Position>;

    let mut current_map_rc = get_map_ptr(&mut game, overworld_pos);

    {
        let mut map = current_map_rc.borrow_mut();
        map.add_player_first_map(&mut game.player);
    }

    let mut last_move_time = 0.0;
    let move_interval = 0.15; // seconds between auto steps
    let mut goal_position: Option<Position> = None;
    let game_interface_offset = PointF::new(410.0, 10.0);
    let mut chest_action: Rc<RefCell<Option<u32>>> = Rc::new(RefCell::new(None));
    let mut map_update = PlayerOverworldEvent::None;
    let mut ui = Ui::new();

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
                let new_map_rc = get_map_ptr(&mut game, new_opos);

                    {
                    let mut map = current_map_rc.borrow_mut();
                    map.remove_creature(&mut game.player);
                    }

                current_map_rc = new_map_rc;
                    
                    current_downstair_teleport_pos = {
                    let map = current_map_rc.borrow();
                        map.downstair_teleport.clone()
                    };
                    println!("Current downstairs: {:?}", current_downstair_teleport_pos);

                    // Setting up the map adjacencies has to be done before locking it
                {
                    let mut overworld_generator = game.overworld_generator.lock().unwrap();
                    overworld_generator.setup_adjacent_maps(new_opos.floor, new_opos.x, new_opos.y, current_downstair_teleport_pos.unwrap());
                }
                let mut map = current_map_rc.borrow_mut();

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
            }
            map_update = PlayerOverworldEvent::None;
        }
        else if let Some(item_id) = chest_action.take() {
            let item = game.items.items[item_id as usize].clone();
            game.player.add_item(item);
            {
                let mut map = current_map_rc.borrow_mut();
                map.remove_chest(game.player.position);
            }
            ui.hide();
        }

        let now = get_time();
        if now - last_move_time < move_interval {
            {
                let mut map = current_map_rc.borrow_mut();
                draw(&mut game, &mut ui, &mut map, game_interface_offset);
            }
            next_frame().await;
            continue;
        }
        clear_background(BLACK);

        let player_event = { current_map_rc.borrow_mut().last_player_event.clone() };

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
            let mut map = current_map_rc.borrow_mut();
                
            map.hovered_tile_changed = map.hovered_tile != Some(current_tile);
            map.hovered_tile = Some(current_tile);
            ui.update_mouse_position(global_mouse_pos);

            if let Some(_click) = input.click {
                if ui.is_focused {
                    ui.handle_click(global_mouse_pos);
                }
                else {
                    goal_position = Some(current_tile)
                }
            };

            if ui.is_focused {
                if input.keyboard_action == KeyboardAction::Cancel {
                    ui.hide();
                }
            }
            else {
                if input.keyboard_action == KeyboardAction::OpenCharacterSheet {
                    ui.toggle_character_sheet();
                }
                else {
                    map.update(&mut game.player, &mut game.lua_interface, input.keyboard_action, input.direction, input.spell, goal_position);
                    let player_pos = game.player.position;

                    if player_event == Some(PlayerEvent::OpenChest) {
                        if let Some(items_vec) = map.get_chest_items(&player_pos) {
                            
                            let actual_items: Vec<(u32, String)> = items_vec.iter()
                                .filter_map(|item_id| {
                                    game.items.items.iter()
                                        .find(|item| item.id() == *item_id)
                                        .map(|item| {
                                            (item.id(), item.name().to_string())
                                        })
                                })
                                .collect();

                            let chest_action_clone = chest_action.clone();
                            ui.show_chest_view(&actual_items, Box::new(move |item_id| {
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

            draw(&mut game, &mut ui, &mut map, game_interface_offset);
        }
        next_frame().await;
    }
}
