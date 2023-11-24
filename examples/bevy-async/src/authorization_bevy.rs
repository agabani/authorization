use std::sync::{Arc, Mutex};

use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
    utils::HashMap,
};
use futures_lite::future;

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

impl From<&Identifier> for authorization::Principal {
    fn from(value: &Identifier) -> Self {
        authorization::Principal {
            id: value.id.clone(),
            noun: value.noun.clone(),
            scope: value.scope.clone(),
        }
    }
}

impl From<&Identifier> for authorization::Resource {
    fn from(value: &Identifier) -> Self {
        authorization::Resource {
            id: value.id.clone(),
            noun: value.noun.clone(),
            scope: value.scope.clone(),
        }
    }
}

/// Authorized.
#[allow(clippy::manual_non_exhaustive)]
#[derive(Debug, Clone)]
pub struct Authorized<T> {
    /// Actor.
    pub actor: Entity,

    /// Data.
    pub data: T,

    _private: (),
}

/// Unauthorized.
#[derive(Debug, Clone)]
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

/// Authorization Error.
#[derive(Debug)]
pub enum AuthorizationError {
    /// Denied.
    Denied,
}

#[derive(Debug, Resource)]
pub struct AuthorizationService {
    database: Arc<Mutex<Database>>,
}

impl AuthorizationService {
    /// Create a new [`AuthorizationService`].
    pub fn new(database: Arc<Mutex<Database>>) -> Self {
        Self { database }
    }

    /// Create a new [`AuthorizationTask`].
    pub fn authorize<T>(
        &self,
        unauthorized: Unauthorized<T>,
        identifiers: &Query<&Identifier>,
    ) -> Option<AuthorizationTask<T>>
    where
        T: IntoContext + Clone + Send + 'static,
    {
        T::into_context(&unauthorized, identifiers)
            .map(|context| AuthorizationTask::new(unauthorized, context, self.database.clone()))
    }
}

/// Authorization Task.
#[derive(Debug, Component)]
pub struct AuthorizationTask<T> {
    result: Option<Result<Authorized<T>, AuthorizationError>>,
    task: Task<Result<Authorized<T>, AuthorizationError>>,
}

impl<T> AuthorizationTask<T>
where
    T: Clone + Send + 'static,
{
    /// Creates a new [`AuthorizationTask`].
    pub fn new(
        unauthorized: Unauthorized<T>,
        context: authorization::Context,
        database: Arc<Mutex<Database>>,
    ) -> Self {
        let task = AsyncComputeTaskPool::get().spawn(async move {
            let context = context;
            let database = database;
            let unauthorized = unauthorized;

            future::yield_now().await;

            let policies = database
                .lock()
                .expect("mutex was poisoned")
                .query_by_principal(&context.principal);

            let policy = authorization::evaluate(&context, &policies);

            match policy {
                Some(policy) => match policy.effect {
                    authorization::Effect::Allow => {
                        debug!("\n[AUTHORIZATION]\n  explicit allow    \n    context: {context:?}    \n    policy: {policy:?}");
                        Ok(unauthorized.to_authorized())
                    },
                    authorization::Effect::Deny => {
                        warn!("\n[AUTHORIZATION]\n  explicit deny    \n    context: {context:?}    \n    policy: {policy:?}");
                        Err(AuthorizationError::Denied)
                    }
                },
                None => {
                    warn!("\n[AUTHORIZATION]\n  implicit deny  \n    context: {context:?}");
                    Err(AuthorizationError::Denied)
                },
            }
        });

        AuthorizationTask { result: None, task }
    }

    /// Polls a future just once and returns an [`Option`] with the result.
    pub fn poll_once(&mut self) -> &Option<Result<Authorized<T>, AuthorizationError>> {
        if self.result.is_some() {
            return &self.result;
        }

        self.result = future::block_on(future::poll_once(&mut self.task));
        &self.result
    }
}

/// Database.
#[derive(Debug, Default, Clone)]
pub struct Database {
    /// Data.
    data: HashMap<String, authorization::Policy>,
}

impl Database {
    /// Deletes policies by principal.
    pub fn delete_by_principal(&mut self, identifier: &Identifier) {
        self.data.retain(|_, policy| {
            !policy.principals.iter().all(|p| {
                p.id == identifier.id && p.noun == identifier.noun && p.scope == identifier.scope
            })
        });
    }

    /// Deletes policies by resource.
    pub fn delete_by_resource(&mut self, identifier: &Identifier) {
        self.data.retain(|_, policy| {
            !policy.resources.iter().all(|r| {
                r.id == identifier.id && r.noun == identifier.noun && r.scope == identifier.scope
            })
        });
    }

    /// Inserts a [`authorization::Policy`].
    pub fn insert(&mut self, policy: authorization::Policy) {
        self.data.insert(policy.id.clone(), policy);
    }

    /// Returns all [`Policy`] containing the [`Principal`].
    pub fn query_by_principal(
        &self,
        principal: &authorization::Principal,
    ) -> Vec<authorization::Policy> {
        self.data
            .values()
            .filter(|policy| {
                policy.principals.iter().any(|p| {
                    (p.id == "*" || p.id == principal.id)
                        && (p.noun == "*" || p.noun == principal.noun)
                        && (p.scope == "*" || p.scope == principal.scope)
                })
            })
            .cloned()
            .collect()
    }
}
