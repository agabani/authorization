use bevy::{prelude::*, utils::Uuid};

use crate::network::{ConnectionTx, Frame, FrameEvent, Request};

pub struct AuthorityPlugin;

impl Plugin for AuthorityPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_request);
    }
}

fn handle_request(
    mut commands: Commands,
    requests: Query<(Entity, &Request)>,
    connections: Query<&ConnectionTx>,
) {
    requests.for_each(|(entity, request)| {
        commands.entity(entity).despawn();

        let mut context = request.context.clone();
        context.resource.id = Uuid::new_v4().to_string();

        connections.for_each(|connection| {
            let frame_event = match context.resource.noun.as_str() {
                "monster" => FrameEvent::Monster(context.clone()),
                "player" => FrameEvent::Player(context.clone()),
                noun => todo!("{noun}"),
            };
            let frame = Frame::Broadcast(frame_event);
            connection.send(frame);
        });

        request.tx.send(Ok(context));
    });
}
