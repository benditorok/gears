pub mod controllers;
pub mod interactive;
pub mod lights;
pub mod misc;
pub mod models;
pub mod physics;
pub mod prefabs;
pub mod transforms;

use super::Component;
use std::time;

pub(crate) trait Pos {
    fn get_pos(&self) -> cgmath::Vector3<f32>;
}

pub(crate) trait Collider {
    fn intersects(&self, other: &Self) -> bool;
    fn move_to(&mut self, pos: impl Pos);
}

pub trait Tick {
    fn on_tick(&mut self, delta_time: time::Duration);
}

pub trait Prefab {
    fn unpack_prefab(&mut self) -> Vec<Box<impl Component>>;
}

pub trait Marker {
    fn describe() -> &'static str;
}
