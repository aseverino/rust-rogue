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

use crate::creature::Creature;
use crate::graphics::graphics_manager::GraphicsManager;
use crate::input::{Input, KeyboardAction};
use crate::items::base_item::ItemKind;
use crate::items::collection::{Items, ItemsArc};
use crate::lua_interface::{self, LuaInterface, LuaInterfaceRc, LuaScripted};
use crate::maps::map::MapRc;
use crate::maps::navigator::Navigator;
use crate::maps::overworld::{Overworld, OverworldPos, VisitedState};
use crate::maps::overworld_generator::OverworldGenerator;
use crate::maps::{GRID_HEIGHT, GRID_WIDTH};
use crate::maps::{TILE_SIZE, map::Map};
use crate::monster::{Monster, MonsterRc};
use crate::monster_kind::MonsterKind;
use crate::player::{self, Player, PlayerRc};
use crate::player_spell::PlayerSpell;
use crate::position::{Direction, Position};
use crate::spell_type::{SpellKind, get_spell_types};
use crate::tile::{NO_CREATURE, PLAYER_CREATURE_ID};
use crate::ui::manager::{Ui, UiEvent};
use crate::ui::point_f::PointF;
use crate::ui::size_f::SizeF;
use macroquad::prelude::*;
use mlua::Table;

use crate::{combat, monster_kind, spell_type};
use macroquad::time::get_time;

use std::cell::{RefCell, RefMut};
use std::cmp::max;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex, RwLock};

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
    AttackChooseTarget,
    AttackConfirm,
    OpenChest,
    Death,
    ReachBorder,
    ClimbDown,
}

pub struct GameState {
    pub turn: u32,
    pub player: PlayerRc,
    pub overworld_generator: Arc<Mutex<OverworldGenerator>>,
    pub overworld: Overworld,
    pub items: ItemsArc,
    pub lua_interface: LuaInterfaceRc,
    pub last_player_event: PlayerEvent,
    pub animate_for: f32,
    pub animating_effects: HashMap<Position, Arc<RwLock<Texture2D>>>,
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
        let player = self.player.borrow();
        (player.hp, player.max_hp)
    }

    pub fn get_player_mp(&self) -> (u32, u32) {
        let player = self.player.borrow();
        (player.mp, player.max_mp)
    }
}

fn draw(
    graphics_manager: &mut GraphicsManager,
    game: &mut GameState,
    ui: &mut Ui,
    map: &mut Map,
    game_interface_offset: PointF,
) {
    let (hp, max_hp) = game.get_player_hp();
    let (mp, max_mp) = game.get_player_mp();

    let mut player = game.player.borrow_mut();
    if !ui.is_focused {
        map.draw(
            graphics_manager,
            &mut player,
            game_interface_offset,
            &game.animating_effects,
            game.animate_for,
        );
    }

    ui.update_geometry(SizeF::new(screen_width(), screen_height()));

    ui.set_player_hp(hp, max_hp);
    ui.set_player_mp(mp, max_mp);

    ui.set_player_sp(player.sp);
    ui.set_player_str(player.strength);
    ui.set_player_dex(player.dexterity);
    ui.set_player_int(player.intelligence);

    ui.set_player_weapon(
        player
            .equipment
            .weapon
            .as_ref()
            .map(|weapon| weapon.base_holdable.base_item.name.clone())
            .unwrap_or_else(|| "".to_string()),
    );

    ui.set_player_armor(
        player
            .equipment
            .armor
            .as_ref()
            .map(|armor| armor.base_holdable.base_item.name.clone())
            .unwrap_or_else(|| "".to_string()),
    );

    ui.set_player_shield(
        player
            .equipment
            .shield
            .as_ref()
            .map(|shield| shield.base_holdable.base_item.name.clone())
            .unwrap_or_else(|| "".to_string()),
    );

    ui.set_player_helmet(
        player
            .equipment
            .helmet
            .as_ref()
            .map(|helmet| helmet.base_holdable.base_item.name.clone())
            .unwrap_or_else(|| "".to_string()),
    );

    ui.set_player_boots(
        player
            .equipment
            .boots
            .as_ref()
            .map(|boots| boots.base_holdable.base_item.name.clone())
            .unwrap_or_else(|| "".to_string()),
    );

    ui.draw();
}

fn get_map_ptr(game: &mut GameState, overworld_pos: OverworldPos) -> MapRc {
    let current_map_rc = game.overworld.get_map_ptr(overworld_pos);

    if current_map_rc.is_none() {
        let generated_map_arc = if let Some(generated_map_arc) = game
            .overworld_generator
            .lock()
            .unwrap()
            .get_generated_map_ptr(overworld_pos)
        {
            generated_map_arc
        } else {
            panic!("Failed to get map pointer from overworld");
        };

        game.overworld
            .add_map(overworld_pos, generated_map_arc.clone())
    } else {
        current_map_rc.unwrap()
    }
}

