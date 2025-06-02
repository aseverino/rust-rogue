use macroquad::prelude::*;
extern crate rand as external_rand;

use external_rand::Rng;
use external_rand::thread_rng;

use std::collections::HashMap;
use crate::creature::Creature;
use crate::monster::Monster;
use crate::monster_type::MonsterType;
use crate::player::Player;
use external_rand::seq::SliceRandom;
use std::rc::Rc;
use std::cell::RefCell;

pub const TILE_SIZE: f32 = 32.0;
pub const GRID_WIDTH: usize = 30;
pub const GRID_HEIGHT: usize = 20;

#[derive(Copy, Clone, PartialEq)]
pub enum Tile {
    Wall,
    Floor,
    Chasm,
}

pub struct Map {
    pub tiles: Vec<Vec<Tile>>,
    pub walkable: Vec<(usize, usize)>,
    pub player: Rc<RefCell<Player>>,
    pub monsters: Vec<Rc<Monster>>,
}

impl Map {
    pub fn generate(player: Rc<RefCell<Player>>, monster_types: &HashMap<String, Rc<MonsterType>>) -> Self {
        let mut rng = thread_rng();
        let mut tiles = vec![vec![Tile::Wall; GRID_WIDTH]; GRID_HEIGHT];
        let mut walkable= Vec::new();

        for y in 1..GRID_HEIGHT - 1 {
            for x in 1..GRID_WIDTH - 1 {
                let roll = rng.gen_range(0..100);
                tiles[y][x] = match roll {
                    0..=65 => Tile::Floor,
                    66..=85 => Tile::Wall,
                    _ => Tile::Chasm,
                };

                if tiles[y][x] == Tile::Floor {
                    walkable.push((x, y));
                }
            }
        }

        tiles[1][1] = Tile::Floor;

        let mut map = Self { tiles, walkable, monsters: Vec::new(), player };
        map.add_random_monsters(monster_types, 10);
        map
    }

    pub fn draw(&self) {
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                let tile = self.tiles[y][x];
                let color = match tile {
                    Tile::Floor => DARKGREEN,
                    Tile::Wall => GRAY,
                    Tile::Chasm => DARKBLUE,
                };

                draw_rectangle(
                    x as f32 * TILE_SIZE,
                    y as f32 * TILE_SIZE + 40.0,
                    TILE_SIZE - 1.0,
                    TILE_SIZE - 1.0,
                    color,
                );
            }
        }

        for monster in &self.monsters {
            monster.draw();
        }

        self.player.borrow().draw();
    }

    pub fn is_walkable(&self, x: usize, y: usize) -> bool {
        x < GRID_WIDTH && y < GRID_HEIGHT && self.tiles[y][x] == Tile::Floor
    }
    
    pub(crate) fn add_random_monsters(
        &mut self,
        monster_types: &HashMap<String, Rc<MonsterType>>,
        count: usize,
    ) {
        let mut rng = thread_rng();

        let mut positions = self.walkable.clone(); // clone so we can shuffle safely
        // 2. Shuffle the positions randomly
        positions.shuffle(&mut rng);

        // 3. Pick up to `count` positions
        let positions = positions.into_iter().take(count);

        let all_types: Vec<_> = monster_types.values().cloned().collect();

        for (x, y) in positions {
            let kind = all_types
                .choose(&mut rng)
                .expect("Monster type list is empty")
                .clone();

            // Wrap the monster in Rc and push to creatures
            self.monsters.push(Rc::new(Monster::new(x, y, kind)));
        }
    }
}