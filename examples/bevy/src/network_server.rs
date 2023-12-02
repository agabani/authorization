use std::sync::{
    mpsc::{self, TryRecvError},
    Mutex,
};

use bevy::prelude::*;

use crate::{
    identity::Principal,
    monster::Monster,
    network::{ConnectionRx, ConnectionTx, ConnectionsRx, Frame, Replicate, ResponseError},
    player::Player,
};

pub struct NetworkServerPlugin;

impl Plugin for NetworkServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                accept_connection,
                disconnect_timed_out_connections,
                read_connection,
            ),
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
 * Read connection
 * ============================================================================
 */

fn read_connection(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &Principal,
        &ConnectionRx,
        &ConnectionTx,
        &mut ConnectionTimeout,
    )>,
    authority: Query<(Entity, &ConnectionTx, &Principal)>,
    broadcast: Query<&ConnectionTx>,
) {
    query.for_each_mut(|(entity, principal, rx, tx, mut timeout)| loop {
        match rx.try_recv() {
            Ok(frame) => {
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
                        if context.principal != principal.0 {
                            warn!("impersonation");
                        }

                        if let Some((entity, tx, _)) = authority
                            .iter()
                            .find(|(_, _, principal)| principal.0.noun == "authority")
                        {
                            let frame = Frame::Request(context, response.clone());
                            if let Err(_) = tx.send(frame) {
                                error!("failed to send to authority");

                                if response
                                    .send(Err(ResponseError::NoAuthorityAvailable))
                                    .is_err()
                                {
                                    warn!("failed to send error");
                                }

                                commands.entity(entity).despawn();
                                warn!("disconnected");
                            }
                        } else {
                            error!("no authority available");

                            if response
                                .send(Err(ResponseError::NoAuthorityAvailable))
                                .is_err()
                            {
                                warn!("failed to send error");
                            }
                        }
                    }
                    Frame::Broadcast(event) => {
                        if principal.0.noun != "authority" {
                            warn!("permission");
                            return;
                        }

                        broadcast.for_each(|tx| {
                            let frame = Frame::Broadcast(event.clone());
                            let _ = tx.send(frame);
                        });

                        event.spawn_broadcast(&mut commands);
                    }
                    Frame::Replicate => {
                        commands.entity(entity).insert((
                            Replicate::<Monster>::default(),
                            Replicate::<Player>::default(),
                        ));
                    }
                }
            }
            Err(error) => {
                match error {
                    TryRecvError::Empty => {}
                    TryRecvError::Disconnected => {
                        commands.entity(entity).despawn();
                        warn!("disconnected");
                    }
                }
                return;
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
