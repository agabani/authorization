use bevy::{prelude::*, utils::Uuid};

use crate::{
    artificial_intelligence::ArtificialIntelligence,
    authorization_bevy::{AuthorizationTask, Identifier, IntoContext},
    stats::{AttackStat, DefenseStat, HitPoints},
};

/// Player Plugin.
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, spawn_player);
    }
}

/// Player.
#[derive(Debug, Clone, Component)]
pub struct Player;

#[derive(Debug, Clone)]
pub struct SpawnPlayer;

impl IntoContext for SpawnPlayer {
    fn into_context(
        unauthorized: &crate::authorization_bevy::Unauthorized<Self>,
        query: &Query<&crate::authorization_bevy::Identifier>,
    ) -> Option<authorization::Context> {
        let actor = query.get(unauthorized.actor);

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

fn spawn_player(
    mut commands: Commands,
    mut query: Query<(Entity, &mut AuthorizationTask<SpawnPlayer>)>,
) {
    query.for_each_mut(|(entity, mut task)| {
        if let Some(result) = task.poll_once() {
            match result {
                Ok(_) => {
                    commands
                        .entity(entity)
                        .remove::<AuthorizationTask<SpawnPlayer>>()
                        .insert((
                            Player,
                            Identifier {
                                id: Uuid::new_v4().to_string(),
                                noun: "player".to_string(),
                                scope: "world".to_string(),
                            },
                            ArtificialIntelligence,
                            AttackStat(10),
                            DefenseStat(8),
                            HitPoints(10),
                        ));
                }
                Err(error) => {
                    println!("{error:?}");
                    commands.entity(entity).despawn();
                }
            }
        }
    });
}
