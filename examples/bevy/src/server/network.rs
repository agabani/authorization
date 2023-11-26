use std::sync::{mpsc, Mutex};

use bevy::prelude::*;

use crate::network::{ConnectionRx, ConnectionTx, ConnectionsRx, Protocol};

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, accept_connection);
    }
}

fn accept_connection(mut commands: Commands, connections: Res<ConnectionsRx>) {
    for handshake in connections.0.lock().expect("poisoned").try_iter() {
        let (tx, rx) = mpsc::channel();
        match handshake.tx.send(Protocol::Connected(tx)) {
            Ok(_) => {
                commands.spawn((ConnectionRx(Mutex::new(rx)), ConnectionTx(handshake.tx)));
                info!("connected");
            }
            Err(_) => {
                warn!("disconnected");
            }
        };
    }
}
