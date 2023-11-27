use bevy::{prelude::*, utils::Uuid};

use crate::network::{ConnectionTx, Request};

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
            if let Err(_) =
                tx.0.send(crate::network::Protocol::Broadcast(context.clone()))
            {
                commands.entity(entity).despawn();
                warn!("disconnected");
            }
        });

        if let Err(_) = request.tx.send(Ok(context)) {
            warn!("disconnected");
        };
    });
}