fn get_new_opos(
    player_pos: &Position,
    player_opos: &OverworldPos,
    map_update: &MapTravelEvent,
) -> OverworldPos {
    let mut new_opos = *player_opos;
    if *map_update == MapTravelEvent::Peek(MapTravelKind::BorderCross)
        || *map_update == MapTravelEvent::Visit(MapTravelKind::BorderCross)
    {
        if player_pos.x == 0 {
            new_opos.x -= 1;
        } else if player_pos.x == GRID_WIDTH - 1 {
            new_opos.x += 1;
        }
        if player_pos.y == 0 {
            new_opos.y -= 1;
        } else if player_pos.y == GRID_HEIGHT - 1 {
            new_opos.y += 1;
        }
    } else {
        new_opos = OverworldPos::new(player_opos.floor + 1, 2, 2); // Climbing down
    }

    new_opos
}

fn check_for_map_update(
    game: &mut GameState,
    map_update: &mut MapTravelEvent,
    last_map_travel_kind: &mut MapTravelKind,
    peek_map_rc: &mut Option<MapRc>,
    current_map_rc: &mut MapRc,
    current_downstair_teleport_pos: &mut Option<Position>,
    overworld_pos: &mut OverworldPos,
) {
    if *map_update != MapTravelEvent::None {
        // Determine player's current border position
        let mut player_pos = { game.player.borrow().position };
        let new_opos = get_new_opos(&player_pos, overworld_pos, &map_update);

        let new_map_rc = get_map_ptr(game, new_opos);

        if let MapTravelEvent::Peek(_) = map_update {
            let mut map = new_map_rc.0.borrow_mut();
            if map.generated_map.visited_state == VisitedState::Visited {
                if *map_update == MapTravelEvent::Peek(MapTravelKind::ClimbDown) {
                    *map_update = MapTravelEvent::Visit(MapTravelKind::ClimbDown);
                } else {
                    *map_update = MapTravelEvent::Visit(MapTravelKind::BorderCross);
                }
            } else {
                // If the map is not visited, we need to set it up
                *peek_map_rc = Some(new_map_rc.clone());
                println!("Peeked at new map: {:?}", new_opos);

                *last_map_travel_kind = match map_update {
                    MapTravelEvent::Peek(kind) => kind.clone(),
                    MapTravelEvent::Visit(kind) => kind.clone(),
                    MapTravelEvent::None => MapTravelKind::BorderCross, // Default case
                };
                *map_update = MapTravelEvent::None; // Reset map update to None

                if map.generated_map.visited_state == VisitedState::Unvisited {
                    update_map_visited_state(game, &mut map, new_opos, VisitedState::Peeked);
                    *current_downstair_teleport_pos = {
                        let current_map = current_map_rc.0.borrow();
                        current_map.generated_map.downstair_teleport.clone()
                    };
                    println!("peeking for the first time");
                    //let _ = game.lua_interface.borrow_mut().on_map_peeked(&mut map);
                    drop(map);
                    let map = peek_map_rc.as_ref().unwrap().clone();
                    let peek_call_result = game.lua_interface.borrow_mut().on_map_peeked(&map);

                    if let Err(e) = peek_call_result {
                        eprintln!("Error calling Lua on_map_peeked: {}", e);
                    }
                }

                //print_overworld(game);
                return;
            }
        }

        if let MapTravelEvent::Visit(_) = map_update {
            let mut current_tier = 0u32;
            {
                let new_map_rc = get_map_ptr(game, new_opos);

                {
                    let mut map = current_map_rc.0.borrow_mut();
                    current_tier = map.generated_map.tier;
                    let mut player_ref = game.player.borrow_mut();
                    map.remove_creature(&mut *player_ref);
                    map.remove_downstairs_teleport();
                }

                *current_map_rc = new_map_rc;

                let mut map = current_map_rc.0.borrow_mut();

                if *map_update == MapTravelEvent::Visit(MapTravelKind::BorderCross) {
                    if player_pos.x == 0 {
                        player_pos.x = GRID_WIDTH - 2;
                    } else if player_pos.x == GRID_WIDTH - 1 {
                        player_pos.x = 1;
                    }
                    if player_pos.y == 0 {
                        player_pos.y = GRID_HEIGHT - 2;
                    } else if player_pos.y == GRID_HEIGHT - 1 {
                        player_pos.y = 1;
                    }
                }

                update_map_visited_state(game, &mut map, new_opos, VisitedState::Visited);

                map.add_player(&mut game.player.borrow_mut(), player_pos);
                peek_map_rc.take();
                println!("Player moved to new map at position: {:?}", new_opos);
            }

            {
                let mut overworld_generator = game.overworld_generator.lock().unwrap();
                game.overworld.clear_unvisited(overworld_pos.clone());
                overworld_generator.clear_unvisited(overworld_pos.clone());
                overworld_generator.setup_adjacent_maps(
                    current_tier + 1,
                    overworld_pos.floor,
                    overworld_pos.x,
                    overworld_pos.y,
                    None,
                );
                overworld_generator.setup_adjacent_maps(
                    current_tier + 1,
                    new_opos.floor,
                    new_opos.x,
                    new_opos.y,
                    current_downstair_teleport_pos.clone(),
                );
            }

            *overworld_pos = new_opos;
        }

        *last_map_travel_kind = match map_update {
            MapTravelEvent::Peek(kind) => kind.clone(),
            MapTravelEvent::Visit(kind) => kind.clone(),
            MapTravelEvent::None => MapTravelKind::BorderCross, // Default case
        };
        *map_update = MapTravelEvent::None;

        //print_overworld(game);
    }
}

