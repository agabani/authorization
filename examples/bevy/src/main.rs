use std::{sync::mpsc, thread};

mod client;
mod network;
mod server;

fn main() {
    let (connections_tx, connections_rx) = mpsc::channel();

    let threads = [
        thread::spawn({
            let connections_rx = connections_rx;
            move || server::run(connections_rx)
        }),
        thread::spawn({
            let connections_tx = connections_tx.clone();
            move || client::run(connections_tx)
        }),
        thread::spawn({
            let connections_tx = connections_tx.clone();
            move || client::run(connections_tx)
        }),
    ];

    for thread in threads {
        thread.join().expect("thread terminated unexpectedly");
    }
}
