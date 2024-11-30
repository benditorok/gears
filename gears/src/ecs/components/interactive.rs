use super::{misc::Health, physics::RigidBody};
use crate::{
    ecs::traits::Component,
    prelude::{Pos3, ViewController},
};
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
        target_body: &RigidBody,
        target_health: &mut Health,
    ) {
    }
}
