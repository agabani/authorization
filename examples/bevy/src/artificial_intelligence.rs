use std::sync::{mpsc, Mutex};

use bevy::prelude::*;

use crate::{
    identity::Principal,
    monster::Monster,
    network::{send, ConnectionTx, Frame, Response},
    player::Player,
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
    connection: Query<(Entity, &ConnectionTx)>,
    query: Query<(), With<Monster>>,
    responses: Query<(), With<Response<Monster>>>,
) {
    let count = query.iter().count() + responses.iter().count();

    (count..5).for_each(|_| {
        if let Ok((entity, tx)) = connection.get_single() {
            let context = authorization::Context {
                action: authorization::Action {
                    noun: "monster".to_string(),
                    scope: "world".to_string(),
                    verb: "spawn".to_string(),
                },
                data: Default::default(),
                principal: principal.0.clone(),
                resource: authorization::Resource {
                    id: "".to_string(),
                    noun: "monster".to_string(),
                    scope: "world".to_string(),
                },
            };

            let (response_tx, rx) = mpsc::channel();
            let frame = Frame::Request(context, response_tx);

            if send(&mut commands, entity, &tx, frame) {
                commands.spawn(Response::<Monster>::new(Mutex::new(rx)));
            }
        };
    });
}

fn try_spawn_player(
    mut commands: Commands,
    principal: Res<Principal>,
    connection: Query<(Entity, &ConnectionTx)>,
    query: Query<(), With<Player>>,
    responses: Query<(), With<Response<Player>>>,
) {
    let count = query.iter().count() + responses.iter().count();

    (count..2).for_each(|_| {
        if let Ok((entity, tx)) = connection.get_single() {
            let context = authorization::Context {
                action: authorization::Action {
                    noun: "player".to_string(),
                    scope: "world".to_string(),
                    verb: "spawn".to_string(),
                },
                data: Default::default(),
                principal: principal.0.clone(),
                resource: authorization::Resource {
                    id: "".to_string(),
                    noun: "player".to_string(),
                    scope: "world".to_string(),
                },
            };

            let (response_tx, rx) = mpsc::channel();
            let frame = Frame::Request(context, response_tx);

            if send(&mut commands, entity, &tx, frame) {
                commands.spawn(Response::<Player>::new(Mutex::new(rx)));
            }
        };
    });
}
