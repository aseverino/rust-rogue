use macroquad::prelude::*;
use crate::map::{Map, Tile};
use crate::player::{Player};
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
    let game = GameState {
        player: player.clone(),
        map: Map::generate(player.clone(), &monster_types),
    };

    //let monsters = game.map.add_random_monsters(&monster_types, 10);

    loop {
        clear_background(BLACK);

        // draw_text("OpenRift - Procedural Map", 10.0, 20.0, 30.0, WHITE);

        game.map.draw();

        game.player.borrow_mut().handle_input(&game.map);
        //game.player.draw();

        next_frame().await;
    }
}