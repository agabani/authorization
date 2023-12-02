mod artificial_intelligence;
mod async_task;
mod authority;
mod identity;
mod monster;
mod network;
mod network_client;
mod network_server;
mod player;

use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
    time::Duration,
};

use artificial_intelligence::ArtificialIntelligencePlugin;
use authority::AuthorityPlugin;
use bevy::{prelude::*, utils::Uuid};
use identity::{Identifiers, Principal};
use monster::MonsterPlugin;
use network::{ConnectionsRx, ConnectionsTx, Handshake};
use network_client::NetworkClientPlugin;
use network_server::NetworkServerPlugin;
use player::PlayerPlugin;

fn main() {
    let (connections_tx, connections_rx) = mpsc::channel();
    let connections_rx = Arc::new(Mutex::new(connections_rx));

    let threads = [
        thread::spawn({
            let connections_rx = connections_rx.clone();
            let connections_tx = connections_tx.clone();
            move || {
                run(
                    connections_rx,
                    connections_tx,
                    authorization::Principal {
                        id: Uuid::new_v4().to_string(),
                        noun: "replication".to_string(),
                        scope: "actor".to_string(),
                    },
                )
            }
        }),
        thread::spawn({
            let connections_rx = connections_rx.clone();
            let connections_tx = connections_tx.clone();
            move || {
                run(
                    connections_rx,
                    connections_tx,
                    authorization::Principal {
                        id: Uuid::new_v4().to_string(),
                        noun: "artificial_intelligence".to_string(),
                        scope: "actor".to_string(),
                    },
                )
            }
        }),
        thread::spawn({
            let connections_rx = connections_rx.clone();
            let connections_tx = connections_tx.clone();
            move || {
                run(
                    connections_rx,
                    connections_tx,
                    authorization::Principal {
                        id: Uuid::new_v4().to_string(),
                        noun: "authority".to_string(),
                        scope: "actor".to_string(),
                    },
                )
            }
        }),
        thread::spawn({
            let connections_rx = connections_rx.clone();
            let connections_tx = connections_tx.clone();
            thread::sleep(Duration::from_secs(5));
            move || {
                run(
                    connections_rx,
                    connections_tx,
                    authorization::Principal {
                        id: Uuid::new_v4().to_string(),
                        noun: "observer".to_string(),
                        scope: "actor".to_string(),
                    },
                )
            }
        }),
    ];

    for thread in threads {
        thread.join().expect("thread terminated unexpectedly");
    }
}

pub fn run(
    connections_rx: Arc<Mutex<mpsc::Receiver<Handshake>>>,
    connections_tx: mpsc::Sender<Handshake>,
    principal: authorization::Principal,
) {
    let mut app = App::new();

    app.add_plugins((
        MinimalPlugins,
        bevy::log::LogPlugin {
            filter: "bevy_app=info,bevy_ecs=info,wgpu=warn".to_string(),
            level: bevy::log::Level::TRACE,
        },
    ));

    if principal.noun == "artificial_intelligence" {
        app.add_plugins(ArtificialIntelligencePlugin);
    }

    if principal.noun == "authority" {
        app.add_plugins(AuthorityPlugin);
    }

    if principal.noun == "replication" {
        app.add_plugins(NetworkServerPlugin)
            .insert_resource(ConnectionsRx(connections_rx));
    } else {
        app.add_plugins(NetworkClientPlugin)
            .insert_resource(ConnectionsTx(connections_tx));
    }

    app.insert_resource(Principal(principal))
        .insert_resource(Identifiers(Default::default()));

    app.add_plugins((MonsterPlugin, PlayerPlugin));

    app.run();
}
