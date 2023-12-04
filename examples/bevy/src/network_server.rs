use std::sync::{mpsc, Mutex};

use bevy::prelude::*;

use crate::{
    identity::Principal,
    monster::Monster,
    network::{
        Broadcast, ConnectionRx, ConnectionTx, ConnectionsRx, Frame, Replicate, ResponseError,
    },
    player::Player,
};

pub struct NetworkServerPlugin;

impl Plugin for NetworkServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (accept_connection, disconnect_timed_out_connections, router),
        );
    }
}

/*
 * ============================================================================
 * Accept connections
 * ============================================================================
 */

fn accept_connection(mut commands: Commands, connections: Res<ConnectionsRx>) {
    for handshake in connections.0.lock().expect("poisoned").try_iter() {
        let (tx, rx) = mpsc::channel();
        if let Ok(_) = handshake.tx.send(Frame::Connected(tx)) {
            info!("connected {:?}", handshake.principal);
            commands.spawn((
                ConnectionRx::new(Mutex::new(rx)),
                ConnectionTx::new(handshake.tx),
                ConnectionTimeout(Timer::from_seconds(2.0, TimerMode::Once)),
                Principal(handshake.principal),
            ));
        } else {
            warn!("disconnected");
        };
    }
}

/*
 * ============================================================================
 * Router
 * ============================================================================
 */

fn router(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &Principal,
        &ConnectionRx,
        &ConnectionTx,
        &mut ConnectionTimeout,
    )>,
    connections: Query<(&ConnectionTx, &Principal)>,
) {
    query.for_each_mut(|(entity, principal, rx, tx, mut timeout)| loop {
        let Ok(frame) = rx.try_recv() else {
            return;
        };

        timeout.0.reset();

        match frame {
            Frame::Connected(_) => panic!("unexpected packet"),
            Frame::Disconnect => todo!("disconnect"),
            Frame::Ping => {
                let frame = Frame::Pong;
                let _ = tx.send(frame);
            }
            Frame::Pong => panic!("unexpected packet"),
            Frame::Request(context, response) => {
                if context.principal == principal.0 {
                    let Some((tx, _)) = connections
                        .iter()
                        .find(|(_, principal)| principal.0.noun == "authority")
                    else {
                        let _ = response.send(Err(ResponseError::NoAuthorityAvailable));
                        return;
                    };

                    let frame = Frame::Request(context, response.clone());
                    if tx.send(frame).is_err() {
                        let _ = response.send(Err(ResponseError::NoAuthorityAvailable));
                    };
                }
            }
            Frame::Broadcast(context) => {
                if principal.0.noun == "authority" {
                    connections.for_each(|(tx, _)| {
                        let frame = Frame::Broadcast(context.clone());
                        let _ = tx.send(frame);
                    });

                    Broadcast::spawn(context, &mut commands);
                }
            }
            Frame::Replicate => {
                commands.entity(entity).insert((
                    Replicate::<Monster>::default(),
                    Replicate::<Player>::default(),
                ));
            }
        }
    });
}

/*
 * ============================================================================
 * Connection Timeout
 * ============================================================================
 */

#[derive(Component)]
struct ConnectionTimeout(Timer);

fn disconnect_timed_out_connections(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &ConnectionTx, &mut ConnectionTimeout)>,
) {
    query.for_each_mut(|(entity, tx, mut timeout)| {
        if timeout.0.tick(time.delta()).finished() {
            if let Err(_) = tx.send(Frame::Disconnect) {
                warn!("disconnected");
            }
            commands.entity(entity).despawn();
            warn!("timed out");
        }
    })
}
