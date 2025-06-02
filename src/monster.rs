use crate::map::TILE_SIZE;
use macroquad::prelude::*;
use crate::creature::Creature;
use crate::position::Position;
use crate::monster_type::MonsterType;
use std::rc::Rc;

pub struct Monster {
    pub hp: i32,
    pub kind: Rc<MonsterType>,
    pub position: Position,
}

impl Monster {
    pub fn new(pos: Position, kind: Rc<MonsterType>) -> Self {
        Self {
            position: pos,
            hp: kind.max_hp,
            kind,
        }
    }
}

impl Creature for Monster {
    fn name(&self) -> &str {
        &self.kind.name
    }

    fn pos(&self) -> Position {
        self.position
    }

    fn set_pos(&mut self, pos: Position) {
        self.position = pos;
    }

    fn draw(&self) {
        draw_rectangle(
            self.position.x as f32 * TILE_SIZE + 8.0,
            self.position.y as f32 * TILE_SIZE + 48.0,
            TILE_SIZE - 16.0,
            TILE_SIZE - 16.0,
            self.kind.color(),
        );

        // Optional glyph drawing
        let glyph = self.kind.glyph.to_string();
        draw_text(&glyph, self.position.x as f32 * TILE_SIZE + 12.0, self.position.y as f32 * TILE_SIZE + 60.0, 16.0, WHITE);
    }

    fn is_monster(&self) -> bool { true }
}
