use std::marker::PhantomData;

use bevy::{
    prelude::*,
    utils::{HashMap, Uuid},
};

/// Authorization Plugin.
pub struct AuthorizationPlugin;

impl Plugin for AuthorizationPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, AuthorizationSet)
            .insert_resource(Database::default());

        app.add_event::<Audit>()
            .add_systems(Update, user_interface.after(AuthorizationSet));
    }
}

/// Authorization Event Plugin.
pub struct AuthorizationEventPlugin<T> {
    marker: PhantomData<T>,
}

impl<T> Default for AuthorizationEventPlugin<T> {
    fn default() -> Self {
        Self {
            marker: Default::default(),
        }
    }
}

impl<T> Plugin for AuthorizationEventPlugin<T>
where
    T: Contextual + Clone + Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        app.add_event::<Authorized<T>>()
            .add_event::<Unauthorized<T>>()
            .add_systems(Update, authorize::<T>.in_set(AuthorizationSet));
    }
}

/// Authorization Set.
#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemSet)]
pub struct AuthorizationSet;

/// Audit.
#[derive(Debug, Clone, Event)]
pub struct Audit {
    /// Context.
    pub context: authorization::Context,

    /// Policy.
    pub policy: Option<authorization::Policy>,
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
#[derive(Default, Resource)]
pub struct Database {
    /// Data.
    data: HashMap<String, authorization::Policy>,
}

impl Database {
    /// Deletes policies by principal.
    pub fn delete_by_principal(&mut self, identifier: &Identifier) {
        self.data.retain(|_, policy| {
            !policy.principals.iter().all(|principal| {
                principal.id == identifier.id.to_string()
                    && principal.noun == identifier.noun
                    && principal.scope == identifier.scope
            })
        });
    }

    /// Deletes policies by resource.
    pub fn delete_by_resource(&mut self, identifier: &Identifier) {
        self.data.retain(|_, policy| {
            !policy.resources.iter().all(|resource| {
                resource.id == identifier.id.to_string()
                    && resource.noun == identifier.noun
                    && resource.scope == identifier.scope
            })
        });
    }

    /// Inserts a [`authorization::Policy`].
    pub fn insert(&mut self, policy: authorization::Policy) {
        self.data.insert(policy.id.clone(), policy);
    }

    /// Returns all [`authorization::Policy`] containing the principal.
    pub fn query_by_principal(&self, identifier: &Identifier) -> Vec<authorization::Policy> {
        self.data
            .values()
            .filter(|policy| {
                policy.principals.iter().any(|principal| {
                    (principal.id == "*" || principal.id == identifier.id.to_string())
                        && (principal.noun == "*" || principal.noun == identifier.noun)
                        && (principal.scope == "*" || principal.scope == identifier.scope)
                })
            })
            .cloned()
            .collect()
    }
}

/// Identifier.
#[derive(Debug, Clone, Component)]
pub struct Identifier {
    /// Id.
    pub id: Uuid,

    /// Noun.
    pub noun: String,

    /// Scope.
    pub scope: String,
}

impl From<&Identifier> for authorization::Principal {
    fn from(value: &Identifier) -> Self {
        authorization::Principal {
            id: value.id.to_string(),
            noun: value.noun.clone(),
            scope: value.scope.clone(),
        }
    }
}

impl From<&Identifier> for authorization::Resource {
    fn from(value: &Identifier) -> Self {
        authorization::Resource {
            id: value.id.to_string(),
            noun: value.noun.clone(),
            scope: value.scope.clone(),
        }
    }
}

/// Checks if an [`Unauthorized`] event can be [`Authorized`] using [`authorization::Policy`] in the [`Database`].
fn authorize<T>(
    database: Res<Database>,
    mut audit: EventWriter<Audit>,
    mut reader: EventReader<Unauthorized<T>>,
    mut writer: EventWriter<Authorized<T>>,
    identifiers: Query<&Identifier>,
) where
    T: Contextual + Clone + Send + Sync + 'static,
{
    for event in reader.read() {
        let actor = identifiers.get(event.actor);
        let context = T::context(event, &identifiers);

        if let (Ok(actor), Some(context)) = (actor, context) {
            let policies = database.query_by_principal(actor);
            let policy = authorization::evaluate(&context, &policies);

            audit.send(Audit {
                context: context.clone(),
                policy: policy.cloned(),
            });

            let authorized = match authorization::evaluate(&context, &policies) {
                Some(policy) => match policy.effect {
                    authorization::Effect::Allow => true,
                    authorization::Effect::Deny => false,
                },
                None => false,
            };

            if authorized {
                writer.send(event.to_authorized());
            }
        };
    }
}

/// Contextual.
pub trait Contextual
where
    Self: Sized,
{
    /// Returns [`authorization::Context`] for an [`Unauthorized`] event.
    fn context(
        event: &Unauthorized<Self>,
        query: &Query<&Identifier>,
    ) -> Option<authorization::Context>;
}

/// Logs when [`Audit`] was emitted.
fn user_interface(mut reader: EventReader<Audit>) {
    for event in reader.read() {
        let context = &event.context;

        match &event.policy {
            Some(policy) => match policy.effect {
                authorization::Effect::Allow => {
                    debug!("\n[AUTHORIZATION]\n  explicit allow    \n    context: {context:?}    \n    policy: {policy:?}")
                }
                authorization::Effect::Deny => {
                    warn!("\n[AUTHORIZATION]\n  explicit deny    \n    context: {context:?}    \n    policy: {policy:?}")
                }
            },
            None => warn!("\n[AUTHORIZATION]\n  implicit deny  \n    context: {context:?}"),
        }
    }
}
