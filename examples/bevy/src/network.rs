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
        mpsc::Sender<Result<authorization::Context, RequestError>>,
    ),
}

/*
 * ============================================================================
 * Request
 * ============================================================================
 */

#[derive(Component)]
pub struct Request {
    result: Option<Result<authorization::Context, RequestError>>,
    rx: Mutex<mpsc::Receiver<Result<authorization::Context, RequestError>>>,
}

impl Request {
    pub fn new(rx: Mutex<mpsc::Receiver<Result<authorization::Context, RequestError>>>) -> Self {
        Self { result: None, rx }
    }

    pub fn poll_once(&mut self) -> &Option<Result<authorization::Context, RequestError>> {
        if self.result.is_some() {
            return &self.result;
        }

        match self.rx.lock().expect("poisoned").try_recv() {
            Ok(result) => {
                self.result = Some(result);
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                self.result = Some(Err(RequestError::Disconnected));
            }
            Err(mpsc::TryRecvError::Empty) => {}
        };

        &self.result
    }
}

pub enum RequestError {
    Denied,
    Disconnected,
    NoAuthorityAvailable,
}
