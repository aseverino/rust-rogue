use crate::position::Position;

pub trait Creature {
    fn name(&self) -> &str;
    fn pos(&self) -> Position;
    fn set_pos(&mut self, pos: Position);
    fn draw(&self);

    fn is_player(&self) -> bool { false }
    fn is_monster(&self) -> bool { false }
}