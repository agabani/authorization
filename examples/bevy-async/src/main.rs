mod authorization_bevy;
mod game_master;
mod monster;
mod player;

use std::sync::{Arc, Mutex};

use authorization_bevy::{AuthorizationService, Database, Identifier};
use bevy::{log::LogPlugin, prelude::*, utils::Uuid};
use game_master::{GameMaster, GameMasterPlugin};
use monster::MonsterPlugin;
use player::PlayerPlugin;

fn main() {
    let mut database = Database::default();
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

    let mut app = App::new();

    app.add_plugins((
        MinimalPlugins,
        LogPlugin {
            level: bevy::log::Level::TRACE,
            filter: "bevy_app=info,bevy_ecs=info,wgpu=warn".to_string(),
        },
    ));

    app.insert_resource(AuthorizationService::new(Arc::new(Mutex::new(database))));

    app.add_plugins((GameMasterPlugin, MonsterPlugin, PlayerPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, user_interface_spawned);

    app.run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        GameMaster,
        Identifier {
            id: Uuid::new_v4().to_string(),
            noun: "game_master".to_string(),
            scope: "world".to_string(),
        },
    ));
}

/// Logs when [`Entity`] was spawned.
fn user_interface_spawned(query: Query<&Identifier, Added<Identifier>>) {
    for identifier in &query {
        info!("\n[SPAWNED]\n  {identifier:?}");
    }
}
