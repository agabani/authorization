use std::sync::{mpsc, Mutex};

use bevy::prelude::*;

use crate::{
    identity::Principal,
    network::{ConnectionTx, Protocol, Response},
    player::Player,
};

pub struct ArtificialIntelligencePlugin;

impl Plugin for ArtificialIntelligencePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, try_spawn_player);
    }
}

fn try_spawn_player(
    mut commands: Commands,
    principal: Res<Principal>,
    connection: Query<(Entity, &ConnectionTx)>,
    query: Query<(), With<Player>>,
    responses: Query<(), With<Response>>,
) {
    let count = query.iter().count() + responses.iter().count();

    (count..2).for_each(|_| {
        if let Ok((entity, connection)) = connection.get_single() {
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

            let (tx, rx) = mpsc::channel();

            if let Err(_) = connection.0.send(Protocol::Request(context, tx)) {
                commands.entity(entity).despawn();
                warn!("disconnected");
            };

            commands.spawn(Response::new(Mutex::new(rx)));
        };
    });
}
