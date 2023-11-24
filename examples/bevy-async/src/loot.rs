use bevy::{prelude::*, utils::Uuid};

use crate::authorization_bevy::{Identifier, IntoContext, Unauthorized};

/// Loot Plugin.
pub struct LootPlugin;

impl Plugin for LootPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AuthorizationEventPlugin::<AuthorizationDatabase, SpawnLoot>::default())
            .add_systems(Update, spawn.after(AuthorizationSet))
            .add_systems(
                Update,
                (allow_all_entity_to_take_abandoned_loot, despawn_taken),
            );
    }
}

/// Spawn Loot.
#[derive(Debug, Clone, Event)]
pub struct SpawnLoot {
    /// [`Entity`] that owns the loot.
    pub owner: Entity,
}

impl IntoContext for SpawnLoot {
    fn into_context(
        event: &Unauthorized<Self>,
        query: &Query<&Identifier>,
    ) -> Option<authorization::Context> {
        let actor = query.get(event.actor);

        if let Ok(actor) = actor {
            Some(authorization::Context {
                action: authorization::Action {
                    noun: "loot".to_string(),
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

/// Loot.
#[derive(Debug, Clone, Component)]
pub struct Loot;

/// Abandoned Timer.
#[derive(Debug, Clone, Component)]
struct AbandonedTimer(Timer);
