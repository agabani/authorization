use std::sync::{mpsc, Mutex};

use bevy::prelude::*;

use crate::network::{ConnectionRx, ConnectionTx, ConnectionsRx, Protocol};

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (accept_connection, disconnect_timed_out_connections),
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
            commands.spawn((
                ConnectionRx(Mutex::new(rx)),
                ConnectionTx(handshake.tx),
                ConnectionTimeout(Timer::from_seconds(10.0, TimerMode::Once)),
            ));
            info!("connected");
        } else {
            warn!("disconnected");
        };
    }
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
