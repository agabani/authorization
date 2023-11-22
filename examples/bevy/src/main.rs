mod artificial_intelligence;
mod attack;
mod game_master;
mod interactable;
mod kill;
mod loot;
mod monster;
mod player;
mod stats;
mod take;

use artificial_intelligence::ArtificialIntelligencePlugin;
use attack::AttackPlugin;
use authorization::Principal;
use authorization_bevy::{Audit, AuthorizationPlugin, AuthorizationSet, Database, Identifier};
use bevy::{
    log::LogPlugin,
    prelude::*,
    utils::{HashMap, Uuid},
};
use game_master::GameMasterPlugin;
use kill::KillPlugin;
use loot::LootPlugin;
use monster::MonsterPlugin;
use player::PlayerPlugin;
use take::TakePlugin;

pub fn main() {
    let mut app = App::new();

    app.add_plugins((
        MinimalPlugins,
        LogPlugin {
            level: bevy::log::Level::TRACE,
            filter: "bevy_app=info,bevy_ecs=info,wgpu=warn".to_string(),
        },
    ));

    app.add_plugins(AuthorizationPlugin)
        .add_systems(Startup, insert_policies_into_database)
        .add_systems(Update, user_interface_authorization.after(AuthorizationSet))
        .insert_resource(AuthorizationDatabase {
            ..Default::default()
        });

    app.add_event::<Despawned>()
        .add_plugins((
            ArtificialIntelligencePlugin,
            AttackPlugin,
            GameMasterPlugin,
            LootPlugin,
            KillPlugin,
            MonsterPlugin,
            PlayerPlugin,
            TakePlugin,
        ))
        .add_systems(
            Update,
            (
                remove_despawned_from_database,
                user_interface_despawned,
                user_interface_spawned,
            ),
        );

    app.run();
}

/// Despawned.
#[derive(Debug, Clone, Event)]
struct Despawned(Identifier);

/// Database.
#[derive(Default, Clone, Resource)]
pub struct AuthorizationDatabase {
    /// Data.
    data: HashMap<String, authorization::Policy>,
}

impl AuthorizationDatabase {
    /// Deletes policies by principal.
    fn delete_by_principal(&mut self, identifier: &Identifier) {
        self.data.retain(|_, policy| {
            !policy.principals.iter().all(|p| {
                p.id == identifier.id && p.noun == identifier.noun && p.scope == identifier.scope
            })
        });
    }

    /// Deletes policies by resource.
    fn delete_by_resource(&mut self, identifier: &Identifier) {
        self.data.retain(|_, policy| {
            !policy.resources.iter().all(|r| {
                r.id == identifier.id && r.noun == identifier.noun && r.scope == identifier.scope
            })
        });
    }

    /// Inserts a [`authorization::Policy`].
    fn insert(&mut self, policy: authorization::Policy) {
        self.data.insert(policy.id.clone(), policy);
    }
}

impl Database for AuthorizationDatabase {
    fn query_by_principal(&self, principal: &Principal) -> Vec<authorization::Policy> {
        self.data
            .values()
            .filter(|policy| {
                policy.principals.iter().any(|p| {
                    (p.id == "*" || p.id == principal.id)
                        && (p.noun == "*" || p.noun == principal.noun)
                        && (p.scope == "*" || p.scope == principal.scope)
                })
            })
            .cloned()
            .collect()
    }
}

