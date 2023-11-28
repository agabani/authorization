use std::{
    marker::PhantomData,
    sync::{mpsc, Arc, Mutex},
};

use bevy::prelude::*;

/*
 * ============================================================================
 * Connections
 * ============================================================================
 */

#[derive(Resource)]
pub struct ConnectionsRx(pub Arc<Mutex<mpsc::Receiver<Handshake>>>);

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
    Replicate,
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
 * Replication
 * ============================================================================
 */
#[derive(Default, Component)]
pub struct Replication<T>
where
    T: Default,
{
    marker: PhantomData<T>,
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
pub struct Response<T> {
    result: Option<Result<authorization::Context, ResponseError>>,
    rx: Mutex<mpsc::Receiver<Result<authorization::Context, ResponseError>>>,
    marker: PhantomData<T>,
}

impl<T> Response<T> {
    pub fn new(rx: Mutex<mpsc::Receiver<Result<authorization::Context, ResponseError>>>) -> Self {
        Self {
            result: None,
            rx,
            marker: PhantomData,
        }
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

/*
 * ============================================================================
 * Send
 * ============================================================================
 */

pub fn send(
    commands: &mut Commands,
    entity: Entity,
    tx: &ConnectionTx,
    protocol: Protocol,
) -> bool {
    let result = tx.0.send(protocol);
    if result.is_err() {
        commands.entity(entity).despawn();
        warn!("disconnected");
    }
    result.is_ok()
}
