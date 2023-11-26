use std::sync::{mpsc, Mutex};

use bevy::ecs::{component::Component, system::Resource};

/*
 * ============================================================================
 * Connections
 * ============================================================================
 */

#[derive(Resource)]
pub struct ConnectionsRx(pub Mutex<mpsc::Receiver<Handshake>>);

#[derive(Resource)]
pub struct ConnectionsTx(pub mpsc::Sender<Handshake>);

pub struct Handshake {
    pub tx: mpsc::Sender<Protocol>,
}

/*
 * ============================================================================
 * Connection
 * ============================================================================
 */

#[derive(Component)]
pub struct ConnectionRx(pub Mutex<mpsc::Receiver<Protocol>>);

#[derive(Component)]
pub struct ConnectionTx(pub mpsc::Sender<Protocol>);

pub enum Protocol {
    Connected(mpsc::Sender<Protocol>),
    Disconnect,
    Ping,
    Pong,
}
