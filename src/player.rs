use macroquad::prelude::*;
use crate::map::{TILE_SIZE, GRID_WIDTH, GRID_HEIGHT, Map};
use crate::creature::Creature;

pub struct Player {
    pub name: String,
    pub x: usize,
    pub y: usize,
    pub acted: bool,
}

impl Player {
    pub fn new(x: usize, y: usize) -> Self {
        Self {
            name: "Player".into(),
            x,
            y,
            acted: false,
        }
    }

    pub fn handle_input(&mut self, map: &Map) {
        if is_key_pressed(KeyCode::Right) && self.x < GRID_WIDTH - 1 && map.is_walkable(self.x + 1, self.y) {
            self.x += 1;
            self.acted = true;
        }
        if is_key_pressed(KeyCode::Left) && self.x > 0 && map.is_walkable(self.x - 1, self.y) {
            self.x -= 1;
            self.acted = true;
        }
        if is_key_pressed(KeyCode::Down) && self.y < GRID_HEIGHT - 1 && map.is_walkable(self.x, self.y + 1) {
            self.y += 1;
            self.acted = true;
        }
        if is_key_pressed(KeyCode::Up) && self.y > 0 && map.is_walkable(self.x, self.y - 1) {
            self.y -= 1;
            self.acted = true;
        }
    }
}

impl Creature for Player {
    fn name(&self) -> &str {
        &self.name
    }

    fn pos(&self) -> (usize, usize) {
        (self.x, self.y)
    }

    fn set_pos(&mut self, x: usize, y: usize) {
        self.x = x;
        self.y = y;
    }

    fn draw(&self) {
        // Base colored rectangle
        draw_rectangle(
            self.x as f32 * TILE_SIZE + 4.0,
            self.y as f32 * TILE_SIZE + 44.0,
            TILE_SIZE - 8.0,
            TILE_SIZE - 8.0,
            BLUE,
        );

        // Glyph overlay
        draw_text(
            "@",
            self.x as f32 * TILE_SIZE + 10.0,
            self.y as f32 * TILE_SIZE + 60.0,
            18.0,
            WHITE,
        );
    }

    fn is_player(&self) -> bool {
        true
    }
}