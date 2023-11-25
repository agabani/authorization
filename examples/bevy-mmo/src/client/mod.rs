use std::{
    sync::{mpsc, Mutex},
    time::Duration,
};

use bevy::{log::LogPlugin, prelude::*};

use crate::core::{ConnectionRx, ConnectionTx, NetworkHandshake, Protocol};

pub fn app_run(tx: mpsc::Sender<NetworkHandshake>, principal: authorization::Principal) {
    let mut app = App::new();

    app.add_plugins((
        MinimalPlugins,
        LogPlugin {
            ..Default::default()
        },
    ))
    .insert_resource(NetworkConnector(tx))
    .insert_resource(Principal(principal))
    .add_systems(Update, (connect, connected, keep_alive, protocol));

    app.run();
}

#[derive(Resource)]

struct Principal(authorization::Principal);

#[derive(Resource)]
struct NetworkConnector(mpsc::Sender<NetworkHandshake>);

/// Initiate connection to replication.
fn connect(
    mut commands: Commands,
    connector: Res<NetworkConnector>,
    principal: Res<Principal>,
    query: Query<(), With<ConnectionRx>>,
) {
    if query.iter().count() == 0 {
        let (tx, rx) = mpsc::channel();
        connector
            .0
            .send(NetworkHandshake {
                principal: principal.0.clone(),
                tx,
            })
            .unwrap();
        commands.spawn(ConnectionRx(Mutex::new(rx)));
    }
}

/// Complete connection to replication.
fn connected(mut commands: Commands, query: Query<(Entity, &ConnectionRx), Without<ConnectionTx>>) {
    query.for_each(|(entity, rx)| {
        match rx.0.lock().expect("poisoned").try_recv() {
            Ok(protocol) => match protocol {
                Protocol::Connected(tx) => {
                    info!("connected");
                    commands
                        .entity(entity)
                        .insert(ConnectionTx(tx))
                        .insert(KeepAliveTimer(Timer::new(
                            Duration::from_secs(10),
                            TimerMode::Repeating,
                        )));
                }
                _ => {
                    todo!("unexpected packet");
                }
            },
            Err(mpsc::TryRecvError::Empty) => {}
            Err(mpsc::TryRecvError::Disconnected) => {
                todo!("connection disconnected");
            }
        };
    });
}

#[derive(Component)]
struct KeepAliveTimer(Timer);

/// Periodically send keep alive packet.
fn keep_alive(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &ConnectionTx, &mut KeepAliveTimer)>,
) {
    query.for_each_mut(|(entity, tx, mut timer)| {
        if timer.0.tick(time.delta()).finished() {
            timer.0.reset();
            if let Err(error) = tx.0.send(Protocol::Ping) {
                commands.entity(entity).despawn();
                warn!("disconnected\n  error: {error:?}");
            }
        }
    });
}

fn protocol(query: Query<(&ConnectionTx, &ConnectionRx)>) {
    query.for_each(|(_, rx)| {
        rx.0.lock()
            .expect("poisoned")
            .try_iter()
            .for_each(|protocol| {
                trace!("protocol received {protocol:?}");
            });
    });
}
