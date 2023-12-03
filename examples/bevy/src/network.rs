use std::{
    marker::PhantomData,
    sync::{mpsc, Arc, Mutex},
};

use bevy::prelude::*;

use crate::{async_task::AsyncTask, identity::Principal, monster::Monster, player::Player};

/*
 * ============================================================================
 * Connections
 * ============================================================================
 */

/// Connections Receiver.
#[derive(Resource)]
pub struct ConnectionsRx(pub Arc<Mutex<mpsc::Receiver<Handshake>>>);

/// Connections Transmitter.
#[derive(Resource)]
pub struct ConnectionsTx(mpsc::Sender<Handshake>);

impl ConnectionsTx {
    /// Creates a new [`ConnectionsTx`].
    pub fn new(sender: mpsc::Sender<Handshake>) -> Self {
        Self(sender)
    }

    /// Creates a [`ConnectionRx`].
    pub fn connect(
        &self,
        principal: &Res<Principal>,
    ) -> Result<ConnectionRx, mpsc::SendError<Handshake>> {
        let (tx, rx) = mpsc::channel();
        self.0.send(Handshake {
            principal: principal.0.clone(),
            tx,
        })?;
        Ok(ConnectionRx::new(Mutex::new(rx)))
    }
}

pub struct Handshake {
    pub principal: authorization::Principal,
    pub tx: mpsc::Sender<Frame>,
}

/*
 * ============================================================================
 * Connection
 * ============================================================================
 */

/// Connection Receiver.
#[derive(Component)]
pub struct ConnectionRx(Mutex<mpsc::Receiver<Frame>>);

impl ConnectionRx {
    /// Creates a new [`ConnectionRx`].
    pub fn new(receiver: Mutex<mpsc::Receiver<Frame>>) -> Self {
        Self(receiver)
    }

    /// Try to receive a frame.
    pub fn try_recv(&self) -> Result<Frame, mpsc::TryRecvError> {
        self.0.lock().expect("poisoned").try_recv()
    }
}

/// Connection Transmitter.
#[derive(Component)]
pub struct ConnectionTx(mpsc::Sender<Frame>);

impl ConnectionTx {
    /// Creates a new [`ConnectionTx`].
    pub fn new(sender: mpsc::Sender<Frame>) -> Self {
        Self(sender)
    }

    /// Send a frame.
    pub fn send(&self, frame: Frame) -> Result<(), mpsc::SendError<Frame>> {
        self.0.send(frame)
    }
}

/*
 * ============================================================================
 * Frame
 * ============================================================================
 */

/// Frame.
pub enum Frame {
    /// Connected.
    Connected(mpsc::Sender<Frame>),

    /// Disconnected.
    Disconnect,

    /// Ping.
    Ping,

    /// Pong.
    Pong,

    /// Request.
    Request(
        authorization::Context,
        mpsc::Sender<Result<authorization::Context, ResponseError>>,
    ),

    /// Broadcast.
    Broadcast(authorization::Context),

    /// Replicate.
    Replicate,
}

/*
 * ============================================================================
 * Broadcast
 * ============================================================================
 */

#[derive(Component)]
pub struct Broadcast<T> {
    pub context: authorization::Context,
    marker: PhantomData<T>,
}

impl Broadcast<()> {
    pub fn spawn(context: authorization::Context, commands: &mut Commands) {
        if context.resource.noun == "monster" {
            commands.spawn(Broadcast {
                context,
                marker: PhantomData::<Monster>,
            });
            return;
        }

        if context.resource.noun == "player" {
            commands.spawn(Broadcast {
                context,
                marker: PhantomData::<Player>,
            });
            return;
        }
    }
}

/*
 * ============================================================================
 * Replication
 * ============================================================================
 */
#[derive(Default, Component)]
pub struct Replicate<T>
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

pub enum RequestError {
    Disconnected,
}

/*
 * ============================================================================
 * Response
 * ============================================================================
 */

pub type Response<M> = AsyncTask<M, Result<authorization::Context, ResponseError>>;

pub enum ResponseError {
    Denied,
    NoAuthorityAvailable,
}
