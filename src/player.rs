use macroquad::prelude::*;
use crate::map::{TILE_SIZE, GRID_WIDTH, GRID_HEIGHT, Map};
use crate::creature::Creature;
use crate::position::Position;
use crate::map::Direction;

#[derive(PartialEq)]
pub enum Action {
    None,
    Move,
    Wait,
}

pub struct Player {
    pub name: String,
    pub position: Position,
    pub action: Action,
}

impl Player {
    pub fn new(x: usize, y: usize) -> Self {
        Self {
            name: "Player".into(),
            position: Position {
                x,
                y,
            },
            action: Action::None,
        }
    }

    pub fn handle_input(&self, map: &Map) -> (Action, Direction) {
        let mut action = Action::None;
        let mut direction = Direction::None;

        if (is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::Kp6)) && self.position.x < GRID_WIDTH - 1 && map.is_walkable(self.position.x + 1, self.position.y) {
            action = Action::Move;
            direction = Direction::Right;
        }
        if (is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::Kp4)) && self.position.x > 0 && map.is_walkable(self.position.x - 1, self.position.y) {
            action = Action::Move;
            direction = Direction::Left;
        }
        if (is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::Kp8)) && self.position.y > 0 && map.is_walkable(self.position.x, self.position.y - 1) {
            action = Action::Move;
            direction = Direction::Up;
        }
        if (is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::Kp2)) && self.position.y < GRID_HEIGHT - 1 && map.is_walkable(self.position.x, self.position.y + 1) {
            action = Action::Move;
            direction = Direction::Down;
        }
        if is_key_pressed(KeyCode::Kp7) && self.position.x > 0 && self.position.y > 0 && map.is_walkable(self.position.x - 1, self.position.y - 1) {
            action = Action::Move;
            direction = Direction::UpLeft;
        }
        if is_key_pressed(KeyCode::Kp9) && self.position.x < GRID_WIDTH - 1 && self.position.y > 0 && map.is_walkable(self.position.x + 1, self.position.y - 1) {
            action = Action::Move;
            direction = Direction::UpRight;
        }
        if is_key_pressed(KeyCode::Kp1) && self.position.x > 0 && self.position.y < GRID_HEIGHT - 1 && map.is_walkable(self.position.x - 1, self.position.y + 1) {
            action = Action::Move;
            direction = Direction::DownLeft;
        }
        if is_key_pressed(KeyCode::Kp3) && self.position.x < GRID_WIDTH - 1 && self.position.y < GRID_HEIGHT - 1 && map.is_walkable(self.position.x + 1, self.position.y + 1) {
            action = Action::Move;
            direction = Direction::DownRight;
        }
        if is_key_pressed(KeyCode::Kp5) {
            action = Action::Wait;
        }

        (action, direction)
    }
}

impl Creature for Player {
    fn name(&self) -> &str {
        &self.name
    }

    fn pos(&self) -> Position {
        self.position
    }

    fn set_pos(&mut self, pos: Position) {
        self.position = pos;
    }

    fn draw(&self) {
        // Base colored rectangle
        draw_rectangle(
            self.position.x as f32 * TILE_SIZE + 4.0,
            self.position.y as f32 * TILE_SIZE + 44.0,
            TILE_SIZE - 8.0,
            TILE_SIZE - 8.0,
            BLUE,
        );

        // Glyph overlay
        draw_text(
            "@",
            self.position.x as f32 * TILE_SIZE + 10.0,
            self.position.y as f32 * TILE_SIZE + 60.0,
            18.0,
            WHITE,
        );
    }

    fn is_player(&self) -> bool {
        true
    }
}