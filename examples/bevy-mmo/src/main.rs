mod client;
mod core;
mod replication;

use std::{sync::mpsc, thread};

use bevy::utils::Uuid;

fn main() {
    let (tx, rx) = mpsc::channel();

    let handles = [
        thread::spawn(move || replication::app_run(rx)),
        {
            let tx = tx.clone();
            thread::spawn(move || {
                client::app_run(
                    tx,
                    authorization::Principal {
                        id: Uuid::new_v4().to_string(),
                        noun: "ai".to_string(),
                        scope: "actor".to_string(),
                    },
                )
            })
        },
        thread::spawn({
            let tx = tx.clone();
            move || {
                client::app_run(
                    tx,
                    authorization::Principal {
                        id: Uuid::new_v4().to_string(),
                        noun: "authority".to_string(),
                        scope: "actor".to_string(),
                    },
                )
            }
        }),
    ];

    for handle in handles {
        handle.join().unwrap();
    }
}
