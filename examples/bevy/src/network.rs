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
pub struct ConnectionTx(pub mpsc::Sender<Frame>);

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

/*
 * ============================================================================
 * Send
 * ============================================================================
 */

pub fn send(commands: &mut Commands, entity: Entity, tx: &ConnectionTx, frame: Frame) -> bool {
    let result = tx.0.send(frame);
    if result.is_err() {
        commands.entity(entity).despawn();
        warn!("disconnected");
    }
    result.is_ok()
}
