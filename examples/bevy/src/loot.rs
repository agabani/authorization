use bevy::{prelude::*, utils::Uuid};

use crate::{
    authorization_bevy::{
        AuthorizationEventPlugin, AuthorizationSet, Authorized, Contextual, Database, Identifier,
        Unauthorized,
    },
    interactable::Interactable,
    take::Taken,
    Despawned,
};

/// Loot Plugin.
pub struct LootPlugin;

impl Plugin for LootPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AuthorizationEventPlugin::<SpawnLoot>::default())
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

impl Contextual for SpawnLoot {
    fn context(
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

/// Spawns a [`Loot`].
fn spawn(
    mut commands: Commands,
    mut database: ResMut<Database>,
    mut reader: EventReader<Authorized<SpawnLoot>>,
    identifiers: Query<&Identifier>,
) {
    for event in reader.read() {
        if let Ok(target) = identifiers.get(event.data.owner) {
            let identifier = Identifier {
                id: Uuid::new_v4(),
                noun: "loot".to_string(),
                scope: "world".to_string(),
            };

            database.insert(authorization::Policy {
                actions: vec![authorization::Action {
                    noun: "*".to_string(),
                    scope: "world".to_string(),
                    verb: "take".to_string(),
                }],
                conditions: Default::default(),
                effect: authorization::Effect::Allow,
                id: Uuid::new_v4().to_string(),
                principals: vec![authorization::Principal {
                    id: target.id.to_string(),
                    noun: target.noun.clone(),
                    scope: target.scope.clone(),
                }],
                resources: vec![(&identifier).into()],
            });

            commands.spawn((
                Loot,
                AbandonedTimer(Timer::from_seconds(3.0, TimerMode::Once)),
                Interactable,
                identifier,
            ));
        };
    }
}

/// Allow all [`Entity`] to take abandoned [`Loot`].
fn allow_all_entity_to_take_abandoned_loot(
    time: Res<Time>,
    mut database: ResMut<Database>,
    mut query: Query<(&Identifier, &mut AbandonedTimer), With<Loot>>,
) {
    for (identifier, mut timer) in &mut query {
        if timer.0.tick(time.delta()).just_finished() {
            database.insert(authorization::Policy {
                actions: vec![authorization::Action {
                    noun: "*".to_string(),
                    scope: "world".to_string(),
                    verb: "take".to_string(),
                }],
                conditions: Default::default(),
                effect: authorization::Effect::Allow,
                id: Uuid::new_v4().to_string(),
                principals: vec![authorization::Principal {
                    id: "*".to_string(),
                    noun: "*".to_string(),
                    scope: "world".to_string(),
                }],
                resources: vec![identifier.into()],
            });

            info!("\n[LOOT EXCLUSIVITY EXPIRED]\n  {identifier:?}");
        }
    }
}

/// Despawn [`Taken`] [`Loot`].
#[allow(clippy::type_complexity)]
fn despawn_taken(
    mut commands: Commands,
    query: Query<(Entity, &Taken, Option<&Identifier>), (With<Loot>, Changed<Taken>)>,
    mut writer: EventWriter<Despawned>,
) {
    for (entity, taken, identifier) in &query {
        if taken.is_taken() {
            commands.entity(entity).despawn();

            if let Some(identifier) = identifier {
                writer.send(Despawned(identifier.clone()));
            }
        } else {
            trace!("system triggered prematurely");
        }
    }
}
