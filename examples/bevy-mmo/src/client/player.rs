use bevy::prelude::*;

use crate::core::IntoContext;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {}
}

#[derive(Component)]
pub struct Player;

pub struct SpawnPlayer;

impl IntoContext for SpawnPlayer {
    fn into_context(
        unauthorized: &crate::core::Unauthorized<Self>,
        identifiers: &Query<&crate::core::Identifier>,
    ) -> Option<authorization::Context> {
        todo!()
    }
}
