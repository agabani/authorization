use std::sync::{
    mpsc::{self, TryRecvError},
    Mutex,
};

use bevy::prelude::*;

use crate::{
    identity::Principal,
    network::{
        Broadcast, ConnectionRx, ConnectionTx, ConnectionsRx, Protocol, Replication, ResponseError,
    },
};

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
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
        if let Ok(_) = handshake.tx.send(Protocol::Connected(tx)) {
            info!("connected {:?}", handshake.principal);
            commands.spawn((
                ConnectionRx(Mutex::new(rx)),
                ConnectionTx(handshake.tx),
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
    broadcast: Query<(Entity, &ConnectionTx)>,
) {
    query.for_each_mut(|(entity, principal, rx, tx, mut timeout)| {
        let rx = rx.0.lock().expect("poisoned");
        loop {
            match rx.try_recv() {
                Ok(protocol) => {
                    timeout.0.reset();

                    match protocol {
                        Protocol::Connected(_) => panic!("unexpected packet"),
                        Protocol::Disconnect => todo!("disconnect"),
                        Protocol::Ping => {
                            if let Err(_) = tx.0.send(Protocol::Pong) {
                                commands.entity(entity).despawn();
                                warn!("disconnected");
                            };
                        }
                        Protocol::Pong => panic!("unexpected packet"),
                        Protocol::Request(context, response) => {
                            if context.principal != principal.0 {
                                warn!("impersonation");
                            }

                            if let Some((entity, tx, _)) = authority
                                .iter()
                                .find(|(_, _, principal)| principal.0.noun == "authority")
                            {
                                if let Err(_) =
                                    tx.0.send(Protocol::Request(context, response.clone()))
                                {
                                    error!("no authority available");

                                    if let Err(_) =
                                        response.send(Err(ResponseError::NoAuthorityAvailable))
                                    {
                                        warn!("failed to send error");
                                    }

                                    commands.entity(entity).despawn();
                                    warn!("disconnected");
                                };
                            } else {
                                error!("no authority available");

                                if let Err(_) =
                                    response.send(Err(ResponseError::NoAuthorityAvailable))
                                {
                                    warn!("failed to send error");
                                }
                            }
                        }
                        Protocol::Broadcast(context) => {
                            if principal.0.noun != "authority" {
                                warn!("permission");
                            }

                            broadcast.for_each(|(entity, tx)| {
                                if let Err(_) = tx.0.send(Protocol::Broadcast(context.clone())) {
                                    commands.entity(entity).despawn();
                                    warn!("disconnected");
                                };
                            });

                            commands.spawn(Broadcast { context });
                        }
                        Protocol::Replicate => {
                            commands.entity(entity).insert(Replication);
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
            if let Err(_) = tx.0.send(Protocol::Disconnect) {
                warn!("disconnected");
            }
            commands.entity(entity).despawn();
            warn!("timed out");
        }
    })
}
