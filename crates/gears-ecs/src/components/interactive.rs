use super::{
    misc::Health,
    physics::{CollisionBox, RigidBody},
};
use crate::{Component, components::controllers::ViewController, components::transforms::Pos3};
use gears_macro::Component;

/// A component representing a weapon that can be used to attack other entities.
#[derive(Component, Debug, Clone, Copy)]
pub struct Weapon {
    /// The damage dealt by the weapon.
    pub damage: f32,
}

impl Weapon {
    /// Creates a new weapon with the specified damage.
    ///
    /// # Arguments
    ///
    /// * `damage` - The damage dealt by the weapon.
    ///
    /// # Returns
    ///
    /// A new [`Weapon`] instance.
    pub fn new(damage: f32) -> Self {
        Self { damage }
    }

    /// Shoots the weapon at the specified target.
    ///
    /// # Arguments
    ///
    /// * `self_pos3` - The position of the shooter.
    /// * `self_view` - The view controller of the shooter.
    /// * `target_pos3` - The position of the target.
    /// * `target_body` - The rigid body of the target.
    /// * `target_health` - The health component of the target.
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
