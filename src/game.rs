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
use crate::creature::Creature;
use crate::items::base_item::ItemKind;
use crate::items::collection::Items;
use crate::lua_interface::{LuaInterface, LuaInterfaceRc};
use crate::maps::map::{MapRef, VisitedState};
use crate::maps::navigator::Navigator;
use crate::maps::overworld::{Overworld, OverworldGenerator, OverworldPos};
use crate::maps::{GRID_HEIGHT, GRID_WIDTH};
use crate::maps::{map::Map, TILE_SIZE};
use crate::monster::Monster;
use crate::tile::{NO_CREATURE, PLAYER_CREATURE_ID};
use crate::ui::point_f::PointF;
use crate::ui::size_f::SizeF;
use crate::ui::manager::{Ui, UiEvent};
use crate::player::Player;
use crate::input::{Input, KeyboardAction};
use crate::position::{Direction, Position};

use crate::{combat, monster_type, spell_type};
use macroquad::time::get_time;

use std::cmp::max;
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum PlayerEvent {
    None,
    Move,
    AutoMove,
    AutoMoveEnd,
    Wait,
    Cancel,
    Confirm,
    MeleeAttack,
    SpellSelect,
    SpellCast,
    OpenChest,
    Death,
    ReachBorder,
    ClimbDown,
}

pub struct GameState {
    pub player: Player,
    pub overworld_generator: Arc<Mutex<OverworldGenerator>>,
    pub overworld: Overworld,
    pub items: Items,
    pub lua_interface: LuaInterfaceRc,
    pub last_player_event: PlayerEvent,
}

#[derive(Clone, PartialEq, Debug)]
enum MapTravelKind {
    BorderCross,
    ClimbDown,
}

