use std::sync::{
    mpsc::{self, TryRecvError},
    Mutex,
};

use bevy::prelude::*;

use crate::{
    identity::Principal,
    network::{ConnectionRx, ConnectionTx, ConnectionsRx, Protocol},
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
    mut query: Query<(Entity, &ConnectionRx, &ConnectionTx, &mut ConnectionTimeout)>,
) {
    query.for_each_mut(|(entity, rx, tx, mut timeout)| {
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
                                warn!("disconnected");
                                commands.entity(entity).despawn();
                            };
                        }
                        Protocol::Pong => panic!("unexpected packet"),
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
