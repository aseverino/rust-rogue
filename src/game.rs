use macroquad::prelude::*;
use crate::map::{Map, Tile};
use crate::player::{Player};
use crate::creature::Creature;
use crate::monster_type::load_monster_types;
use std::rc::Rc;

pub struct GameState {
    pub map: Map,
    pub player: Player,
}

pub async fn run() {
    let mut game = GameState {
        map: Map::generate(),
        player: Player::new(1, 1),
    };

    let monster_types = load_monster_types().await;

    let monsters = game.map.add_random_monsters(&monster_types, 10);

    loop {
        clear_background(BLACK);

        // draw_text("OpenRift - Procedural Map", 10.0, 20.0, 30.0, WHITE);

        game.map.draw();
        for monster in &monsters {
            monster.draw();
        }

        game.player.handle_input(&game.map);
        game.player.draw();

        next_frame().await;
    }
}