pub fn update_map_visited_state(
    game: &mut GameState,
    map: &mut RefMut<'_, Map>,
    overworld_pos: OverworldPos,
    visited_state: VisitedState,
) {
    map.generated_map.visited_state = visited_state.clone();

    if let Some(generated_map_arc) = game
        .overworld_generator
        .lock()
        .unwrap()
        .get_generated_map_ptr(overworld_pos)
    {
        let mut generated_map = generated_map_arc.lock().unwrap();
        generated_map.visited_state = visited_state;
    }
}

pub fn print_overworld(game: &mut GameState) {
    //  Overworld          OverworldGenerator
    // [ 0, 0, 0, 0, 0] | [ 0, 0, 0, 0, 0]
    // [ 0, 0, n, 0, 0] | [ 0, 0, n, 0, 0]
    // [ 0, n, v, n, 0] | [ 0, n, v, n, 0]
    // [ 0, 0, p, 0, 0] | [ 0, 0, p, 0, 0]
    // [ 0, 0, 0, 0, 0] | [ 0, 0, 0, 0, 0]
    // n = unvisited, v = visited, p = peeked

    // Overworld: maps[y][x] (row-major)
    let states = |overworld: &Overworld| {
        let mut grid = vec![vec!['0'; 5]; 5];
        for y in 0..5 {
            for x in 0..5 {
                let pos = OverworldPos { floor: 0, x, y };
                if let Some(map_rc) = overworld.get_map_ptr(pos) {
                    let map = map_rc.0.borrow();
                    grid[y][x] = match map.generated_map.visited_state {
                        VisitedState::Unvisited => 'n',
                        VisitedState::Visited => 'v',
                        VisitedState::Peeked => 'p',
                    };
                }
            }
        }
        grid
    };

    // OverworldGenerator: generated_maps[floor][x][y] (column-major, to match Overworld)
    let states_generated = |generated_maps: &Vec<
        [[Option<Arc<Mutex<crate::maps::generated_map::GeneratedMap>>>; 5]; 5],
    >| {
        let mut grid = vec![vec!['0'; 5]; 5];
        if let Some(floor_maps) = generated_maps.get(0) {
            for y in 0..5 {
                for x in 0..5 {
                    if let Some(gmap_arc) = &floor_maps[x][y] {
                        let gmap = gmap_arc.lock().unwrap();
                        grid[y][x] = match gmap.visited_state {
                            VisitedState::Unvisited => 'n',
                            VisitedState::Visited => 'v',
                            VisitedState::Peeked => 'p',
                        };
                    }
                }
            }
        }
        grid
    };

    let left = states(&game.overworld);

    let overworld_generator = game.overworld_generator.lock().unwrap();
    let right = states_generated(&*overworld_generator.generated_maps.lock().unwrap());

    println!("//  Overworld          OverworldGenerator");
    for y in 0..5 {
        let left_row: String = left[y].iter().map(|c| format!("{}, ", c)).collect();
        let right_row: String = right[y].iter().map(|c| format!("{}, ", c)).collect();
        println!(
            "[ {}] | [ {}]",
            &left_row[..left_row.len() - 2],
            &right_row[..right_row.len() - 2]
        );
    }
    println!("// n = unvisited, v = visited, p = peeked");
}

