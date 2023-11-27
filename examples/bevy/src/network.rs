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
    pub principal: authorization::Principal,
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
    Request(
        authorization::Context,
        mpsc::Sender<Result<authorization::Context, ResponseError>>,
    ),
    Broadcast(authorization::Context),
}

/*
 * ============================================================================
 * Broadcast
 * ============================================================================
 */

#[derive(Component)]
pub struct Broadcast {
    pub context: authorization::Context,
}

/*
 * ============================================================================
 * Request
 * ============================================================================
 */

#[derive(Component)]
pub struct Request {
    pub context: authorization::Context,
    pub tx: mpsc::Sender<Result<authorization::Context, ResponseError>>,
}

/*
 * ============================================================================
 * Response
 * ============================================================================
 */

#[derive(Component)]
pub struct Response {
    result: Option<Result<authorization::Context, ResponseError>>,
    rx: Mutex<mpsc::Receiver<Result<authorization::Context, ResponseError>>>,
}

impl Response {
    pub fn new(rx: Mutex<mpsc::Receiver<Result<authorization::Context, ResponseError>>>) -> Self {
        Self { result: None, rx }
    }

    pub fn poll_once(&mut self) -> &Option<Result<authorization::Context, ResponseError>> {
        if self.result.is_some() {
            return &self.result;
        }

        match self.rx.lock().expect("poisoned").try_recv() {
            Ok(result) => {
                self.result = Some(result);
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                self.result = Some(Err(ResponseError::Disconnected));
            }
            Err(mpsc::TryRecvError::Empty) => {}
        };

        &self.result
    }
}

pub enum ResponseError {
    Denied,
    Disconnected,
    NoAuthorityAvailable,
}
