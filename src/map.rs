use macroquad::prelude::*;
extern crate rand as external_rand;

use external_rand::Rng;
use external_rand::thread_rng;

use std::collections::HashMap;
use crate::creature::Creature;
use crate::monster::Monster;
use crate::monster_type::MonsterType;
use crate::position::Position;
use crate::player::{Player, Action};
use crate::tile::{Tile, TileKind, NO_CREATURE, PLAYER_CREATURE_ID};
use external_rand::seq::SliceRandom;
use std::rc::Rc;
use std::cell::RefCell;
use pathfinding::prelude::astar;

pub const TILE_SIZE: f32 = 32.0;
pub const GRID_WIDTH: usize = 30;
pub const GRID_HEIGHT: usize = 20;

pub enum Direction {
    Up,
    Right,
    Down,
    Left,
    UpRight,
    DownRight,
    DownLeft,
    UpLeft,
    None,
}

pub struct Map {
    pub tiles: Vec<Vec<Tile>>,
    pub walkable_cache: Vec<Position>,
    pub player: Rc<RefCell<Player>>,
    pub monsters: Vec<Rc<RefCell<Monster>>>,
}

pub fn find_path<F>(start_pos: Position, goal_pos: Position, is_walkable: F) -> Option<Vec<Position>>
where
    F: Fn(Position) -> bool,
{
    let start = (start_pos.x, start_pos.y);
    let goal = (goal_pos.x, goal_pos.y);
    let path = astar(
        &start,
        |&(x, y)| {
            let mut neighbors = Vec::new();
            let directions = [
                (0isize, -1), (1, 0), (0, 1), (-1, 0), // cardinal
                (-1, -1), (1, -1), (1, 1), (-1, 1),    // diagonals
            ];

            for (dx, dy) in directions {
                let nx = x as isize + dx;
                let ny = y as isize + dy;
                if nx >= 0 && ny >= 0 {
                    let (ux, uy) = (nx as usize, ny as usize);
                    if is_walkable(Position { x: ux, y: uy }) {
                        // Diagonal steps have slightly higher cost
                        let cost = if dx != 0 && dy != 0 { 14 } else { 10 };
                        neighbors.push(((ux, uy), cost));
                    }
                }
            }

            neighbors
        },
        |&(x, y)| {
            let dx = (goal.0 as isize - x as isize).abs();
            let dy = (goal.1 as isize - y as isize).abs();
            10 * (dx + dy) - 6 * dx.min(dy) // diagonal heuristic
        },
        |&pos| pos == goal,
    )
    .map(|(path, _)| path);

    path.map(|p| p.into_iter().map(|(x, y)| Position { x, y }).collect())
}

impl Map {
    pub fn generate(player: Rc<RefCell<Player>>, monster_types: &HashMap<String, Rc<MonsterType>>) -> Self {
        let mut rng = thread_rng();
        let mut walkable_cache= Vec::new();

        let mut tiles = Vec::with_capacity(GRID_WIDTH);
        for x in 0..GRID_WIDTH {
            let mut column = Vec::with_capacity(GRID_HEIGHT);
            for y in 0..GRID_HEIGHT {
                let roll = rng.gen_range(0..100);
                let kind = match roll {
                    0..=65 => TileKind::Floor,
                    66..=85 => TileKind::Wall,
                    _ => TileKind::Chasm,
                };

                let pos = Position { x, y };
                
                if kind == TileKind::Floor {
                    walkable_cache.push(pos);
                }
                column.push(Tile::new(pos, kind));
            }
            tiles.push(column);
        }

        tiles[1][1].kind = TileKind::Floor;

        let mut map = Self { tiles, walkable_cache, monsters: Vec::new(), player };
        map.add_random_monsters(monster_types, 10);
        map
    }

