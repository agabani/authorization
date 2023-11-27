use bevy::{prelude::*, utils::Uuid};

use crate::network::Request;

pub struct AuthorityPlugin;

impl Plugin for AuthorityPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_request);
    }
}

fn handle_request(mut commands: Commands, requests: Query<(Entity, &Request)>) {
    requests.for_each(|(entity, request)| {
        let mut context = request.context.clone();
        context.resource.id = Uuid::new_v4().to_string();

        if let Err(_) = request.tx.send(Ok(context)) {
            warn!("disconnected");
        };

        commands.entity(entity).despawn();
    });
}
