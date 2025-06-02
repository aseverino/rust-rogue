use crate::map::TILE_SIZE;
use macroquad::prelude::*;
use crate::creature::Creature;
use crate::monster_type::MonsterType;
use std::rc::Rc;

pub struct Monster {
    pub x: usize,
    pub y: usize,
    pub hp: i32,
    pub kind: Rc<MonsterType>
}

impl Monster {
    pub fn new(x: usize, y: usize, kind: Rc<MonsterType>) -> Self {
        Self {
            x,
            y,
            hp: kind.max_hp,
            kind,
        }
    }
}

impl Creature for Monster {
    fn name(&self) -> &str {
        &self.kind.name
    }

    fn pos(&self) -> (usize, usize) {
        (self.x, self.y)
    }

    fn set_pos(&mut self, x: usize, y: usize) {
        self.x = x;
        self.y = y;
    }

    fn draw(&self) {
        draw_rectangle(
            self.x as f32 * TILE_SIZE + 8.0,
            self.y as f32 * TILE_SIZE + 48.0,
            TILE_SIZE - 16.0,
            TILE_SIZE - 16.0,
            self.kind.color(),
        );

        // Optional glyph drawing
        let glyph = self.kind.glyph.to_string();
        draw_text(&glyph, self.x as f32 * TILE_SIZE + 12.0, self.y as f32 * TILE_SIZE + 60.0, 16.0, WHITE);
    }

    fn is_monster(&self) -> bool { true }
}
