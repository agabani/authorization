use bevy::prelude::*;

use crate::{
    authorization_bevy::{AuthorizationService, AuthorizationTask, Identifier, Unauthorized},
    monster::{Monster, SpawnMonster},
    player::{Player, SpawnPlayer},
};

/// Game Master Plugin.
pub struct GameMasterPlugin;

impl Plugin for GameMasterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (try_spawn_players, try_spawn_monster));
    }
}

/// Game Master.
#[derive(Debug, Clone, Component)]
pub struct GameMaster;

fn try_spawn_monster(
    mut commands: Commands,
    authorization_service: Res<AuthorizationService>,
    query: Query<Entity, With<GameMaster>>,
    identifiers: Query<&Identifier>,
    monsters: Query<(), With<Monster>>,
    spawn_monsters: Query<(), With<AuthorizationTask<SpawnMonster>>>,
) {
    query.for_each(|entity| {
        let current = monsters.iter().len() + spawn_monsters.iter().len();
        for _ in current..5 {
            let task = authorization_service.authorize(
                Unauthorized {
                    actor: entity,
                    data: SpawnMonster,
                },
                &identifiers,
            );

            if let Some(task) = task {
                commands.spawn(task);
            }
        }
    });
}

fn try_spawn_players(
    mut commands: Commands,
    authorization_service: Res<AuthorizationService>,
    query: Query<Entity, With<GameMaster>>,
    identifiers: Query<&Identifier>,
    players: Query<(), With<Player>>,
    spawn_players: Query<(), With<AuthorizationTask<SpawnPlayer>>>,
) {
    query.for_each(|entity| {
        let current = players.iter().len() + spawn_players.iter().len();
        for _ in current..2 {
            let task = authorization_service.authorize(
                Unauthorized {
                    actor: entity,
                    data: SpawnPlayer,
                },
                &identifiers,
            );

            if let Some(task) = task {
                commands.spawn(task);
            }
        }
    });
}
