use bevy::prelude::*;

pub struct ArtificialIntelligencePlugin;

impl Plugin for ArtificialIntelligencePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, try_spawn_player);
    }
}

#[derive(Component)]
pub struct ArtificialIntelligence;

fn try_spawn_player() {}
