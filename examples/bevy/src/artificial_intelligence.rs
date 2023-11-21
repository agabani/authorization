use bevy::{prelude::*, utils::Uuid};

use crate::{
    attack::Attack,
    authorization_bevy::{AuthorizationSet, Unauthorized},
    take::Take,
};

/// Artificial Intelligence Plugin.
pub struct ArtificialIntelligencePlugin;

impl Plugin for ArtificialIntelligencePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, inject).add_systems(
            Update,
            (try_attack_a_random_entity, try_take_a_random_entity).before(AuthorizationSet),
        );
    }
}

/// Artificial Intelligence.
#[derive(Debug, Clone, Component)]
pub struct ArtificialIntelligence;

/// Attack Cooldown.
#[derive(Debug, Clone, Component)]
struct AttackCooldown(Timer);

/// Take Cooldown.
#[derive(Debug, Clone, Component)]
struct TakeCooldown(Timer);

/// Injects state into newly created [`Entity`] with [`ArtificialIntelligence`].
fn inject(mut commands: Commands, query: Query<Entity, Added<ArtificialIntelligence>>) {
    for entity in &query {
        commands.entity(entity).insert((
            AttackCooldown(Timer::from_seconds(1.0, TimerMode::Repeating)),
            TakeCooldown(Timer::from_seconds(1.0, TimerMode::Repeating)),
        ));
    }
}

/// Try to attack a random [`Entity`].
fn try_attack_a_random_entity(
    time: Res<Time>,
    mut writer: EventWriter<Unauthorized<Attack>>,
    mut query: Query<(Entity, &mut AttackCooldown), With<ArtificialIntelligence>>,
    entities: Query<Entity>,
) {
    for (entity, mut cooldown) in &mut query {
        if cooldown.0.tick(time.delta()).just_finished() {
            cooldown.0.reset();

            let entity_count = entities.iter().count();

            if entity_count > 0 {
                let random_entity = entities
                    .iter()
                    .nth(Uuid::new_v4().as_u128() as usize % entity_count)
                    .unwrap();

                writer.send(Unauthorized {
                    actor: entity,
                    data: Attack {
                        who: entity,
                        what: random_entity,
                    },
                });
            }
        }
    }
}

/// Try to take a random [`Entity`].
fn try_take_a_random_entity(
    time: Res<Time>,
    mut writer: EventWriter<Unauthorized<Take>>,
    mut query: Query<(Entity, &mut TakeCooldown), With<ArtificialIntelligence>>,
    entities: Query<Entity>,
) {
    for (entity, mut cooldown) in &mut query {
        if cooldown.0.tick(time.delta()).just_finished() {
            cooldown.0.reset();

            let entity_count = entities.iter().count();

            if entity_count > 0 {
                let random_entity = entities
                    .iter()
                    .nth(Uuid::new_v4().as_u128() as usize % entity_count)
                    .unwrap();

                writer.send(Unauthorized {
                    actor: entity,
                    data: Take {
                        who: entity,
                        what: random_entity,
                    },
                });
            }
        }
    }
}
