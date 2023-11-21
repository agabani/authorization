mod artificial_intelligence;
mod attack;
mod authorization_bevy;
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
use authorization_bevy::{AuthorizationPlugin, Database, Identifier};
use bevy::{log::LogPlugin, prelude::*, utils::Uuid};
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
            filter: "wgpu=warn,bevy_ecs=info,bevy_app=info".to_string(),
        },
    ));

    app.add_plugins(AuthorizationPlugin)
        .add_systems(Startup, insert_policies_into_database);

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

/// Insert [`authorization::Policy`] policies into [`authorization_bevy::Database`].
fn insert_policies_into_database(mut database: ResMut<Database>) {
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
    mut database: ResMut<Database>,
    mut reader: EventReader<Despawned>,
) {
    for event in reader.read() {
        database.delete_by_principal(&event.0);
        database.delete_by_resource(&event.0);
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
