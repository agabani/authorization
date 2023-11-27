use bevy::{prelude::*, utils::Uuid};

use crate::network::{send, ConnectionTx, Protocol, Request};

pub struct AuthorityPlugin;

impl Plugin for AuthorityPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_request);
    }
}

fn handle_request(
    mut commands: Commands,
    requests: Query<(Entity, &Request)>,
    tx: Query<(Entity, &ConnectionTx)>,
) {
    requests.for_each(|(entity, request)| {
        commands.entity(entity).despawn();

        let mut context = request.context.clone();
        context.resource.id = Uuid::new_v4().to_string();

        tx.for_each(|(entity, tx)| {
            let protocol = Protocol::Broadcast(context.clone());
            send(&mut commands, entity, tx, protocol);
        });

        if let Err(_) = request.tx.send(Ok(context)) {
            warn!("disconnected");
        };
    });
}
