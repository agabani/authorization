use authorization_bevy::{AuthorizationSet, Identifier, Unauthorized};
use bevy::{prelude::*, utils::Uuid};

use crate::{
    kill::Killed,
    loot::SpawnLoot,
    monster::{Monster, SpawnMonster},
    player::{Player, SpawnPlayer},
};

/// Game Master Plugin.
pub struct GameMasterPlugin;

impl Plugin for GameMasterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn).add_systems(
            Update,
            (
                try_spawn_loot_when_entity_is_killed,
                try_spawn_monster_when_population_is_below_five,
                try_spawn_player_when_population_is_below_two,
            )
                .before(AuthorizationSet),
        );
    }
}

/// Game Master.
#[derive(Debug, Clone, Component)]
pub struct GameMaster;

/// Spawns a [`GameMaster`].
fn spawn(mut commands: Commands) {
    commands.spawn((
        GameMaster,
        Identifier {
            id: Uuid::new_v4().to_string(),
            noun: "game_master".to_string(),
            scope: "world".to_string(),
        },
    ));
}

/// Try to spawn loot when an [`Entity`] is [`Killed`].
fn try_spawn_loot_when_entity_is_killed(
    mut writer: EventWriter<Unauthorized<SpawnLoot>>,
    game_masters: Query<Entity, With<GameMaster>>,
    query: Query<&Killed, Added<Killed>>,
) {
    for game_master in &game_masters {
        for killed in &query {
            writer.send(Unauthorized {
                actor: game_master,
                data: SpawnLoot { owner: killed.by },
            });
        }
    }
}

/// Try to spawn a [`Monster`] when the [`Monster`] population drops below 5.
fn try_spawn_monster_when_population_is_below_five(
    mut writer: EventWriter<Unauthorized<SpawnMonster>>,
    game_masters: Query<Entity, With<GameMaster>>,
    monsters: Query<(), With<Monster>>,
) {
    for game_master in &game_masters {
        for _ in monsters.iter().count()..5 {
            writer.send(Unauthorized {
                actor: game_master,
                data: SpawnMonster,
            });
        }
    }
}

/// Try to spawn a [`Player`] when the [`Player`] population drops below 2.
fn try_spawn_player_when_population_is_below_two(
    mut writer: EventWriter<Unauthorized<SpawnPlayer>>,
    game_masters: Query<Entity, With<GameMaster>>,
    players: Query<(), With<Player>>,
) {
    for game_master in &game_masters {
        for _ in players.iter().count()..2 {
            writer.send(Unauthorized {
                actor: game_master,
                data: SpawnPlayer,
            });
        }
    }
}
