use bevy::prelude::*;

use crate::{attack::Damaged, authorization_bevy::Identifier, stats::HitPoints};

/// Kill Plugin.
pub struct KillPlugin;

impl Plugin for KillPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (despawn_killed_entities, user_interface))
            .add_systems(PostUpdate, inject);
    }
}

/// Killed.
#[allow(clippy::manual_non_exhaustive)]
#[derive(Debug, Clone, Component)]
pub struct Killed {
    /// [`Entity`] that caused the death.
    pub by: Entity,

    _private: (),
}

/// Despawn killed entities.
fn despawn_killed_entities(mut commands: Commands, query: Query<Entity, With<Killed>>) {
    for entity in &query {
        if let Some(mut entity) = commands.get_entity(entity) {
            entity.despawn()
        }
    }
}

/// Injects [`Killed`] into entities with [`HitPoints`] when hit points reach 0.
fn inject(mut commands: Commands, query: Query<(Entity, &HitPoints, &Damaged), Without<Killed>>) {
    for (entity, hit_points, damaged) in &query {
        if hit_points.0 == 0 {
            commands.entity(entity).insert(Killed {
                by: damaged.by.expect("system ran prematurely"),
                _private: (),
            });
        }
    }
}

/// Logs when [`Entity`] was killed.
fn user_interface(query: Query<(Entity, &Killed), Added<Killed>>, identifiers: Query<&Identifier>) {
    for (entity, killed) in &query {
        let what = identifiers.get(entity);
        let who = identifiers.get(killed.by);

        if let (Ok(what), Ok(who)) = (what, who) {
            info!("\n[KILLED]\n  {what:?}\n    by: {who:?}");
        }
    }
}
