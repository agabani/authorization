use std::sync::{
    mpsc::{self, TryRecvError},
    Mutex,
};

use bevy::prelude::*;

use crate::network::{ConnectionRx, ConnectionTx, ConnectionsTx, Handshake, Protocol};

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                initiate_connection,
                initialize_connection,
                keep_connection_alive,
                read_connection,
            ),
        );
    }
}

/*
 * ============================================================================
 * Create connections
 * ============================================================================
 */

fn initiate_connection(
    mut commands: Commands,
    connections: Res<ConnectionsTx>,
    query: Query<&ConnectionRx>,
) {
    if query.is_empty() {
        let (tx, rx) = mpsc::channel();
        connections
            .0
            .send(Handshake { tx })
            .expect("connections closed");
        commands.spawn(ConnectionRx(Mutex::new(rx)));
    }
}

fn initialize_connection(
    mut commands: Commands,
    query: Query<(Entity, &ConnectionRx), Without<ConnectionTx>>,
) {
    query.for_each(
        |(entity, rx)| match rx.0.lock().expect("poisoned").try_recv() {
            Ok(Protocol::Connected(tx)) => {
                commands
                    .entity(entity)
                    .insert(ConnectionTx(tx))
                    .insert(KeepAlive(Timer::from_seconds(1.0, TimerMode::Repeating)));
                info!("connected");
            }
            Ok(_) => {
                panic!("unexpected packet");
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                commands.entity(entity).despawn();
                warn!("disconnected");
            }
            Err(mpsc::TryRecvError::Empty) => {}
        },
    );
}

/*
 * ============================================================================
 * Read connection
 * ============================================================================
 */

fn read_connection(
    mut commands: Commands,
    query: Query<(Entity, &ConnectionRx), With<ConnectionTx>>,
) {
    query.for_each(|(entity, rx)| {
        let rx = rx.0.lock().expect("poisoned");
        loop {
            match rx.try_recv() {
                Ok(protocol) => match protocol {
                    Protocol::Connected(_) => panic!("unexpected packet"),
                    Protocol::Disconnect => {
                        commands.entity(entity).despawn();
                        warn!("disconnected");
                    }
                    Protocol::Ping => panic!("unexpected packet"),
                    Protocol::Pong => {}
                },
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
 * Keep alive connection
 * ============================================================================
 */

#[derive(Component)]
struct KeepAlive(Timer);

fn keep_connection_alive(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &ConnectionTx, &mut KeepAlive)>,
) {
    query.for_each_mut(|(entity, tx, mut keep_alive)| {
        if keep_alive.0.tick(time.delta()).finished() {
            if let Ok(_) = tx.0.send(Protocol::Ping) {
                keep_alive.0.reset();
            } else {
                commands.entity(entity).despawn();
                warn!("disconnected");
            };
        }
    });
}
