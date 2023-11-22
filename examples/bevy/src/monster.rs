use authorization_bevy::{
    AuthorizationEventPlugin, AuthorizationSet, Authorized, Identifier, IntoUnauthorizedContext,
    Unauthorized,
};
use bevy::{prelude::*, utils::Uuid};

use crate::{
    artificial_intelligence::ArtificialIntelligence,
    interactable::Interactable,
    stats::{AttackStat, DefenseStat, HitPoints},
    AuthorizationDatabase,
};

/// Monster Plugin.
pub struct MonsterPlugin;

impl Plugin for MonsterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AuthorizationEventPlugin::<
            AuthorizationDatabase,
            SpawnMonster,
        >::default())
            .add_systems(Update, spawn.after(AuthorizationSet));
    }
}

/// Spawn Monster.
#[derive(Debug, Clone, Event)]
pub struct SpawnMonster;

impl IntoUnauthorizedContext for SpawnMonster {
    fn into_unauthorized_context(
        event: &Unauthorized<Self>,
        query: &Query<&Identifier>,
    ) -> Option<authorization::Context> {
        let actor = query.get(event.actor);

        if let Ok(actor) = actor {
            Some(authorization::Context {
                action: authorization::Action {
                    noun: "monster".to_string(),
                    scope: "world".to_string(),
                    verb: "spawn".to_string(),
                },
                data: std::collections::HashMap::new(),
                principal: actor.into(),
                resource: authorization::Resource {
                    id: "".to_string(),
                    noun: "".to_string(),
                    scope: "world".to_string(),
                },
            })
        } else {
            None
        }
    }
}

/// Monster.
#[derive(Debug, Clone, Component)]
pub struct Monster;

/// Spawns a [`Monster`].
fn spawn(mut commands: Commands, mut reader: EventReader<Authorized<SpawnMonster>>) {
    for _ in reader.read() {
        commands.spawn((
            Monster,
            Identifier {
                id: Uuid::new_v4().to_string(),
                noun: "monster".to_string(),
                scope: "world".to_string(),
            },
            ArtificialIntelligence,
            Interactable,
            AttackStat(9),
            DefenseStat(5),
            HitPoints(10),
        ));
    }
}
