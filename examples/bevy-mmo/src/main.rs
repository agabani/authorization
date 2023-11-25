mod client;
mod replication;

use std::{
    sync::{mpsc, Mutex},
    thread,
};

use bevy::{prelude::*, utils::Uuid};

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

#[derive(Component)]
struct ConnectionTx(mpsc::Sender<Protocol>);

#[derive(Component)]
struct ConnectionRx(Mutex<mpsc::Receiver<Protocol>>);

struct NetworkHandshake {
    principal: authorization::Principal,
    tx: mpsc::Sender<Protocol>,
}

#[derive(Debug)]
enum Protocol {
    Connected(mpsc::Sender<Protocol>),
    Disconnected,
    Ping,
    Pong,
}
