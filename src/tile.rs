use crate::position::Position;

pub const NO_CREATURE: i32 = -1;
pub const PLAYER_CREATURE_ID: i32 = i32::MAX; // or any large unique value

#[derive(Copy, Clone, PartialEq)]
pub enum TileKind {
    Chasm,
    Wall,
    Floor,
}

#[derive(Clone)]
pub struct Tile {
    pub position: Position,
    pub kind: TileKind,
    pub creature: i32, // Index of creatures on this tile
}

impl Tile {
    pub fn new(pos: Position, kind: TileKind) -> Self {
        Self { position: pos, kind, creature: NO_CREATURE }
    }

    pub fn is_walkable(&self) -> bool {
        self.kind == TileKind::Floor && (self.creature == NO_CREATURE || self.creature == PLAYER_CREATURE_ID)
    }
}