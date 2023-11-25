mod client;
mod replication;

use std::{
    sync::{mpsc, Mutex},
    thread,
};

use bevy::prelude::*;

fn main() {
    let (tx, rx) = mpsc::channel::<Handshake>();

    let tx_1 = tx.clone();
    let tx_2 = tx.clone();

    let handles = [
        thread::spawn(move || replication::app_run(rx)),
        thread::spawn(move || client::app_run(tx_1, client::Role::Ai)),
        thread::spawn(move || client::app_run(tx_2, client::Role::Authority)),
    ];

    for handle in handles {
        handle.join().unwrap();
    }
}

#[derive(Component)]
struct ConnectionTx(mpsc::Sender<Protocol>);

#[derive(Component)]
struct ConnectionRx(Mutex<mpsc::Receiver<Protocol>>);

struct Handshake {
    identity: (),
    tx: mpsc::Sender<Protocol>,
}

#[derive(Debug)]
enum Protocol {
    Connected(mpsc::Sender<Protocol>),
    Ping,
    Pong,
}