    pub fn draw(&self) {
        for x in 0..GRID_WIDTH {
            for y in 0..GRID_HEIGHT {
                let tile = &self.tiles[x][y];
                let color = match tile.kind {
                    TileKind::Floor => DARKGREEN,
                    TileKind::Wall => GRAY,
                    TileKind::Chasm => DARKBLUE,
                };

                draw_rectangle(
                    x as f32 * TILE_SIZE,
                    y as f32 * TILE_SIZE + 40.0,
                    TILE_SIZE - 1.0,
                    TILE_SIZE - 1.0,
                    color,
                );

                if self.tiles[x][y].creature != NO_CREATURE {
                        draw_rectangle(
                        x as f32 * TILE_SIZE,
                        y as f32 * TILE_SIZE + 40.0,
                        TILE_SIZE - 1.0,
                        TILE_SIZE - 1.0,
                        Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 },
                    );
                }
            }
        }

        for monster in &self.monsters {
            monster.borrow().draw();
        }

        self.player.borrow().draw();
    }

    pub fn is_walkable(&self, x: usize, y: usize) -> bool {
        x < GRID_WIDTH && y < GRID_HEIGHT && self.tiles[x][y].is_walkable()
    }
    
    pub(crate) fn add_random_monsters(
        &mut self,
        monster_types: &HashMap<String, Rc<MonsterType>>,
        count: usize,
    ) {
        let mut rng = thread_rng();

        let mut positions = self.walkable_cache.clone(); // clone so we can shuffle safely
        // 2. Shuffle the positions randomly
        positions.shuffle(&mut rng);

        // 3. Pick up to `count` positions
        let positions = positions.into_iter().take(count);

        let all_types: Vec<_> = monster_types.values().cloned().collect();

        for pos in positions {
            let kind = all_types
                .choose(&mut rng)
                .expect("Monster type list is empty")
                .clone();

            // Wrap the monster in Rc and push to creatures
            self.monsters.push(Rc::new(RefCell::new(Monster::new(pos, kind))));
        }
    }

    pub fn update(&mut self, player_action: Action, player_direction: Direction) {
        let player_pos = self.player.borrow().pos();

        let new_player_pos = match player_action {
            Action::Move => {
                let pos = match player_direction {
                    Direction::Up => (0, -1),
                    Direction::Right => (1, 0),
                    Direction::Down => (0, 1),
                    Direction::Left => (-1, 0),
                    Direction::UpRight => (1, -1),
                    Direction::DownRight => (1, 1),
                    Direction::DownLeft => (-1, 1),
                    Direction::UpLeft => (-1, -1),
                    Direction::None => (0, 0),
                };

                Position {
                    x: (player_pos.x as isize + pos.0) as usize,
                    y: (player_pos.y as isize + pos.1) as usize
                }
            }
            Action::Wait => player_pos,
            _ => player_pos,
        };

        self.tiles[player_pos.x][player_pos.y].creature = NO_CREATURE;
        self.tiles[new_player_pos.x][new_player_pos.y].creature = PLAYER_CREATURE_ID;

        self.player.borrow_mut().set_pos(new_player_pos);

        for (i, monster) in self.monsters.iter().enumerate() {
            let mut m = monster.borrow_mut();
            let monster_pos = m.pos();

            let path = find_path(monster_pos, new_player_pos, |pos| {
                pos.x < GRID_WIDTH && pos.y < GRID_HEIGHT && self.tiles[pos.x][pos.y].is_walkable()
            });

            if let Some(path) = path {
                if path.len() > 1 {
                    let next_step = path[1];

                    if next_step == new_player_pos {
                        m.hp -= 1;
                        println!("Monster {} hit!", m.name());
                        if m.hp <= 0 {
                            self.tiles[monster_pos.x][monster_pos.y].creature = NO_CREATURE;
                            // self.monsters.remove(i);
                            // todo death
                        }
                        continue;
                    }

                    self.tiles[monster_pos.x][monster_pos.y].creature = NO_CREATURE;
                    self.tiles[next_step.x][next_step.y].creature = i as i32;
                    
                    m.set_pos(next_step);
                }
            }
        }
    }
}