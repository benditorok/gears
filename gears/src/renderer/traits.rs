pub(crate) trait Pos {
    fn get_pos(&self) -> cgmath::Point3<f32>;
}

pub(crate) trait Collider {
    fn intersects(&self, other: &Self) -> bool;
    fn move_to(&mut self, pos: impl Pos);
}
