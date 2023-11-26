use std::sync::{
    mpsc::{self, TryRecvError},
    Mutex,
};

use bevy::prelude::*;

use crate::network::{ConnectionRx, ConnectionTx, ConnectionsTx, Handshake, Protocol};

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (initiate_connection, initialize_connection))
            .add_systems(Update, read_connection);
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
                commands.entity(entity).insert(ConnectionTx(tx));
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
                Ok(_protocol) => {}
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
