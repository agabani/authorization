use std::sync::{mpsc, Mutex};

use bevy::{log::LogPlugin, prelude::*};

use crate::{ConnectionRx, ConnectionTx, Handshake, Protocol};

pub fn app_run(rx: mpsc::Receiver<Handshake>) {
    let mut app = App::new();

    app.add_plugins((
        MinimalPlugins,
        LogPlugin {
            ..Default::default()
        },
    ))
    .insert_resource(NetworkListener(Mutex::new(rx)))
    .add_systems(Update, (listen, protocol));

    app.run();
}

#[derive(Resource)]
struct NetworkListener(Mutex<mpsc::Receiver<Handshake>>);

fn listen(mut commands: Commands, listener: Res<NetworkListener>) {
    listener
        .0
        .lock()
        .expect("poisoned")
        .try_iter()
        .for_each(|handshake| {
            info!("handshake received");

            let (tx, rx) = mpsc::channel();
            handshake.tx.send(Protocol::Connected(tx)).unwrap();

            let connection_rx = ConnectionRx(Mutex::new(rx));
            let connection_tx = ConnectionTx(handshake.tx);
            commands.spawn((connection_rx, connection_tx));
        });
}

fn protocol(query: Query<(&ConnectionTx, &ConnectionRx)>) {
    query.for_each(|(tx, rx)| {
        rx.0.lock()
            .expect("poisoned")
            .try_iter()
            .for_each(|protocol| {
                info!("protocol received {protocol:?}");
                match protocol {
                    Protocol::Connected(_) => todo!(),
                    Protocol::Ping => {
                        tx.0.send(Protocol::Pong).unwrap();
                    }
                    Protocol::Pong => todo!(),
                }
            });
    });
}
