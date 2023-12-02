use std::sync::{mpsc, Mutex};

use bevy::prelude::*;

use crate::{
    async_task::AsyncError,
    identity::{Identifier, Identifiers, Principal},
    network::{
        Broadcast, ConnectionTx, Frame, FrameEvent, Replicate, RequestError, Response,
        ResponseError,
    },
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
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
pub struct Player;

fn replicate_to_connection(
    mut commands: Commands,
    principal: Res<Principal>,
    connections: Query<(Entity, &ConnectionTx), With<Replicate<Player>>>,
    query: Query<&Identifier, With<Player>>,
) {
    connections.for_each(|(entity, tx)| {
        commands.entity(entity).remove::<Replicate<Player>>();

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
            let frame = Frame::Broadcast(FrameEvent::Player(context));
            tx.send(frame);
        }
    });
}

fn spawn_from_broadcast(
    mut commands: Commands,
    principal: Res<Principal>,
    mut identifiers: ResMut<Identifiers>,
    query: Query<(Entity, &Broadcast<Player>)>,
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
    mut query: Query<(Entity, &mut Response<Player>)>,
) {
    query.for_each_mut(|(entity, mut response)| match response.poll_once() {
        Ok(Some(Ok(context))) => {
            commands.entity(entity).despawn();
            spawn(&mut commands, &principal, &mut identifiers, context);
        }
        Ok(Some(Err(ResponseError::Denied))) => {
            commands.entity(entity).despawn();
            warn!("denied");
        }
        Ok(Some(Err(ResponseError::NoAuthorityAvailable))) => {
            commands.entity(entity).despawn();
            warn!("no authority available");
        }
        Ok(None) => {}
        Err(AsyncError::Disconnected) => {
            commands.entity(entity).despawn();
            warn!("disconnected");
        }
        Err(AsyncError::Poisoned) => {
            commands.entity(entity).despawn();
            warn!("poisoned");
        }
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
        let id = commands.spawn((Player, identifier.clone())).id();
        identifiers.0.insert(identifier, id);

        info!("spawned {:?} {:?}", principal.0, context.resource);
    }
}

pub struct PlayerService;

impl PlayerService {
    pub fn task(
        principal: &Res<Principal>,
        connection: &ConnectionTx,
    ) -> Result<Response<Player>, RequestError> {
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
        let frame = Frame::Request(context, tx);

        connection
            .send(frame)
            .map(|_| Response::<Player>::new(Mutex::new(rx)))
            .map_err(|_| RequestError::Disconnected)
    }
}