/// Insert [`authorization::Policy`] policies into [`authorization_bevy::Database`].
fn insert_policies_into_database(mut database: ResMut<AuthorizationDatabase>) {
    let allow_player_attacking_monster = true;
    let allow_player_attacking_player = false;
    let allow_monster_attacking_monster = false;
    let allow_monster_attacking_player = false;

    // any game master in the world can spawn loot, monsters, and players into the world.
    database.insert(authorization::Policy {
        actions: vec![
            authorization::Action {
                noun: "loot".to_string(),
                scope: "world".to_string(),
                verb: "spawn".to_string(),
            },
            authorization::Action {
                noun: "monster".to_string(),
                scope: "world".to_string(),
                verb: "spawn".to_string(),
            },
            authorization::Action {
                noun: "player".to_string(),
                scope: "world".to_string(),
                verb: "spawn".to_string(),
            },
        ],
        conditions: vec![],
        effect: authorization::Effect::Allow,
        id: Uuid::new_v4().to_string(),
        principals: vec![authorization::Principal {
            id: "*".to_string(),
            noun: "game_master".to_string(),
            scope: "world".to_string(),
        }],
        resources: vec![authorization::Resource {
            id: "*".to_string(),
            noun: "*".to_string(),
            scope: "world".to_string(),
        }],
    });

    if allow_player_attacking_monster {
        // any player in the world can attack any monster in the world.
        database.insert(authorization::Policy {
            actions: vec![authorization::Action {
                noun: "*".to_string(),
                scope: "world".to_string(),
                verb: "attack".to_string(),
            }],
            conditions: vec![],
            effect: authorization::Effect::Allow,
            id: Uuid::new_v4().to_string(),
            principals: vec![authorization::Principal {
                id: "*".to_string(),
                noun: "player".to_string(),
                scope: "world".to_string(),
            }],
            resources: vec![authorization::Resource {
                id: "*".to_string(),
                noun: "monster".to_string(),
                scope: "world".to_string(),
            }],
        });
    }

    if allow_player_attacking_player {
        // any player in the world can attack any player in the world.
        database.insert(authorization::Policy {
            actions: vec![authorization::Action {
                noun: "*".to_string(),
                scope: "world".to_string(),
                verb: "attack".to_string(),
            }],
            conditions: vec![],
            effect: authorization::Effect::Allow,
            id: Uuid::new_v4().to_string(),
            principals: vec![authorization::Principal {
                id: "*".to_string(),
                noun: "player".to_string(),
                scope: "world".to_string(),
            }],
            resources: vec![authorization::Resource {
                id: "*".to_string(),
                noun: "player".to_string(),
                scope: "world".to_string(),
            }],
        });
    }

    if allow_monster_attacking_monster {
        // any monster in the world can attack any monster in the world.
        database.insert(authorization::Policy {
            actions: vec![authorization::Action {
                noun: "*".to_string(),
                scope: "world".to_string(),
                verb: "attack".to_string(),
            }],
            conditions: vec![],
            effect: authorization::Effect::Allow,
            id: Uuid::new_v4().to_string(),
            principals: vec![authorization::Principal {
                id: "*".to_string(),
                noun: "monster".to_string(),
                scope: "world".to_string(),
            }],
            resources: vec![authorization::Resource {
                id: "*".to_string(),
                noun: "monster".to_string(),
                scope: "world".to_string(),
            }],
        });
    }

    if allow_monster_attacking_player {
        // any monster in the world can attack any player in the world.
        database.insert(authorization::Policy {
            actions: vec![authorization::Action {
                noun: "*".to_string(),
                scope: "world".to_string(),
                verb: "attack".to_string(),
            }],
            conditions: vec![],
            effect: authorization::Effect::Allow,
            id: Uuid::new_v4().to_string(),
            principals: vec![authorization::Principal {
                id: "*".to_string(),
                noun: "monster".to_string(),
                scope: "world".to_string(),
            }],
            resources: vec![authorization::Resource {
                id: "*".to_string(),
                noun: "player".to_string(),
                scope: "world".to_string(),
            }],
        });
    }
}

/// Removes despawned entities from [`authorization_bevy::Database`].
fn remove_despawned_from_database(
    mut database: ResMut<AuthorizationDatabase>,
    mut reader: EventReader<Despawned>,
) {
    for event in reader.read() {
        database.delete_by_principal(&event.0);
        database.delete_by_resource(&event.0);
    }
}

/// Logs when [`Audit`] was emitted.
fn user_interface_authorization(mut reader: EventReader<Audit>) {
    for event in reader.read() {
        let context = &event.context;

        match &event.policy {
            Some(policy) => match policy.effect {
                authorization::Effect::Allow => {
                    debug!("\n[AUTHORIZATION]\n  explicit allow    \n    context: {context:?}    \n    policy: {policy:?}")
                }
                authorization::Effect::Deny => {
                    warn!("\n[AUTHORIZATION]\n  explicit deny    \n    context: {context:?}    \n    policy: {policy:?}")
                }
            },
            None => warn!("\n[AUTHORIZATION]\n  implicit deny  \n    context: {context:?}"),
        }
    }
}

/// Logs when [`Entity`] was despawned.
fn user_interface_despawned(mut reader: EventReader<Despawned>) {
    for event in reader.read() {
        info!("\n[DESPAWNED]\n  {:?}", event.0);
    }
}

/// Logs when [`Entity`] was spawned.
fn user_interface_spawned(query: Query<&Identifier, Added<Identifier>>) {
    for identifier in &query {
        info!("\n[SPAWNED]\n  {identifier:?}");
    }
}
