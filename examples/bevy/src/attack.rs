use authorization_bevy::{
    AuthorizationEventPlugin, AuthorizationSet, Authorized, Identifier, IntoUnauthorizedContext,
    Unauthorized,
};
use bevy::prelude::*;

use crate::{
    stats::{AttackStat, DefenseStat, HitPoints},
    AuthorizationDatabase,
};

/// Attack Plugin.
pub struct AttackPlugin;

impl Plugin for AttackPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AuthorizationEventPlugin::<AuthorizationDatabase, Attack>::default())
            .add_event::<Attacked>()
            .add_systems(Update, attack.after(AuthorizationSet))
            .add_systems(PostUpdate, (inject, user_interface));
    }
}

/// Attack.
#[derive(Debug, Clone, Event)]
pub struct Attack {
    /// [`Entity`] that is attacking.
    pub who: Entity,

    /// [`Entity`] that is being attacked.
    pub what: Entity,
}

impl IntoUnauthorizedContext for Attack {
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
                    verb: "attack".to_string(),
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

/// Attacked.
#[allow(clippy::manual_non_exhaustive)]
#[derive(Debug, Clone, Event)]
pub struct Attacked {
    /// Damaged caused by the attack.
    pub damage: u32,

    /// [`Entity`] that attacked.
    pub who: Entity,

    /// [`Entity`] that was attacked.
    pub what: Entity,

    /// [`Entity`] that was attacked remaining hit points.
    pub what_hit_points: u32,

    _private: (),
}

/// Damage.
#[allow(clippy::manual_non_exhaustive)]
#[derive(Debug, Clone, Component)]
pub struct Damaged {
    /// [`Entity`] that caused the last damage.
    pub by: Option<Entity>,

    _private: (),
}

/// Actions [`Authorized`] [`Attack`].
fn attack(
    mut reader: EventReader<Authorized<Attack>>,
    mut writer: EventWriter<Attacked>,
    attacker: Query<&AttackStat>,
    mut defender: Query<(&mut HitPoints, &DefenseStat, Option<&mut Damaged>)>,
) {
    for event in reader.read() {
        let who = attacker.get(event.data.who);
        let what = defender.get_mut(event.data.what);

        let (Ok(attack_stat), Ok((mut hit_points, defense_stat, damaged))) = (who, what) else {
            break;
        };

        if let Some(mut damaged) = damaged {
            if hit_points.0 > 0 {
                let attack_stat = attack_stat.0 as i32;
                let defense_stat = defense_stat.0 as i32;

                let damage = (attack_stat - defense_stat).max(0);
                hit_points.0 = (hit_points.0 as i32 - damage).max(0) as u32;

                writer.send(Attacked {
                    who: event.data.who,
                    what: event.data.what,
                    damage: damage as u32,
                    what_hit_points: hit_points.0,
                    _private: (),
                });

                if damage > 0 {
                    damaged.by = Some(event.data.who)
                }
            }
        } else {
            trace!("target has 0 hit points");
        }
    }
}

/// Injects [`Damaged`] into new entities with [`HitPoints`].
fn inject(mut commands: Commands, query: Query<Entity, (With<HitPoints>, Without<Damaged>)>) {
    for entity in &query {
        commands.entity(entity).insert(Damaged {
            by: None,
            _private: (),
        });
    }
}

/// Logs when [`Entity`] was attacked.
fn user_interface(mut reader: EventReader<Attacked>, identifiers: Query<&Identifier>) {
    for event in reader.read() {
        let what = identifiers.get(event.what);
        let who = identifiers.get(event.who);

        if let (Ok(what), Ok(who)) = (what, who) {
            info!(
                "\n[ATTACKED]\n  {what:?}\n    by: {who:?}\n    damage: {}\n    remaining hit points: {}",
                event.damage, event.what_hit_points
            );
        }
    }
}
