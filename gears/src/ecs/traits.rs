use super::Entity;

pub trait EntityBuilder {
    fn new_entity(&mut self) -> &mut Self;
    fn add_component<T: 'static + Send + Sync>(&mut self, component: T) -> &mut Self;
    fn build(&mut self) -> Entity;
}
