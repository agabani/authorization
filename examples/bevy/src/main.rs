mod artificial_intelligence;
mod authority;
mod identity;
mod network;
mod network_client;
mod network_server;
mod player;

use std::{
    sync::{mpsc, Mutex},
    thread,
    time::Duration,
};

use artificial_intelligence::ArtificialIntelligencePlugin;
use authority::AuthorityPlugin;
use bevy::{prelude::*, utils::Uuid};
use identity::{Identifiers, Principal};
use network::{ConnectionsRx, ConnectionsTx, Handshake};
use network_client::NetworkClientPlugin;
use network_server::NetworkServerPlugin;
use player::PlayerPlugin;

fn main() {
    let (connections_tx, connections_rx) = mpsc::channel();

    let threads = [
        thread::spawn({
            let connections_rx = connections_rx;
            move || {
                server(
                    connections_rx,
                    authorization::Principal {
                        id: Uuid::new_v4().to_string(),
                        noun: "replication".to_string(),
                        scope: "actor".to_string(),
                    },
                )
            }
        }),
        thread::spawn({
            let connections_tx = connections_tx.clone();
            move || {
                client(
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
            let connections_tx = connections_tx.clone();
            move || {
                client(
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
            let connections_tx = connections_tx.clone();
            thread::sleep(Duration::from_secs(5));
            move || {
                client(
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

pub fn client(connections_tx: mpsc::Sender<Handshake>, principal: authorization::Principal) {
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

    app.add_plugins(NetworkClientPlugin)
        .insert_resource(ConnectionsTx(connections_tx))
        .insert_resource(Principal(principal));

    app.insert_resource(Identifiers(Default::default()));

    app.add_plugins(PlayerPlugin);

    app.run();
}

pub fn server(connections_rx: mpsc::Receiver<Handshake>, principal: authorization::Principal) {
    let mut app = App::new();

    app.add_plugins((
        MinimalPlugins,
        bevy::log::LogPlugin {
            filter: "bevy_app=info,bevy_ecs=info,wgpu=warn".to_string(),
            level: bevy::log::Level::TRACE,
        },
    ));

    app.add_plugins(NetworkServerPlugin)
        .insert_resource(ConnectionsRx(Mutex::new(connections_rx)))
        .insert_resource(Principal(principal));

    app.insert_resource(Identifiers(Default::default()));

    app.add_plugins(PlayerPlugin);

    app.run();
}
