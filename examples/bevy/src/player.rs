use bevy::prelude::*;

use crate::{
    identity::{Identifier, Identifiers, Principal},
    network::{
        send, Broadcast, ConnectionTx, Protocol, ProtocolEvent, Replication, Response,
        ResponseError,
    },
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (broadcast, replication, response));
    }
}

#[derive(Default, Component)]
pub struct Player;

fn broadcast(
    mut commands: Commands,
    principal: Res<Principal>,
    mut identifiers: ResMut<Identifiers>,
    query: Query<(Entity, &Broadcast<Player>)>,
) {
    query.for_each(|(entity, broadcast)| {
        commands.entity(entity).despawn();

        let identifier = Identifier::from(broadcast.context.resource.clone());

        if !identifiers.0.contains_key(&identifier) {
            let id = commands.spawn((Player, identifier.clone())).id();
            identifiers.0.insert(identifier, id);

            info!(
                "[broadcast] spawned player {:?} {:?}",
                principal.0, broadcast.context.resource
            );
        }
    });
}

fn replication(
    mut commands: Commands,
    principal: Res<Principal>,
    connections: Query<(Entity, &ConnectionTx), With<Replication<Player>>>,
    query: Query<&Identifier, With<Player>>,
) {
    connections.for_each(|(entity, tx)| {
        commands.entity(entity).remove::<Replication<Player>>();

        for identifier in &query {
            let context = authorization::Context {
                action: authorization::Action {
                    noun: identifier.noun.clone(),
                    scope: identifier.scope.clone(),
                    verb: "spawn".to_string(),
                },
                data: Default::default(),
                principal: principal.0.clone(),
                resource: identifier.clone().into(),
            };

            let protocol = Protocol::Broadcast(ProtocolEvent::Player(context));
            if !send(&mut commands, entity, tx, protocol) {
                return;
            }
        }
    });
}

fn response(
    mut commands: Commands,
    principal: Res<Principal>,
    mut identifiers: ResMut<Identifiers>,
    mut query: Query<(Entity, &mut Response<Player>)>,
) {
    query.for_each_mut(|(entity, mut response)| {
        if let Some(result) = response.poll_once() {
            match result {
                Ok(context) => {
                    commands.entity(entity).despawn();

                    let identifier = Identifier::from(context.resource.clone());

                    if !identifiers.0.contains_key(&identifier) {
                        let id = commands.spawn((Player, identifier.clone())).id();
                        identifiers.0.insert(identifier, id);

                        info!(
                            "[response] spawned player {:?} {:?}",
                            principal.0, context.resource
                        );
                    }
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
