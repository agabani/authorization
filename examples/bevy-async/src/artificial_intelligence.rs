use bevy::{prelude::*, utils::Uuid};

use crate::{
    attack::Attack,
    authorization_bevy::{AuthorizationService, AuthorizationTask, Identifier, Unauthorized},
};

/// Artificial Intelligence Plugin.
pub struct ArtificialIntelligencePlugin;

impl Plugin for ArtificialIntelligencePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, inject).add_systems(
            Update,
            (try_attack_a_random_entity, remove_completed_attack_request),
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
    mut commands: Commands,
    time: Res<Time>,
    authorization_service: Res<AuthorizationService>,
    mut query: Query<
        (Entity, &mut AttackCooldown),
        (
            With<ArtificialIntelligence>,
            Without<AuthorizationTask<Attack>>,
        ),
    >,
    entities: Query<Entity>,
    identifiers: Query<&Identifier>,
) {
    query.for_each_mut(|(entity, mut cooldown)| {
        if cooldown.0.tick(time.delta()).just_finished() {
            cooldown.0.reset();

            let entity_count = entities.iter().count();

            if entity_count > 0 {
                let random_entity = entities
                    .iter()
                    .nth(Uuid::new_v4().as_u128() as usize % entity_count)
                    .unwrap();

                if let Some(task) = authorization_service.authorize(
                    Unauthorized {
                        actor: entity,
                        data: Attack {
                            who: entity,
                            what: random_entity,
                        },
                    },
                    &identifiers,
                ) {
                    commands.entity(entity).insert(task);
                };
            }
        }
    });
}

fn remove_completed_attack_request(
    mut commands: Commands,
    mut query: Query<(Entity, &mut AuthorizationTask<Attack>), With<ArtificialIntelligence>>,
) {
    query.for_each_mut(|(entity, mut task)| {
        if task.poll_once().is_some() {
            commands
                .entity(entity)
                .remove::<AuthorizationTask<Attack>>();
        }
    });
}