#[derive(Clone, PartialEq, Debug)]
enum MapTravelEvent {
    None,
    Peek(MapTravelKind),
    Visit(MapTravelKind),
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
    ui.set_player_dex(game.player.dexterity);
    ui.set_player_int(game.player.intelligence);

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

fn get_new_opos(player_pos: &Position, player_opos: &OverworldPos, map_update: &MapTravelEvent) -> OverworldPos {
    let mut new_opos = *player_opos;
    if *map_update == MapTravelEvent::Peek(MapTravelKind::BorderCross) || *map_update == MapTravelEvent::Visit(MapTravelKind::BorderCross) {
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

    new_opos
}

fn check_for_map_update(
    game: &mut GameState,
    map_update: &mut MapTravelEvent,
    last_map_travel_kind: &mut MapTravelKind,
    peek_map_rc: &mut Option<MapRef>,
    current_map_rc: &mut MapRef,
    current_downstair_teleport_pos: &mut Option<Position>,
    overworld_pos: &mut OverworldPos) {
    if *map_update != MapTravelEvent::None {
        // Determine player's current border position
        let mut player_pos = game.player.position;
        let new_opos = get_new_opos(&player_pos, overworld_pos, &map_update);

        let new_map_rc = get_map_ptr(game, new_opos);
        
        if let MapTravelEvent::Peek(_) = map_update {
            let mut map = new_map_rc.borrow_mut();
            if map.visited_state == VisitedState::Visited {
                if *map_update == MapTravelEvent::Peek(MapTravelKind::ClimbDown) {
                    *map_update = MapTravelEvent::Visit(MapTravelKind::ClimbDown);
                }
                else {
                    *map_update = MapTravelEvent::Visit(MapTravelKind::BorderCross);
                }
            }
            else {
                // If the map is not visited, we need to set it up
                *peek_map_rc = Some(new_map_rc.clone());
                println!("Peeked at new map: {:?}", new_opos);
                *map_update = MapTravelEvent::None; // Reset map update to None

                if map.visited_state == VisitedState::NotVisited {
                    map.visited_state = VisitedState::Peeked;
                    *current_downstair_teleport_pos = {
                        let current_map = current_map_rc.borrow();
                        current_map.downstair_teleport.clone()
                    };
                    // Setting up the map adjacencies has to be done before locking it
                    {
                        println!("peeking for the first time");
                        let mut overworld_generator = game.overworld_generator.lock().unwrap();
                        overworld_generator.setup_adjacent_maps(new_opos.floor, new_opos.x, new_opos.y, current_downstair_teleport_pos.unwrap());
                    }
                }
                //*overworld_pos = new_opos;
                return;
            }
        }

        if let MapTravelEvent::Visit(_) = map_update {
            {
                let new_map_rc = get_map_ptr(game, new_opos);

                {
                    let mut map = current_map_rc.borrow_mut();
                    map.remove_creature(&mut game.player);
                }

                *current_map_rc = new_map_rc;

                let mut map = current_map_rc.borrow_mut();

                if *map_update == MapTravelEvent::Visit(MapTravelKind::BorderCross) {
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
                
                map.visited_state = VisitedState::Visited;
                map.add_player(&mut game.player, player_pos);
                peek_map_rc.take();
                println!("Player moved to new map at position: {:?}", new_opos);
                
                *overworld_pos = new_opos;
            }
        }
        
        *last_map_travel_kind = match map_update {
            MapTravelEvent::Peek(kind) => kind.clone(),
            MapTravelEvent::Visit(kind) => kind.clone(),
            MapTravelEvent::None => MapTravelKind::BorderCross, // Default case
        };
        *map_update = MapTravelEvent::None;
    }
}

pub async fn run() {
    let lua_interface = LuaInterface::new();
    
    let spell_types = spell_type::load_spell_types().await;
    spell_type::set_global_spell_types(spell_types);

    let monster_types = Arc::new(Mutex::new(monster_type::load_monster_types(&lua_interface).await));

    let mut game = GameState {
        player: Player::new(Position::new(1, 1)),
        overworld_generator: OverworldGenerator::new(&lua_interface, &monster_types).await,
        overworld: Overworld::new(),
        items: Items::new(),
        lua_interface: lua_interface,
        last_player_event: PlayerEvent::None,
    };

    game.items.load_holdable_items(&game.lua_interface).await;

    let mut overworld_pos = OverworldPos { floor: 0, x: 2, y: 2 };
    let mut current_downstair_teleport_pos: Option<Position> = None;

    let mut current_map_rc = get_map_ptr(&mut game, overworld_pos);
    let mut peek_map_rc: Option<MapRef> = None;

    let shared_map_ptr: Rc<RefCell<Rc<RefCell<Map>>>> = Rc::new(RefCell::new(current_map_rc.clone()));

    {
        let mut map = current_map_rc.borrow_mut();
        map.add_player_first_map(&mut game.player);
        map.visited_state = VisitedState::Visited;
    }

    let mut last_move_time = 0.0;
    let move_interval = 0.15; // seconds between auto steps
    let mut goal_position: Option<Position> = None;
    let game_interface_offset = PointF::new(410.0, 10.0);
    let mut map_update = MapTravelEvent::None;
    let mut last_map_travel_kind = MapTravelKind::BorderCross;
    let mut ui = Ui::new();

    let shared_map_ptr_clone = shared_map_ptr.clone();
    game.lua_interface.borrow_mut().add_monster_callback = Some(Rc::new(move |kind_id, pos| {
        let binding = monster_types.lock().unwrap();
        let kind = binding.iter()
            .find(|mt| mt.id == kind_id)
            .expect("Monster type not found");

        // Create a new monster and wrap it in Rc
        let monster = Monster::new(pos.clone(), kind.clone());
        
        let binding = shared_map_ptr_clone.borrow_mut();
        let mut map = binding.borrow_mut();
        map.tiles[pos].creature = monster.id; // Set the creature ID in the tile
        // Wrap the monster in Rc and push to creatures
        map.monsters.insert(monster.id, monster);
    }));

    let _ = LuaInterface::register_api(&game.lua_interface);

    loop {
        while ui.events.len() > 0 {
            let event = ui.events.pop_front().unwrap();
            match event {
                UiEvent::IncStrength => {
                    if game.player.sp > 0 {
                        game.player.strength += 1;
                        game.player.sp -= 1;
                    }
                },
                UiEvent::IncDexterity => {
                    if game.player.sp > 0 {
                        game.player.dexterity += 1;
                        game.player.sp -= 1;
                    }
                },
                UiEvent::IncIntelligence => {
                    if game.player.sp > 0 {
                        game.player.intelligence += 1;
                        game.player.sp -= 1;
                    }
                },
                UiEvent::ChestAction(item_id) => {
                    let item = game.items.items[item_id as usize].clone();
                    game.player.add_item(item);
                    {
                        let mut map = current_map_rc.borrow_mut();
                        map.remove_chest(game.player.position);
                    }
                    ui.hide();
                },
                _ => {}
            }
        }

        check_for_map_update(
            &mut game,
            &mut map_update,
            &mut last_map_travel_kind,
            &mut peek_map_rc,
            &mut current_map_rc,
            &mut current_downstair_teleport_pos,
            &mut overworld_pos
        );

        if !Rc::ptr_eq(&current_map_rc, &shared_map_ptr.borrow()) {
            // Update the shared map pointer if it has changed
            *shared_map_ptr.borrow_mut() = current_map_rc.clone();
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

        let player_event = game.last_player_event;

        if player_event == PlayerEvent::Death {
            draw_text("Game Over!", 10.0, 20.0, 30.0, WHITE);
            next_frame().await;
            continue;
        }

        let input = Input::poll();

        if peek_map_rc.is_some() {
            if input.keyboard_action == KeyboardAction::Confirm {
                map_update = MapTravelEvent::Visit(last_map_travel_kind.clone());
            } else if input.keyboard_action == KeyboardAction::Cancel {
                peek_map_rc = None; // Reset peek map
            } else {
                let mut map = peek_map_rc.as_mut().unwrap().borrow_mut();
                map.compute_player_fov(&mut game.player, max(GRID_WIDTH, GRID_HEIGHT));
                draw(&mut game, &mut ui, &mut map, game_interface_offset);
            }

            next_frame().await;
            continue;
        }

        let global_mouse_pos = PointF::new(input.mouse.x, input.mouse.y);
        let map_mouse_pos = PointF::new(input.mouse.x - game_interface_offset.x, input.mouse.y - game_interface_offset.y);
        let map_hover_x = (map_mouse_pos.x / TILE_SIZE) as usize;
        let map_hover_y = ((map_mouse_pos.y) / TILE_SIZE) as usize;
        let current_tile = Position { x: map_hover_x, y: map_hover_y };

        {
            let mut map = current_map_rc.borrow_mut();
            map.hovered_tile_changed = map.hovered_tile != Some(current_tile);
            map.hovered_tile = Some(current_tile);
        }

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
                update(&mut game, &mut current_map_rc, input.keyboard_action, input.direction, input.spell, goal_position);
                let player_pos = game.player.position;

                if player_event == PlayerEvent::OpenChest {
                    if let Some(items_vec) = current_map_rc.borrow_mut().get_chest_items(&player_pos) {
                        
                        let actual_items: Vec<(u32, String)> = items_vec.iter()
                            .filter_map(|item_id| {
                                game.items.items.iter()
                                    .find(|item| item.id() == *item_id)
                                    .map(|item| {
                                        (item.id(), item.name().to_string())
                                    })
                            })
                            .collect();

                        ui.show_chest_view(&actual_items);
                    }
                }
                else if player_event == PlayerEvent::ReachBorder {
                    map_update = MapTravelEvent::Peek(MapTravelKind::BorderCross);
                }
                else if player_event == PlayerEvent::ClimbDown {
                    map_update = MapTravelEvent::Peek(MapTravelKind::ClimbDown);
                }
            }
        }

        {
            let mut map = current_map_rc.borrow_mut();
            if game.last_player_event == PlayerEvent::AutoMove {
                last_move_time = now; // Update last move time for auto step
            } else {
                goal_position = None;
            }

            draw(&mut game, &mut ui, &mut map, game_interface_offset);
        }
        
        next_frame().await;
    }
}

pub fn update(
        game: &mut GameState,
        map_ref: &mut MapRef,
        player_action: KeyboardAction,
        player_direction: Direction,
        spell_action: i32,
        player_goal_position: Option<Position>
) {
    game.last_player_event = PlayerEvent::None;
    let player_pos = game.player.position;

    let mut new_player_pos: Option<Position> = None;
    let mut update_monsters = false;

    if let Some(player_goal) = player_goal_position {
        let spell_index = game.player.selected_spell;

        if let Some(index) = spell_index {
            let mut should_cast = false;
            {
                let (in_line_of_sight, spell_range) = {
                    let in_line_of_sight = game.player.line_of_sight.contains(&player_goal);
                    let spell_range = game.player.spells.get(index)
                        .expect("Selected spell index out of bounds")
                        .spell_type.range;
                    (in_line_of_sight, spell_range)
                };

                if in_line_of_sight && player_pos.in_range(&player_goal, spell_range as usize) {
                    if let Some(spell) = game.player.spells.get_mut(index) {
                        if spell.charges > 0 {
                            spell.charges -= 1;
                            println!("Casting spell charges {}", spell.charges);
                            should_cast = true;
                        }
                        else {
                            println!("No charges left for this spell!");
                        }
                    }
                }
            }
            
            if should_cast {
                combat::do_spell_combat(&mut game.player, map_ref, player_pos, player_goal, index, &game.lua_interface);
                update_monsters = true;
            }

            game.player.selected_spell = None;
            game.player.goal_position = None; // Clear goal position
            game.last_player_event = PlayerEvent::SpellCast;
        }
        else {
            let path: Option<Vec<Position>> = Navigator::find_path(player_pos, player_goal, |pos| {
                map_ref.borrow().is_tile_walkable(pos)
            });

            if let Some(path) = path {
                if path.len() > 1 {
                    new_player_pos = Some(path[1]);
                    game.last_player_event = PlayerEvent::AutoMove;
                }
                else {
                    game.last_player_event = PlayerEvent::AutoMoveEnd;
                }
                game.player.goal_position = player_goal_position;
            }
            else {
                game.player.goal_position = None; // Clear goal if no path found
                game.last_player_event = PlayerEvent::AutoMoveEnd;
            }
        }
        
    }
    else if player_action == KeyboardAction::SpellSelect && spell_action > 0 {
        let index = spell_action as usize - 1;

        let spell_name = {
            game.player.spells.get(index).map(|spell| spell.spell_type.name.clone())
        };
        if let Some(name) = spell_name {
            game.player.selected_spell = Some(index);
            println!("Spell selected: {}", name);
        } else {
            println!("No spell selected!");
        }

        game.last_player_event = PlayerEvent::SpellSelect;
    }
    else if player_action == KeyboardAction::Cancel {
        game.player.selected_spell = None;
        game.player.goal_position = None; // Clear goal position

        game.last_player_event = PlayerEvent::Cancel;
    }
    else if player_action == KeyboardAction::Move {
        let pos_change = match player_direction {
            Direction::Up => (0, -1),
            Direction::Right => (1, 0),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::UpRight => (1, -1),
            Direction::DownRight => (1, 1),
            Direction::DownLeft => (-1, 1),
            Direction::UpLeft => (-1, -1),
            Direction::None => (0, 0),
        };

        let pos = Position {
            x: (player_pos.x as isize + pos_change.0) as usize,
            y: (player_pos.y as isize + pos_change.1) as usize
        };

        {
            let map = map_ref.borrow();
            if map.is_tile_enemy_occupied(pos) {
                game.last_player_event = PlayerEvent::MeleeAttack;
                update_monsters = true; // Update monsters if player attacks
                drop(map);
                combat::do_melee_combat(&mut game.player, map_ref, player_pos, pos, &game.lua_interface);
            }
            else if map.tiles[pos].is_border(&pos) && !map.monsters.is_empty() {
                game.last_player_event = PlayerEvent::Cancel;
            }
            else {
                if map.is_tile_walkable(pos) {
                    new_player_pos = Some(pos);
                    update_monsters = true; // Update monsters if player moves
                }

                game.last_player_event = PlayerEvent::Move;
            }
        }
        
        game.player.goal_position = None;
    }
    else if player_action == KeyboardAction::Wait {
        new_player_pos = Some(player_pos); // Stay in place
        game.player.goal_position = None; // Clear goal position

        game.last_player_event = PlayerEvent::Wait;
    }

    if let Some(pos) = new_player_pos {
        let mut map = map_ref.borrow_mut();
        map.tiles[player_pos].creature = NO_CREATURE;
        map.tiles[pos].creature = PLAYER_CREATURE_ID;

        game.player.set_pos(pos);
        
        if new_player_pos == game.player.goal_position {
            game.player.goal_position = None; // Clear goal position if reached
        }

        map.compute_player_fov(&mut game.player, max(GRID_WIDTH, GRID_HEIGHT));
        update_monsters = true;

        let mut to_remove: Vec<usize> = Vec::new();
        
        for (idx, item) in map.tiles[pos].items.iter().rev().enumerate() {
            match item {
                ItemKind::Orb(_) => {
                    println!("Player picked up an orb at index {idx}!");
                    game.player.sp += 1;
                    to_remove.push(idx); // Collect for removal
                }
                ItemKind::Teleport(_) => {
                    if map.monsters.is_empty() {
                        println!("Player walked downstairs.");
                        game.last_player_event = PlayerEvent::ClimbDown;
                        return;
                    }
                }
                ItemKind::Container(_) => {
                    game.last_player_event = PlayerEvent::OpenChest;
                    //to_remove.push(idx); // Collect for removal
                }
                _ => {
                    // println!("Player found an item: {:?}", other_item);
                }
            }
        }

        for idx in to_remove {
            map.tiles[pos].remove_item(idx);
        }

        if map.tiles[pos].is_border(&pos) {
            game.last_player_event = PlayerEvent::ReachBorder;
        }
    }

    if update_monsters {
        let mut map = map_ref.borrow_mut();
        let walkable_tiles = map.tiles.clone(); // Clone the tiles to avoid borrowing conflicts
        let mut monster_moves: Vec<(Position, Position, usize)> = Vec::new();

        for (id, monster) in &mut map.monsters {
            if monster.hp <= 0 {
                continue; // Skip dead monsters
            }
            let monster_pos = monster.pos();

            let path = Navigator::find_path(monster_pos, game.player.position, |pos| {
                pos.x < GRID_WIDTH && pos.y < GRID_HEIGHT && walkable_tiles[pos].is_walkable()
            });

            if let Some(path) = path {
                if path.len() > 1 {
                    let next_step = path[1];

                    if next_step == game.player.position {
                        println!("Monster {} hit player for {} damage!", monster.name(), monster.kind.melee_damage);
                        game.player.add_health(-monster.kind.melee_damage);
                        if game.player.hp <= 0 {
                            println!("Player has been defeated!");
                            game.last_player_event = PlayerEvent::Death;
                            return;
                        }
                        continue;
                    }

                    monster_moves.push((monster_pos, next_step, *id as usize));
                    monster.set_pos(next_step);
                }
            }
        }

        for (monster_pos, next_step, i) in monster_moves {
            map.tiles[monster_pos].creature = NO_CREATURE;
            map.tiles[next_step].creature = i as u32;
        }
    }
}