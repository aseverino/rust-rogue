mod game;
mod map;
mod creature;
mod monster_type;
mod monster;
mod player;
use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "OpenRift".to_string(),
        window_width: 1000,
        window_height: 720,
        fullscreen: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    game::run().await;
}