use std::{
    sync::{mpsc, Mutex},
    time::Duration,
};

use bevy::{log::LogPlugin, prelude::*};

use crate::{ConnectionRx, ConnectionTx, Handshake, Protocol};

pub fn app_run(tx: mpsc::Sender<Handshake>, role: Role) {
    let mut app = App::new();

    app.add_plugins((
        MinimalPlugins,
        LogPlugin {
            ..Default::default()
        },
    ))
    .insert_resource(NetworkConnector(tx))
    .add_systems(Startup, connect)
    .add_systems(Update, (connected, keep_alive, protocol));

    app.run();
}

#[derive(Resource)]
struct NetworkConnector(mpsc::Sender<Handshake>);

#[derive(Debug, Resource)]
pub enum Role {
    Ai,
    Authority,
}

fn connect(mut commands: Commands, connector: Res<NetworkConnector>) {
    let (tx, rx) = mpsc::channel();
    connector.0.send(Handshake { identity: (), tx }).unwrap();
    commands.spawn(ConnectionRx(Mutex::new(rx)));
}

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

fn keep_alive(time: Res<Time>, mut query: Query<(&mut KeepAliveTimer, &ConnectionTx)>) {
    query.for_each_mut(|(mut timer, tx)| {
        if timer.0.tick(time.delta()).just_finished() {
            tx.0.send(Protocol::Ping).unwrap();
            timer.0.reset();
        }
    });
}

fn protocol(query: Query<(&ConnectionTx, &ConnectionRx)>) {
    query.for_each(|(_, rx)| {
        rx.0.lock()
            .expect("poisoned")
            .try_iter()
            .for_each(|protocol| {
                info!("protocol received {protocol:?}");
            });
    });
}
