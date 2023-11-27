use std::{sync::mpsc, thread};

use bevy::utils::Uuid;

mod client;
mod identity;
mod network;
mod player;
mod server;

fn main() {
    let (connections_tx, connections_rx) = mpsc::channel();

    let threads = [
        thread::spawn({
            let connections_rx = connections_rx;
            move || {
                server::run(
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
                client::run(
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
                client::run(
                    connections_tx,
                    authorization::Principal {
                        id: Uuid::new_v4().to_string(),
                        noun: "authority".to_string(),
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
