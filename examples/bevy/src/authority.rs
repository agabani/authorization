use bevy::{prelude::*, utils::Uuid};

use crate::network::{send, ConnectionTx, Frame, FrameEvent, Request};

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
            let frame_event = match context.resource.noun.as_str() {
                "monster" => FrameEvent::Monster(context.clone()),
                "player" => FrameEvent::Player(context.clone()),
                noun => todo!("{noun}"),
            };
            let frame = Frame::Broadcast(frame_event);
            send(&mut commands, entity, tx, frame);
        });

        if let Err(_) = request.tx.send(Ok(context)) {
            warn!("disconnected");
        };
    });
}
