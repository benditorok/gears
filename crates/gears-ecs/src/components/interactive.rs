use super::{
    misc::Health,
    physics::{CollisionBox, RigidBody},
};
use crate::{components::controllers::ViewController, components::transforms::Pos3, Component};
use gears_macro::Component;

#[derive(Component, Debug, Clone, Copy)]
pub struct Weapon {
    pub damage: f32,
}

impl Weapon {
    pub fn new(damage: f32) -> Self {
        Self { damage }
    }

    pub fn shoot(
        &self,
        self_pos3: &Pos3,
        self_view: &ViewController,
        target_pos3: &Pos3,
        target_body: &RigidBody<impl CollisionBox>,
        target_health: &mut Health,
    ) {
    }
}
