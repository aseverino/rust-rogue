pub trait Creature {
    fn name(&self) -> &str;
    fn pos(&self) -> (usize, usize);
    fn set_pos(&mut self, x: usize, y: usize);
    fn draw(&self);

    fn is_player(&self) -> bool { false }
    fn is_monster(&self) -> bool { false }
}