use bevy::prelude::*;

use crate::{
    identity::Principal,
    monster::{Monster, MonsterService},
    network::{ConnectionTx, Response},
    player::{Player, PlayerService},
};

pub struct ArtificialIntelligencePlugin;

impl Plugin for ArtificialIntelligencePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (try_spawn_monster, try_spawn_player));
    }
}

fn try_spawn_monster(
    mut commands: Commands,
    principal: Res<Principal>,
    connection: Query<&ConnectionTx>,
    query: Query<(), With<Monster>>,
    responses: Query<(), With<Response<Monster>>>,
) {
    let count = query.iter().count() + responses.iter().count();

    (count..5).for_each(|_| {
        if let Ok(connection) = connection.get_single() {
            if let Ok(task) = MonsterService::task(&principal, connection) {
                commands.spawn(task);
            };
        };
    });
}

fn try_spawn_player(
    mut commands: Commands,
    principal: Res<Principal>,
    connection: Query<&ConnectionTx>,
    query: Query<(), With<Player>>,
    responses: Query<(), With<Response<Player>>>,
) {
    let count = query.iter().count() + responses.iter().count();

    (count..2).for_each(|_| {
        if let Ok(connection) = connection.get_single() {
            if let Ok(task) = PlayerService::task(&principal, connection) {
                commands.spawn(task);
            };
        };
    });
}
