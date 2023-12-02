use std::{
    marker::PhantomData,
    sync::{mpsc, Arc, Mutex},
};

use bevy::prelude::*;

use crate::{async_task::AsyncTask, monster::Monster, player::Player};

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
    pub tx: mpsc::Sender<Frame>,
}

/*
 * ============================================================================
 * Connection
 * ============================================================================
 */

#[derive(Component)]
pub struct ConnectionRx(pub Mutex<mpsc::Receiver<Frame>>);

#[derive(Component)]
pub struct ConnectionTx(mpsc::Sender<Frame>);

impl ConnectionTx {
    pub fn new(sender: mpsc::Sender<Frame>) -> Self {
        Self(sender)
    }

    pub fn send(&self, frame: Frame) -> Result<(), SendError> {
        self.0.send(frame).map_err(|_| SendError::Disconnected)
    }
}

pub enum SendError {
    Disconnected,
}

pub enum Frame {
    Connected(mpsc::Sender<Frame>),
    Disconnect,
    Ping,
    Pong,
    Request(
        authorization::Context,
        mpsc::Sender<Result<authorization::Context, ResponseError>>,
    ),
    Broadcast(FrameEvent),
    Replicate,
}

#[derive(Clone)]
pub enum FrameEvent {
    Monster(authorization::Context),
    Player(authorization::Context),
}

impl FrameEvent {
    pub fn spawn_broadcast(self, commands: &mut Commands) {
        match self {
            FrameEvent::Monster(context) => {
                commands.spawn(Broadcast {
                    context,
                    marker: PhantomData::<Monster>,
                });
            }
            FrameEvent::Player(context) => {
                commands.spawn(Broadcast {
                    context,
                    marker: PhantomData::<Player>,
                });
            }
        }
    }
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
