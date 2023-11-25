use std::sync::{mpsc, Mutex};

use bevy::prelude::*;

/*
 * ============================================================================
 * Authorization
 * ============================================================================
 */

/// Authorized.
#[allow(clippy::manual_non_exhaustive)]
#[derive(Debug, Clone, Event)]
pub struct Authorized<T> {
    /// Actor.
    pub actor: Entity,

    /// Data.
    pub data: T,

    _private: (),
}

/// Unauthorized.
#[derive(Debug, Clone, Event)]
pub struct Unauthorized<T> {
    /// Actor.
    pub actor: Entity,

    /// Data.
    pub data: T,
}

impl<T> Unauthorized<T>
where
    T: Clone,
{
    /// Converts an [`Unauthorized`] into [`Authorized`].
    #[must_use]
    fn to_authorized(&self) -> Authorized<T> {
        Authorized {
            actor: self.actor,
            data: self.data.clone(),
            _private: (),
        }
    }
}

/*
 * ============================================================================
 * Networking
 * ============================================================================
 */

#[derive(Component)]
pub struct ConnectionTx(pub mpsc::Sender<Protocol>);

#[derive(Component)]
pub struct ConnectionRx(pub Mutex<mpsc::Receiver<Protocol>>);

pub struct NetworkHandshake {
    pub principal: authorization::Principal,
    pub tx: mpsc::Sender<Protocol>,
}

#[derive(Debug)]
pub enum Protocol {
    Connected(mpsc::Sender<Protocol>),
    Disconnected,
    Ping,
    Pong,
}

/*
 * ============================================================================
 * Game
 * ============================================================================
 */

#[derive(Component)]
pub struct Player;
