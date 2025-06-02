use macroquad::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use serde_json::from_str;
use std::rc::Rc;

pub async fn load_monster_types() -> HashMap<String, Rc<MonsterType>> {
    let file = load_string("assets/monsters.json").await.unwrap();
    let list: Vec<MonsterType> = from_str(&file).unwrap();

    list.into_iter()
        .map(|mt| (mt.name.clone(), Rc::new(mt)))
        .collect()
}

#[derive(Debug, Deserialize)]
pub struct MonsterType {
    pub name: String,
    pub glyph: char,
    pub color: [u8; 3], // RGB, will convert to macroquad::Color
    pub max_hp: i32,
}

impl MonsterType {
    pub fn color(&self) -> Color {
        Color::from_rgba(self.color[0], self.color[1], self.color[2], 255)
    }
}