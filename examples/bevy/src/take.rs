use authorization_bevy::{
    AuthorizationEventPlugin, AuthorizationSet, Authorized, Identifier, IntoUnauthorizedContext,
    Unauthorized,
};
use bevy::prelude::*;

use crate::{interactable::Interactable, AuthorizationDatabase};

/// Take Plugin.
pub struct TakePlugin;

impl Plugin for TakePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AuthorizationEventPlugin::<AuthorizationDatabase, Take>::default())
            .add_systems(Update, take.after(AuthorizationSet))
            .add_systems(Update, user_interface)
            .add_systems(PostUpdate, inject);
    }
}

/// Take.
#[derive(Debug, Clone, Event)]
pub struct Take {
    /// [`Entity`] that is taking.
    pub who: Entity,

    /// [`Entity`] that is being taken.
    pub what: Entity,
}

impl IntoUnauthorizedContext for Take {
    fn into_unauthorized_context(
        event: &Unauthorized<Self>,
        query: &Query<&Identifier>,
    ) -> Option<authorization::Context> {
        let actor = query.get(event.actor);
        let who = query.get(event.data.who);
        let what = query.get(event.data.what);

        if let (Ok(actor), Ok(who), Ok(what)) = (actor, who, what) {
            Some(authorization::Context {
                action: authorization::Action {
                    noun: who.noun.clone(),
                    scope: who.scope.clone(),
                    verb: "take".to_string(),
                },
                data: std::collections::HashMap::from([
                    (
                        "who:id".to_string(),
                        std::collections::HashSet::from([who.id.to_string()]),
                    ),
                    (
                        "who:noun".to_string(),
                        std::collections::HashSet::from([who.noun.to_string()]),
                    ),
                    (
                        "who:scope".to_string(),
                        std::collections::HashSet::from([who.scope.to_string()]),
                    ),
                ]),
                principal: actor.into(),
                resource: what.into(),
            })
        } else {
            None
        }
    }
}

/// Taken
#[allow(clippy::manual_non_exhaustive)]
#[derive(Debug, Clone, Component)]
pub struct Taken {
    /// [`Entity`] that caused the taken.
    pub by: Option<Entity>,

    _private: (),
}

impl Taken {
    /// Returns `true` if this value was taken.
    pub fn is_taken(&self) -> bool {
        self.by.is_some()
    }
}

/// Actions [`Authorized`] [`Take`].
fn take(mut reader: EventReader<Authorized<Take>>, mut query: Query<&mut Taken>) {
    for event in reader.read() {
        if let Ok(mut taken) = query.get_mut(event.data.what) {
            if taken.by.is_none() {
                taken.by = Some(event.data.who);
            }
        }
    }
}

/// Injects [`Taken`] into new entities with [`Interactable`].
fn inject(mut commands: Commands, query: Query<Entity, (With<Interactable>, Without<Taken>)>) {
    for entity in &query {
        commands.entity(entity).insert(Taken {
            by: None,
            _private: (),
        });
    }
}

/// Logs when [`Entity`] was taken.
fn user_interface(query: Query<(Entity, &Taken), Changed<Taken>>, identifiers: Query<&Identifier>) {
    for (entity, taken) in &query {
        if let Some(by) = taken.by {
            let what = identifiers.get(entity);
            let who = identifiers.get(by);

            if let (Ok(what), Ok(who)) = (what, who) {
                info!("\n[TAKEN]\n  {what:?}\n    by: {who:?}");
            }
        } else {
            trace!("system triggered prematurely");
        }
    }
}
