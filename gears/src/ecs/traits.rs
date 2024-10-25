use super::Entity;

/// A component that can be attached to an entity.
pub trait Component: 'static + Send + Sync {}

pub trait EntityBuilder {
    fn new_entity(&mut self) -> &mut Self;
    fn add_component(&mut self, component: impl Component) -> &mut Self;
    fn build(&mut self) -> Entity;
}

pub(crate) trait Pos {
    fn get_pos(&self) -> cgmath::Vector3<f32>;
}

pub(crate) trait Collider {
    fn intersects(&self, other: &Self) -> bool;
    fn move_to(&mut self, pos: impl Pos);
}
