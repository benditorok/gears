use super::Entity;

pub trait EntityBuilder {
    fn new_entity(&mut self) -> &mut Self;
    fn add_component<T: 'static + Send + Sync>(&mut self, component: T) -> &mut Self;
    fn add_components<T: 'static + Send + Sync, I>(&mut self, components: I) -> &mut Self
    where
        I: IntoIterator<Item = T>,
    {
        for component in components {
            self.add_component(component);
        }

        self
    }
    fn build(&mut self) -> Entity;
}
