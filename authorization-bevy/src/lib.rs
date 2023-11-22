#![deny(clippy::pedantic, missing_docs)]
#![forbid(unsafe_code)]

//! Authorization Bevy.

#[cfg(test)]
mod tests;

use std::marker::PhantomData;

use authorization::{evaluate, Context, Effect, Policy, Principal, Resource};
use bevy::prelude::*;

/// Authorization Plugin.
pub struct AuthorizationPlugin;

impl Plugin for AuthorizationPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Audit>()
            .configure_sets(Update, AuthorizationSet);
    }
}

/// Authorization Event Plugin.
pub struct AuthorizationEventPlugin<D, T> {
    d: PhantomData<D>,
    t: PhantomData<T>,
}

impl<D, T> Default for AuthorizationEventPlugin<D, T> {
    fn default() -> Self {
        Self {
            d: PhantomData,
            t: PhantomData,
        }
    }
}

impl<D, T> Plugin for AuthorizationEventPlugin<D, T>
where
    D: Database + bevy::prelude::Resource + Send + Sync + 'static,
    T: IntoUnauthorizedContext + Clone + Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        app.add_event::<Authorized<T>>()
            .add_event::<Unauthorized<T>>()
            .add_systems(Update, authorize::<D, T>.in_set(AuthorizationSet));
    }
}

/// Authorization Set.
#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemSet)]
pub struct AuthorizationSet;

/// Audit.
#[derive(Debug, Clone, Event)]
pub struct Audit {
    /// Context.
    pub context: Context,

    /// Policy.
    pub policy: Option<Policy>,
}

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

/// Database.
pub trait Database {
    /// Returns all [`Policy`] containing the [`Principal`].
    fn query_by_principal(&self, principal: &Principal) -> Vec<Policy>;
}

/// Into Unauthorized Context.
pub trait IntoUnauthorizedContext
where
    Self: Sized,
{
    /// Returns [`Context`] for an [`Unauthorized`] event.
    fn into_unauthorized_context(
        event: &Unauthorized<Self>,
        query: &Query<&Identifier>,
    ) -> Option<Context>;
}

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

impl From<&Identifier> for Principal {
    fn from(value: &Identifier) -> Self {
        Principal {
            id: value.id.clone(),
            noun: value.noun.clone(),
            scope: value.scope.clone(),
        }
    }
}

impl From<&Identifier> for Resource {
    fn from(value: &Identifier) -> Self {
        Resource {
            id: value.id.clone(),
            noun: value.noun.clone(),
            scope: value.scope.clone(),
        }
    }
}

/// Checks if an [`Unauthorized`] event can be [`Authorized`] using any [`Policy`] in the [`Database`].
#[allow(clippy::needless_pass_by_value)]
fn authorize<D, T>(
    database: Res<D>,
    mut audit: EventWriter<Audit>,
    mut reader: EventReader<Unauthorized<T>>,
    mut writer: EventWriter<Authorized<T>>,
    identifiers: Query<&Identifier>,
) where
    D: Database + bevy::prelude::Resource,
    T: IntoUnauthorizedContext + Clone + Send + Sync + 'static,
{
    for event in reader.read() {
        if let Some(context) = T::into_unauthorized_context(event, &identifiers) {
            let policies = database.query_by_principal(&context.principal);
            let policy = evaluate(&context, &policies);

            audit.send(Audit {
                context,
                policy: policy.cloned(),
            });

            if let Some(policy) = policy {
                if policy.effect == Effect::Allow {
                    writer.send(event.to_authorized());
                }
            }
        };
    }
}
