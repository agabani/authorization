use bevy::prelude::*;

use crate::{
    identity::{Identifier, Identifiers, Principal},
    network::{
        send, Broadcast, ConnectionTx, Frame, FrameEvent, Replicate, Response, ResponseError,
    },
};

pub struct MonsterPlugin;

impl Plugin for MonsterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_from_broadcast,
                replicate_to_connection,
                spawn_from_response,
            ),
        );
    }
}

#[derive(Default, Component)]
pub struct Monster;

fn replicate_to_connection(
    mut commands: Commands,
    principal: Res<Principal>,
    connections: Query<(Entity, &ConnectionTx), With<Replicate<Monster>>>,
    query: Query<&Identifier, With<Monster>>,
) {
    connections.for_each(|(entity, tx)| {
        commands.entity(entity).remove::<Replicate<Monster>>();

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

            let frame = Frame::Broadcast(FrameEvent::Monster(context));
            if !send(&mut commands, entity, tx, frame) {
                return;
            }
        }
    });
}

fn spawn_from_broadcast(
    mut commands: Commands,
    principal: Res<Principal>,
    mut identifiers: ResMut<Identifiers>,
    query: Query<(Entity, &Broadcast<Monster>)>,
) {
    query.for_each(|(entity, broadcast)| {
        commands.entity(entity).despawn();
        spawn(
            &mut commands,
            &principal,
            &mut identifiers,
            &broadcast.context,
        );
    });
}

fn spawn_from_response(
    mut commands: Commands,
    principal: Res<Principal>,
    mut identifiers: ResMut<Identifiers>,
    mut query: Query<(Entity, &mut Response<Monster>)>,
) {
    query.for_each_mut(|(entity, mut response)| {
        if let Some(result) = response.poll_once() {
            commands.entity(entity).despawn();
            match result {
                Ok(context) => {
                    spawn(&mut commands, &principal, &mut identifiers, context);
                }
                Err(error) => match error {
                    ResponseError::Denied => warn!("denied"),
                    ResponseError::Disconnected => warn!("disconnected"),
                    ResponseError::NoAuthorityAvailable => warn!("no authority available"),
                },
            }
        };
    });
}

fn spawn(
    commands: &mut Commands,
    principal: &Principal,
    identifiers: &mut Identifiers,
    context: &authorization::Context,
) {
    let identifier = Identifier::from(context.resource.clone());

    if !identifiers.0.contains_key(&identifier) {
        let id = commands.spawn((Monster, identifier.clone())).id();
        identifiers.0.insert(identifier, id);

        info!("spawned {:?} {:?}", principal.0, context.resource);
    }
}
