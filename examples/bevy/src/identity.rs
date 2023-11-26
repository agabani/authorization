pub use bevy::prelude::*;

#[derive(Component, Resource)]
pub struct Principal(pub authorization::Principal);
