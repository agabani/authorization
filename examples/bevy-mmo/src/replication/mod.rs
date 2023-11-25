use std::sync::{mpsc, Mutex};

use bevy::{log::LogPlugin, prelude::*};

use crate::core::{ConnectionRx, ConnectionTx, NetworkHandshake, Protocol};

pub fn app_run(rx: mpsc::Receiver<NetworkHandshake>) {
    let mut app = App::new();

    app.add_plugins((
        MinimalPlugins,
        LogPlugin {
            ..Default::default()
        },
    ))
    .insert_resource(NetworkListener(Mutex::new(rx)))
    .add_systems(Update, (listen, protocol, timeout));

    app.run();
}

#[derive(Component)]
struct KeepAliveTimeout(Timer);

#[derive(Resource)]
struct NetworkListener(Mutex<mpsc::Receiver<NetworkHandshake>>);

#[derive(Component)]
struct Principal(authorization::Principal);

/// Accepts new connections from client.
fn listen(mut commands: Commands, listener: Res<NetworkListener>) {
    listener
        .0
        .lock()
        .expect("poisoned")
        .try_iter()
        .for_each(|handshake| {
            info!("handshake received\n  principal: {:?}", handshake.principal);

            let (tx, rx) = mpsc::channel();
            handshake.tx.send(Protocol::Connected(tx)).unwrap();

            commands.spawn((
                ConnectionRx(Mutex::new(rx)),
                ConnectionTx(handshake.tx),
                KeepAliveTimeout(Timer::from_seconds(20.0, TimerMode::Once)),
                Principal(handshake.principal),
            ));
        });
}

fn protocol(mut query: Query<(&ConnectionRx, &ConnectionTx, &mut KeepAliveTimeout)>) {
    query.for_each_mut(|(rx, tx, mut timeout)| {
        rx.0.lock()
            .expect("poisoned")
            .try_iter()
            .for_each(|protocol| {
                timeout.0.reset();
                match protocol {
                    Protocol::Connected(_) => {
                        todo!("did not expect to receive this from the client");
                    }
                    Protocol::Ping => {
                        tx.0.send(Protocol::Pong).unwrap();
                    }
                    Protocol::Pong => todo!(),
                    Protocol::Disconnected => todo!(),
                    Protocol::Authorize(_context, _tx) => todo!(),
                }
            });
    });
}

/// Disconnects and removes client on timeout.
fn timeout(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &ConnectionTx, &mut KeepAliveTimeout, &Principal)>,
) {
    query.for_each_mut(|(entity, tx, mut timeout, principal)| {
        if timeout.0.tick(time.delta()).finished() {
            commands.entity(entity).despawn();
            let disconnected = tx.0.send(Protocol::Disconnected);
            warn!(
                "timeout\n  principal: {:?}\n  disconnected: {:?}",
                principal.0, disconnected
            );
        }
    });
}
