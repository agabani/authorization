use bevy::{prelude::*, utils::Uuid};

use crate::{
    artificial_intelligence::ArtificialIntelligence,
    authorization_bevy::{AuthorizationTask, Identifier, IntoContext},
    stats::{AttackStat, DefenseStat, HitPoints},
};

/// Monster Plugin.
pub struct MonsterPlugin;

impl Plugin for MonsterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, spawn_monster);
    }
}

/// Monster.
#[derive(Debug, Clone, Component)]
pub struct Monster;

#[derive(Debug, Clone)]
pub struct SpawnMonster;

impl IntoContext for SpawnMonster {
    fn into_context(
        unauthorized: &crate::authorization_bevy::Unauthorized<Self>,
        query: &Query<&crate::authorization_bevy::Identifier>,
    ) -> Option<authorization::Context> {
        let actor = query.get(unauthorized.actor);

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

fn spawn_monster(
    mut commands: Commands,
    mut query: Query<(Entity, &mut AuthorizationTask<SpawnMonster>)>,
) {
    query.for_each_mut(|(entity, mut task)| {
        if let Some(result) = task.poll_once() {
            match result {
                Ok(_) => {
                    commands
                        .entity(entity)
                        .remove::<AuthorizationTask<SpawnMonster>>()
                        .insert((
                            Monster,
                            Identifier {
                                id: Uuid::new_v4().to_string(),
                                noun: "monster".to_string(),
                                scope: "world".to_string(),
                            },
                            ArtificialIntelligence,
                            AttackStat(9),
                            DefenseStat(5),
                            HitPoints(10),
                        ));
                }
                Err(_) => {
                    if let Some(mut entity) = commands.get_entity(entity) {
                        entity.despawn();
                    }
                }
            }
        }
    });
}
