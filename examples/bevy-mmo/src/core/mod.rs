use std::{
    marker::PhantomData,
    sync::{mpsc, Mutex},
};

use bevy::prelude::*;

/*
 * ============================================================================
 * Identity
 * ============================================================================
 */

/// Identifier.
#[derive(Debug, Clone, Component)]
pub struct Identifier {
    /// Id.
    pub id: String,

    /// Noun.
    pub noun: String,

    /// Scope.
    pub scope: String,
}

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

pub enum AuthorizationError {
    Context,
    Denied,
    Disconnected,
}

#[derive(Component)]
pub struct AuthorizationService;

impl AuthorizationService {
    /// Create a new [`AuthorizationTask`].
    pub fn authorize<T>(
        &self,
        unauthorized: Unauthorized<T>,
        identifiers: &Query<&Identifier>,
        tx: &ConnectionTx,
    ) -> Result<AuthorizationTask<T>, AuthorizationError>
    where
        T: IntoContext + Clone + Send + 'static,
    {
        let context =
            T::into_context(&unauthorized, identifiers).ok_or(AuthorizationError::Context)?;

        let (result_tx, result_rx) = mpsc::channel();

        tx.0.send(Protocol::Authorize(context, result_tx))
            .map_err(|_| AuthorizationError::Disconnected)?;

        Ok(AuthorizationTask {
            result: None,
            rx: result_rx,
            marker: PhantomData,
        })
    }
}

#[derive(Component)]
pub struct AuthorizationTask<T> {
    result: Option<Result<authorization::Context, AuthorizationError>>,
    rx: mpsc::Receiver<Result<authorization::Context, AuthorizationError>>,
    marker: PhantomData<T>,
}

impl<T> AuthorizationTask<T> {
    /// Polls a future just once and returns an [`Option`] with the result.
    pub fn poll_once(&mut self) -> &Option<Result<authorization::Context, AuthorizationError>> {
        if self.result.is_some() {
            return &self.result;
        }

        match self.rx.try_recv() {
            Ok(result) => {
                self.result = Some(result);
                &self.result
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                todo!()
            }
            Err(mpsc::TryRecvError::Empty) => &self.result,
        }
    }
}

/// Into Context.
pub trait IntoContext
where
    Self: Sized,
{
    /// Returns [`Context`] for an [`Unauthorized`] event.
    fn into_context(
        unauthorized: &Unauthorized<Self>,
        identifiers: &Query<&Identifier>,
    ) -> Option<authorization::Context>;
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
    Authorize(
        authorization::Context,
        mpsc::Sender<Result<authorization::Context, AuthorizationError>>,
    ),
}

/*
 * ============================================================================
 * Game
 * ============================================================================
 */

#[derive(Component)]
pub struct Player;
