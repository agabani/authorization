use bevy::prelude::*;

use crate::{
    identity::{Identifier, Identifiers, Principal},
    network::{send, Broadcast, ConnectionTx, Protocol, Replication, Response, ResponseError},
};

pub struct MonsterPlugin;

impl Plugin for MonsterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (broadcast, replication, response));
    }
}

#[derive(Default, Component)]
pub struct Monster;

fn broadcast(
    mut commands: Commands,
    principal: Res<Principal>,
    mut identifiers: ResMut<Identifiers>,
    query: Query<(Entity, &Broadcast)>,
) {
    query.for_each(|(entity, broadcast)| {
        commands.entity(entity).despawn();

        let identifier = Identifier::from(broadcast.context.resource.clone());

        if !identifiers.0.contains_key(&identifier) {
            let id = commands.spawn((Monster, identifier.clone())).id();
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
    connections: Query<(Entity, &ConnectionTx), With<Replication<Monster>>>,
    query: Query<&Identifier, With<Monster>>,
) {
    connections.for_each(|(entity, tx)| {
        commands.entity(entity).remove::<Replication<Monster>>();

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

            let protocol = Protocol::Broadcast(context);
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
    mut query: Query<(Entity, &mut Response<Monster>)>,
) {
    query.for_each_mut(|(entity, mut response)| {
        if let Some(result) = response.poll_once() {
            match result {
                Ok(context) => {
                    commands.entity(entity).despawn();

                    let identifier = Identifier::from(context.resource.clone());

                    if !identifiers.0.contains_key(&identifier) {
                        let id = commands.spawn((Monster, identifier.clone())).id();
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
