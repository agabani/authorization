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

/// Player Plugin.
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AuthorizationEventPlugin::<AuthorizationDatabase, SpawnPlayer>::default())
            .add_systems(Update, spawn.after(AuthorizationSet));
    }
}

/// Spawn Player.
#[derive(Debug, Clone, Event)]
pub struct SpawnPlayer;

impl IntoUnauthorizedContext for SpawnPlayer {
    fn into_unauthorized_context(
        event: &Unauthorized<Self>,
        query: &Query<&Identifier>,
    ) -> Option<authorization::Context> {
        let actor = query.get(event.actor);

        if let Ok(actor) = actor {
            Some(authorization::Context {
                action: authorization::Action {
                    noun: "player".to_string(),
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

/// Player.
#[derive(Debug, Clone, Component)]
pub struct Player;

/// Spawns a [`Player`].
fn spawn(mut commands: Commands, mut reader: EventReader<Authorized<SpawnPlayer>>) {
    for _ in reader.read() {
        commands.spawn((
            Player,
            Identifier {
                id: Uuid::new_v4().to_string(),
                noun: "player".to_string(),
                scope: "world".to_string(),
            },
            ArtificialIntelligence,
            Interactable,
            AttackStat(10),
            DefenseStat(8),
            HitPoints(10),
        ));
    }
}
