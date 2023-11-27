use std::sync::{mpsc, Mutex};

use bevy::prelude::*;

use crate::{
    identity::{Identifier, Principal},
    network::{ConnectionTx, Protocol, Response, ResponseError},
    player::Player,
};

pub struct ArtificialIntelligencePlugin;

impl Plugin for ArtificialIntelligencePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (initiate_spawn_player, initialize_spawn_player));
    }
}

fn initiate_spawn_player(
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

fn initialize_spawn_player(mut commands: Commands, mut responses: Query<(Entity, &mut Response)>) {
    responses.for_each_mut(|(entity, mut request)| {
        if let Some(result) = request.poll_once() {
            match result {
                Ok(context) => {
                    commands
                        .entity(entity)
                        .remove::<Response>()
                        .insert(Player)
                        .insert(Identifier::from(context.resource.clone()));

                    info!("spawned player {:?}", context.resource);
                }
                Err(error) => {
                    match error {
                        ResponseError::Denied => warn!("denied"),
                        ResponseError::Disconnected => warn!("disconnected"),
                        ResponseError::NoAuthorityAvailable => warn!("no authority available"),
                    }
                    commands.entity(entity).despawn();
                }
            }
        };
    });
}
