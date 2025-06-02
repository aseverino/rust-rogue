use macroquad::prelude::*;
use crate::map::{Map, GRID_WIDTH, GRID_HEIGHT, TILE_SIZE};
use crate::player::{KeyboardAction, Player};
use crate::position::Position;
use crate::creature::Creature;
use crate::monster_type::load_monster_types;

use std::rc::Rc;
use std::cell::RefCell;

pub struct GameState {
    pub map: Map,
    pub player: Rc<RefCell<Player>>,
}

pub async fn run() {
    let monster_types = load_monster_types().await;
    let player = Rc::new(RefCell::new(Player::new(1, 1)));
    let mut game = GameState {
        player: player.clone(),
        map: Map::generate(player.clone(), &monster_types),
    };

    let mut mouse_down_tile: Option<Position> = None;

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

        if hover_x < GRID_WIDTH && hover_y < GRID_HEIGHT {
            game.map.hovered = Some(current_tile);
        } else {
            game.map.hovered = None;
        }

        if is_mouse_button_released(MouseButton::Left) {
            if let Some(down_tile) = mouse_down_tile.take() {
                if down_tile == current_tile {
                    // A full click on the same tile — treat as a click!
                    if down_tile.x < GRID_WIDTH && down_tile.y < GRID_HEIGHT
                        && game.map.is_walkable(down_tile.x, down_tile.y)
                    {
                        goal_position = Some(down_tile);
                    }
                }
            }
        }

        // draw_text("OpenRift - Procedural Map", 10.0, 20.0, 30.0, WHITE);
        game.player.borrow_mut().keyboard_action = KeyboardAction::None;
        let (keyboard_action, direction) = game.player.borrow_mut().handle_input(&game.map);

        if keyboard_action != KeyboardAction::None || goal_position.is_some() {
            game.map.update(keyboard_action, direction, goal_position);
        }
        game.map.draw();
        //game.player.draw();

        next_frame().await;
    }
}