pub async fn run() {
    let lua_interface = LuaInterface::new();

    let spell_types = spell_type::load_spell_types().await;
    spell_type::set_global_spell_types(spell_types);

    let monster_kinds = monster_kind::load_monster_kinds(&lua_interface).await;

    let items = Arc::new(RwLock::new(Items::new()));

    let mut game = GameState {
        player: Rc::new(RefCell::new(Player::new(Position::new(1, 1)).await)),
        overworld_generator: OverworldGenerator::new(
            &lua_interface,
            monster_kinds.read().unwrap().vec.clone(),
            &items,
        )
        .await,
        overworld: Overworld::new(),
        items: items,
        lua_interface: lua_interface,
        last_player_event: PlayerEvent::None,
        turn: 1,
        animating_effects: HashMap::new(),
        animate_for: 0.0,
    };

    game.items
        .write()
        .unwrap()
        .load_holdable_items(&game.lua_interface)
        .await;

    let mut overworld_pos = OverworldPos {
        floor: 0,
        x: 2,
        y: 2,
    };
    let mut current_downstair_teleport_pos: Option<Position> = None;

    let mut current_map_rc = get_map_ptr(&mut game, overworld_pos);
    let mut peek_map_rc: Option<MapRc> = None;

    let shared_map_ptr: Rc<RefCell<MapRc>> = Rc::new(RefCell::new(current_map_rc.clone()));

    {
        let mut map = current_map_rc.0.borrow_mut();
        map.add_player_first_map(&mut game.player.borrow_mut());
        update_map_visited_state(&mut game, &mut map, overworld_pos, VisitedState::Visited);
    }

    let mut last_move_time = 0.0;
    let move_interval = 0.15; // seconds between auto steps
    let mut goal_position: Option<Position> = None;
    let game_interface_offset = PointF::new(410.0, 10.0);
    let mut map_update = MapTravelEvent::None;
    let mut last_map_travel_kind = MapTravelKind::BorderCross;
    let mut ui = Ui::new(get_spell_types());
    //ui.add_player_skills(&game.player.borrow().spells[0]);

    let mut graphics_manager = GraphicsManager::new();

    {
        let mut lua_interface = game.lua_interface.borrow_mut();
        let monster_kinds_clone = monster_kinds.clone();
        lua_interface.map_add_monster_callback = Some(Rc::new(
            move |map_rc, kind_id, pos: Position| -> MonsterRc {
                let binding = monster_kinds_clone.read().unwrap();
                let binding = binding.vec.read().unwrap();
                let kind = binding
                    .iter()
                    .find(|mt| mt.id == kind_id)
                    .expect("Monster type not found");

                // Create a new monster and wrap it in Rc
                let monster = Rc::new(RefCell::new(Monster::new(pos.clone(), kind.clone())));

                let mut map = map_rc.0.borrow_mut();
                map.generated_map.tiles[pos].creature = monster.borrow().id; // Set the creature ID in the tile
                // Wrap the monster in Rc and push to creatures
                map.monsters.insert(monster.borrow().id, monster.clone());
                monster
            },
        ));

        let shared_map_ptr_clone = shared_map_ptr.clone();
        let player_clone = game.player.clone();
        lua_interface.teleport_creature_to_callback =
            Some(Rc::new(move |creature_id, pos: Position| {
                let map_rc = shared_map_ptr_clone.borrow();
                teleport_creature_to(&player_clone, &map_rc, creature_id, pos);
                Ok(())
            }));

        let shared_map_ptr_clone = shared_map_ptr.clone();
        let player_clone = game.player.clone();
        lua_interface.find_monster_path_callback =
            Some(Rc::new(move |monster: &Monster| -> Vec<Position> {
                //let monster = monster_rc.borrow();
                let player_pos = { player_clone.borrow().position };
                let path = find_monster_path(
                    &shared_map_ptr_clone.borrow(),
                    monster.position,
                    player_pos,
                    monster.kind.flying,
                );

                if path.is_none() {
                    return vec![];
                }

                path.unwrap()
            }));

        let shared_player_ptr_clone = game.player.clone();
        lua_interface.get_player_callback = Some(Rc::new(move || -> PlayerRc {
            shared_player_ptr_clone.clone()
        }));

        let shared_map_ptr_clone = shared_map_ptr.clone();
        lua_interface.get_monster_by_id_callback = Some(Rc::new(move |id| -> Option<MonsterRc> {
            let binding = shared_map_ptr_clone.borrow();
            let map = binding.0.borrow();
            if let Some(monster) = map.monsters.get(&id) {
                Some(monster.clone())
            } else {
                None
            }
        }));
        let monster_kinds_clone = monster_kinds.clone();
        //let shared_map_ptr_clone = shared_map_ptr.clone();
        lua_interface.get_monster_kind_by_id_callback =
            Some(Rc::new(move |id| -> Option<MonsterKind> {
                //let binding = shared_map_ptr_clone.borrow();
                //let map = binding.0.borrow();
                if let Some(monster_kind) = monster_kinds_clone
                    .read()
                    .unwrap()
                    .vec
                    .read()
                    .unwrap()
                    .get(id as usize)
                {
                    let monster_kind_ref = monster_kind.as_ref();
                    let monster_kind_clone = (*monster_kind_ref).clone();
                    Some(monster_kind_clone)
                } else {
                    None
                }
            }));
        let shared_map_ptr_clone = shared_map_ptr.clone();
        lua_interface.get_current_map_callback = Some(Rc::new(move || -> MapRc {
            let binding = shared_map_ptr_clone.borrow();
            binding.clone()
        }));
    }

    let _peek_call_result = game
        .lua_interface
        .borrow_mut()
        .on_map_peeked(&current_map_rc);

    let _ = LuaInterface::register_api(&game.lua_interface);
    let mut just_update_turn = false;

    loop {
        if game.animate_for > 0.0 {
            game.animate_for -= get_frame_time();
            if game.animate_for <= 0.0 {
                game.animate_for = 0.0;
                just_update_turn = true;
            }
            let mut map = current_map_rc.0.borrow_mut();
            draw(
                &mut graphics_manager,
                &mut game,
                &mut ui,
                &mut map,
                game_interface_offset,
            );

            next_frame().await;
            continue;
        }
        game.animating_effects.clear();

        if just_update_turn {
            just_update_turn = false;
            game.last_player_event = PlayerEvent::None;
            update_turn(&mut game, &mut current_map_rc);
            //next_frame().await;
            continue;
        }

        {
            let mut map = current_map_rc.0.borrow_mut();
            for monster_ref in map.monsters.values_mut() {
                let should_call_on_spawn = {
                    let mut monster = monster_ref.borrow_mut();
                    if !monster.initialized {
                        monster.initialized = true;
                        monster.kind.is_scripted()
                    } else {
                        false
                    }
                };
                if should_call_on_spawn {
                    let r = game.lua_interface.borrow_mut().on_spawn(monster_ref);
                    if let Err(e) = r {
                        eprintln!("Error calling Lua on_spawn: {}", e);
                    }
                }
            }
        }

        while ui.events.len() > 0 {
            let mut player = game.player.borrow_mut();
            let event = ui.events.pop_front().unwrap();
            match event {
                UiEvent::IncStrength => {
                    if player.sp > 0 {
                        player.strength += 1;
                        player.sp -= 1;
                    }
                }
                UiEvent::IncDexterity => {
                    if player.sp > 0 {
                        player.dexterity += 1;
                        player.sp -= 1;
                    }
                }
                UiEvent::IncIntelligence => {
                    if player.sp > 0 {
                        player.intelligence += 1;
                        player.sp -= 1;
                    }
                }
                UiEvent::SkillPurchase(skill) => {
                    get_spell_types()
                        .get(skill as usize)
                        .and_then(|spell_opt| spell_opt.clone())
                        .map(|spell| {
                            if player.sp >= spell.cost {
                                let player_spell = PlayerSpell {
                                    spell_type: spell.clone(),
                                };
                                ui.add_player_skills(&player_spell);
                                player.spells.push(player_spell);
                                player.sp -= spell.cost;
                            } else {
                                println!("Not enough SP to purchase this skill.");
                            }
                        });
                }
                UiEvent::ChestAction(item_id) => {
                    let items_borrow = game.items.read().unwrap();
                    let item = items_borrow.items_by_id.get(&item_id);

                    if let Some(item) = item {
                        player.add_item(item.clone());
                        {
                            let mut map = current_map_rc.0.borrow_mut();
                            map.remove_chest(player.position);
                        }
                    } else {
                        println!("Item with ID {} not found.", item_id);
                        continue;
                    }

                    ui.hide();
                }
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
            &mut overworld_pos,
        );

        if !Rc::ptr_eq(&current_map_rc.0, &shared_map_ptr.borrow().0) {
            // Update the shared map pointer if it has changed
            *shared_map_ptr.borrow_mut() = current_map_rc.clone();
        }

        let now = get_time();
        if now - last_move_time < move_interval {
            {
                let mut map = current_map_rc.0.borrow_mut();
                draw(
                    &mut graphics_manager,
                    &mut game,
                    &mut ui,
                    &mut map,
                    game_interface_offset,
                );
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
                let mut map = peek_map_rc.as_mut().unwrap().0.borrow_mut();
                map.compute_player_fov(&mut game.player.borrow_mut(), max(GRID_WIDTH, GRID_HEIGHT));
                draw(
                    &mut graphics_manager,
                    &mut game,
                    &mut ui,
                    &mut map,
                    game_interface_offset,
                );
            }

            next_frame().await;
            continue;
        }

        let global_mouse_pos = PointF::new(input.mouse.x, input.mouse.y);
        let map_mouse_pos = PointF::new(
            input.mouse.x - game_interface_offset.x,
            input.mouse.y - game_interface_offset.y,
        );
        let map_hover_x = (map_mouse_pos.x / TILE_SIZE) as usize;
        let map_hover_y = ((map_mouse_pos.y) / TILE_SIZE) as usize;
        let current_tile = Position {
            x: map_hover_x,
            y: map_hover_y,
        };

        {
            let mut map = current_map_rc.0.borrow_mut();
            map.hovered_tile_changed = map.hovered_tile != Some(current_tile);
            map.hovered_tile = Some(current_tile);
        }

        ui.update_mouse_position(global_mouse_pos);

        if let Some(_click) = input.click {
            if ui.is_focused {
                ui.handle_click(global_mouse_pos);
            } else {
                goal_position = Some(current_tile)
            }
        };

        if ui.is_focused {
            if input.keyboard_action == KeyboardAction::Cancel {
                ui.hide();
            }
        } else {
            if input.keyboard_action == KeyboardAction::OpenCharacterSheet {
                ui.toggle_character_sheet();
            } else {
                update(
                    &mut game,
                    &mut current_map_rc,
                    input.keyboard_action,
                    input.direction,
                    input.spell,
                    goal_position,
                );
                let player_pos = game.player.borrow().position;

                if player_event == PlayerEvent::OpenChest {
                    if let Some(items_vec) =
                        current_map_rc.0.borrow_mut().get_chest_items(&player_pos)
                    {
                        let actual_items: Vec<(u32, String)> = items_vec
                            .iter()
                            .filter_map(|item_id| {
                                game.items
                                    .read()
                                    .unwrap()
                                    .items_by_id
                                    .get(item_id)
                                    .map(|item| (item.id(), item.name().to_string()))
                            })
                            .collect();

                        ui.show_chest_view(&actual_items);
                    }
                } else if player_event == PlayerEvent::ReachBorder {
                    map_update = MapTravelEvent::Peek(MapTravelKind::BorderCross);
                } else if player_event == PlayerEvent::ClimbDown {
                    map_update = MapTravelEvent::Peek(MapTravelKind::ClimbDown);
                }
            }
        }

        {
            let mut map = current_map_rc.0.borrow_mut();
            if game.last_player_event == PlayerEvent::AutoMove {
                last_move_time = now; // Update last move time for auto step
            } else {
                goal_position = None;
            }

            draw(
                &mut graphics_manager,
                &mut game,
                &mut ui,
                &mut map,
                game_interface_offset,
            );
        }

        next_frame().await;
    }
}

pub fn update(
    game: &mut GameState,
    map_ref: &mut MapRc,
    player_action: KeyboardAction,
    player_direction: Direction,
    spell_action: i32,
    player_goal_position: Option<Position>,
) {
    game.last_player_event = PlayerEvent::None;
    let player_pos = { game.player.borrow().position };

    let mut new_player_pos: Option<Position> = None;
    let mut should_update_turn = false;

    if let Some(player_goal) = player_goal_position {
        let spell_index = { game.player.borrow().selected_spell };

        if let Some(index) = spell_index {
            if index == u8::MAX {
                game.last_player_event = PlayerEvent::AttackChooseTarget;
                let can_attack = {
                    let weapon = { game.player.borrow().equipment.weapon.clone() };
                    let range = {
                        if let Some(weapon) = weapon {
                            weapon.range.unwrap_or(1)
                        } else {
                            1 // Default range if no weapon
                        }
                    };
                    map_ref.0.borrow().is_tile_enemy_occupied(player_goal)
                        && (player_pos.euclidean_distance_squared(&player_goal)
                            <= (range * range) as f64
                            || player_pos.is_neighbor(&player_goal))
                        && game
                            .player
                            .borrow_mut()
                            .line_of_sight
                            .contains(&player_goal)
                };

                if can_attack {
                    combat::do_melee_combat(
                        &mut game.player,
                        map_ref,
                        player_pos,
                        player_goal,
                        &game.lua_interface,
                    );
                    should_update_turn = true;
                    game.last_player_event = PlayerEvent::AttackConfirm;
                }
            } else {
                let mut should_cast = false;
                {
                    let mut player = game.player.borrow_mut();

                    let (in_line_of_sight, spell_range) = {
                        let in_line_of_sight = player.line_of_sight.contains(&player_goal);
                        let spell_range = player
                            .spells
                            .get(index as usize)
                            .expect("Selected spell index out of bounds")
                            .spell_type
                            .range;
                        (in_line_of_sight, spell_range)
                    };

                    if in_line_of_sight
                        && (spell_range.is_none()
                            || player_pos.in_range(&player_goal, spell_range.unwrap() as usize))
                    {
                        if let Some(spell) = player.spells.get_mut(index as usize) {
                            let mp_cost = spell.spell_type.mp_cost;
                            if player.mp > mp_cost {
                                player.mp -= mp_cost;
                                println!("Casting spell");
                                should_cast = true;
                            } else {
                                println!("Not enough mp for this spell!");
                            }
                        }
                    }
                }

                if should_cast {
                    let spell_type = {
                        let mut player_ref = game.player.borrow_mut();
                        player_ref
                            .spells
                            .get_mut(index as usize)
                            .expect("Selected spell index out of bounds")
                            .spell_type
                            .clone()
                    };
                    let positions = combat::do_spell_combat(
                        &mut game.player,
                        map_ref,
                        player_pos,
                        player_goal,
                        &spell_type,
                        &game.lua_interface,
                    );
                    game.animate_for = 0.2;

                    if let Some(sprite) = &spell_type.sprite {
                        for pos in positions {
                            game.animating_effects.insert(pos, sprite.clone());
                        }
                    }

                    game.last_player_event = PlayerEvent::SpellCast;
                    let mut player = game.player.borrow_mut();
                    player.selected_spell = None;
                    player.goal_position = None;
                    return;
                }
            }

            let mut player = game.player.borrow_mut();
            player.selected_spell = None;
            player.goal_position = None; // Clear goal position
        } else {
            let attack = {
                let player = game.player.borrow();
                if let Some(weapon) = &player.equipment.weapon {
                    let range = weapon.range.unwrap_or(1);
                    let map = map_ref.0.borrow();
                    if map.is_tile_enemy_occupied(player_goal) {
                        (player_pos.euclidean_distance_squared(&player_goal)
                            <= (range * range) as f64
                            || player_pos.is_neighbor(&player_goal))
                            && player.line_of_sight.contains(&player_goal)
                    } else {
                        false
                    }
                } else {
                    false
                }
            };

            if attack {
                should_update_turn = true; // Update monsters if player attacks
                combat::do_melee_combat(
                    &mut game.player,
                    map_ref,
                    player_pos,
                    player_goal,
                    &game.lua_interface,
                );
            } else {
                let path: Option<Vec<Position>> =
                    Navigator::find_path(player_pos, player_goal, |pos| {
                        map_ref.0.borrow().is_tile_walkable(pos)
                    });

                if let Some(path) = path {
                    if path.len() > 1 {
                        new_player_pos = Some(path[1]);
                        game.last_player_event = PlayerEvent::AutoMove;
                    } else {
                        game.last_player_event = PlayerEvent::AutoMoveEnd;
                    }
                    game.player.borrow_mut().goal_position = player_goal_position;
                } else {
                    game.player.borrow_mut().goal_position = None; // Clear goal if no path found
                    game.last_player_event = PlayerEvent::AutoMoveEnd;
                }
            }
        }
    } else if player_action == KeyboardAction::AttackChooseTarget {
        game.last_player_event = PlayerEvent::AttackChooseTarget;
        game.player.borrow_mut().selected_spell = Some(u8::MAX);
    } else if player_action == KeyboardAction::SpellSelect && spell_action > 0 {
        let index = spell_action as usize - 1;
        let mut player = game.player.borrow_mut();

        let spell_name = {
            player
                .spells
                .get(index)
                .map(|spell| spell.spell_type.name.clone())
        };
        if let Some(name) = spell_name {
            player.selected_spell = Some(index as u8);
            println!("Spell selected: {}", name);
        } else {
            println!("No spell selected!");
        }

        game.last_player_event = PlayerEvent::SpellSelect;
    } else if player_action == KeyboardAction::Cancel {
        let mut player = game.player.borrow_mut();
        player.selected_spell = None;
        player.goal_position = None; // Clear goal position

        game.last_player_event = PlayerEvent::Cancel;
    } else if player_action == KeyboardAction::Move {
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
            y: (player_pos.y as isize + pos_change.1) as usize,
        };

        {
            let map = map_ref.0.borrow();
            if map.is_tile_enemy_occupied(pos) {
                game.last_player_event = PlayerEvent::MeleeAttack;
                should_update_turn = true; // Update monsters if player attacks
                drop(map);
                combat::do_melee_combat(
                    &mut game.player,
                    map_ref,
                    player_pos,
                    pos,
                    &game.lua_interface,
                );
            } else if map.generated_map.tiles[pos].is_border(&pos) && !map.monsters.is_empty() {
                game.last_player_event = PlayerEvent::Cancel;
            } else {
                if map.is_tile_walkable(pos) {
                    new_player_pos = Some(pos);
                    should_update_turn = true; // Update monsters if player moves
                }

                game.last_player_event = PlayerEvent::Move;
            }
        }

        game.player.borrow_mut().goal_position = None;
    } else if player_action == KeyboardAction::Wait {
        new_player_pos = Some(player_pos); // Stay in place
        game.player.borrow_mut().goal_position = None; // Clear goal position

        game.last_player_event = PlayerEvent::Wait;
    }

    if let Some(pos) = new_player_pos {
        let mut map = map_ref.0.borrow_mut();
        map.generated_map.tiles[player_pos].creature = NO_CREATURE;
        map.generated_map.tiles[pos].creature = PLAYER_CREATURE_ID;

        let mut player = game.player.borrow_mut();
        player.set_pos(pos);

        if new_player_pos == player.goal_position {
            player.goal_position = None; // Clear goal position if reached
        }

        map.compute_player_fov(&mut player, max(GRID_WIDTH, GRID_HEIGHT));
        should_update_turn = true;

        let mut to_remove: Vec<usize> = Vec::new();

        for (idx, item) in map.generated_map.tiles[pos].items.iter().rev().enumerate() {
            match item {
                ItemKind::Orb(_) => {
                    println!("Player picked up an orb at index {idx}!");
                    player.sp += 1;
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
            map.generated_map.tiles[pos].remove_item(idx);
        }

        if map.generated_map.tiles[pos].is_border(&pos) {
            game.last_player_event = PlayerEvent::ReachBorder;
        }
    }

    if should_update_turn {
        update_turn(game, map_ref);
    }
}

pub fn update_turn(game: &mut GameState, map_ref: &MapRc) {
    let mut player = game.player.borrow_mut();
    if player.accumulated_speed >= 200 {
        player.accumulated_speed -= 100;
        return;
    }
    let mut player_accumulated_speed = player.accumulated_speed;
    let player_speed = player.get_speed();
    let player_pos = player.position.clone();
    drop(player);

    while player_accumulated_speed < 100 {
        let map = map_ref.0.borrow_mut();
        let mut monsters = map.monsters.clone(); // Clone the monsters to avoid borrowing conflicts
        drop(map);

        let mut update_monsters_again = true;
        let mut update_iteration = 0;

        while update_monsters_again {
            update_monsters_again = false;
            for (id, monster_ref) in &mut monsters {
                let monster = monster_ref.borrow_mut();
                if monster.hp <= 0 {
                    continue; // Skip dead monsters
                }

                if monster.kind.is_scripted() {
                    drop(monster);
                    let mut clone = monster_ref.clone();
                    let r = game
                        .lua_interface
                        .borrow_mut()
                        .on_update(&mut clone, update_iteration);
                    if let Err(e) = r {
                        eprintln!("Error calling Lua on_update: {}", e);
                    } else {
                        if r.unwrap() {
                            // If the Lua script returned true, we skip the rest of the update for this monster
                            // The update has already been handled in Lua
                            continue;
                        }
                    }
                    drop(clone);
                } else {
                    drop(monster); // Just drop the immutable borrow
                }

                let mut monster = monster_ref.borrow_mut();
                let mut monster_speed = monster.kind.speed + monster.accumulated_speed;

                if monster_speed >= 100 {
                    monster_speed -= 100;

                    let monster_pos = monster.pos();
                    let path =
                        find_monster_path(map_ref, monster_pos, player_pos, monster.kind.flying);

                    if let Some(path) = path {
                        if path.len() > 1 {
                            let next_step = path[1];

                            if next_step == player_pos {
                                println!(
                                    "Monster {} hit player for {} damage!",
                                    monster.name(),
                                    monster.kind.melee_damage
                                );
                                {
                                    let mut player = game.player.borrow_mut();
                                    player.add_health(-monster.kind.melee_damage);
                                    if player.hp <= 0 {
                                        println!("Player has been defeated!");
                                        game.last_player_event = PlayerEvent::Death;
                                        return;
                                    }
                                }

                                continue;
                            }

                            //monster_moves.push((monster_pos, next_step, *id as usize));
                            monster.set_pos(next_step);

                            let mut map = map_ref.0.borrow_mut();
                            map.generated_map.tiles[monster_pos].creature = NO_CREATURE;
                            map.generated_map.tiles[next_step].creature = *id;
                        }
                    }
                }

                monster.accumulated_speed = monster_speed as u32;

                if monster_speed >= 100 {
                    update_monsters_again = true;
                }
            }
            update_iteration += 1;
        }

        game.turn += 1;
        player_accumulated_speed += player_speed;
    }
    player_accumulated_speed -= 100;
    game.player.borrow_mut().accumulated_speed = player_accumulated_speed;
}

fn find_monster_path(
    map_ref: &MapRc,
    monster_pos: Position,
    player_pos: Position,
    flying: bool,
) -> Option<Vec<Position>> {
    Navigator::find_path(monster_pos, player_pos, |pos| {
        if pos.x >= GRID_WIDTH || pos.y >= GRID_HEIGHT {
            return false;
        }
        // borrow the map _immutably_ each time to see current occupancy:
        let map = map_ref.0.borrow();

        if flying {
            return !map.generated_map.tiles[pos].is_blocking();
        } else {
            return map.generated_map.tiles[pos].is_walkable();
        }
    })
}
fn teleport_creature_to(player: &PlayerRc, map_rc: &MapRc, creature_id: u32, pos: Position) {
    let mut map = map_rc.0.borrow_mut();
    if creature_id == PLAYER_CREATURE_ID as u32 {
        let mut player_ref = player.borrow_mut();
        player_ref.position = pos;
    } else if let Some(monster) = map.monsters.get(&creature_id) {
        let mut monster_ref = monster.borrow_mut();
        monster_ref.position = pos;
        drop(monster_ref);
        map.generated_map.tiles[pos].creature = creature_id;
    }
}
