use std::sync::{
    mpsc::{self, TryRecvError},
    Mutex,
};

use bevy::prelude::*;

use crate::{
    identity::Principal,
    network::{send, ConnectionRx, ConnectionTx, ConnectionsTx, Handshake, Protocol, Request},
};

pub struct NetworkClientPlugin;

impl Plugin for NetworkClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                initiate_connection,
                initialize_connection,
                keep_connection_alive,
                read_connection,
                replicate,
            ),
        );
    }
}

/*
 * ============================================================================
 * Create connections
 * ============================================================================
 */

fn initiate_connection(
    mut commands: Commands,
    connections: Res<ConnectionsTx>,
    principal: Res<Principal>,
    query: Query<&ConnectionRx>,
) {
    if query.is_empty() {
        let (tx, rx) = mpsc::channel();
        connections
            .0
            .send(Handshake {
                principal: principal.0.clone(),
                tx,
            })
            .expect("connections closed");
        commands.spawn(ConnectionRx(Mutex::new(rx)));
    }
}

fn initialize_connection(
    mut commands: Commands,
    principal: Res<Principal>,
    query: Query<(Entity, &ConnectionRx), Without<ConnectionTx>>,
) {
    query.for_each(
        |(entity, rx)| match rx.0.lock().expect("poisoned").try_recv() {
            Ok(Protocol::Connected(tx)) => {
                commands
                    .entity(entity)
                    .insert(ConnectionTx(tx))
                    .insert(KeepAlive(Timer::from_seconds(1.0, TimerMode::Repeating)));
                info!("connected {:?}", principal.0);
            }
            Ok(_) => {
                panic!("unexpected packet");
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                commands.entity(entity).despawn();
                warn!("disconnected");
            }
            Err(mpsc::TryRecvError::Empty) => {}
        },
    );
}

/*
 * ============================================================================
 * Read connection
 * ============================================================================
 */

fn read_connection(
    mut commands: Commands,
    principal: Res<Principal>,
    query: Query<(Entity, &ConnectionRx), With<ConnectionTx>>,
) {
    query.for_each(|(entity, rx)| {
        let rx = rx.0.lock().expect("poisoned");
        loop {
            match rx.try_recv() {
                Ok(protocol) => match protocol {
                    Protocol::Connected(_) => panic!("unexpected packet"),
                    Protocol::Disconnect => {
                        commands.entity(entity).despawn();
                        warn!("disconnected");
                    }
                    Protocol::Ping => panic!("unexpected packet"),
                    Protocol::Pong => {}
                    Protocol::Request(context, tx) => {
                        if principal.0.noun != "authority" {
                            panic!("unexpected packet");
                        }
                        commands.spawn(Request { context, tx });
                    }
                    Protocol::Broadcast(event) => event.spawn_broadcast(&mut commands),
                    Protocol::Replicate => panic!("unexpected packet"),
                },
                Err(error) => {
                    match error {
                        TryRecvError::Empty => {}
                        TryRecvError::Disconnected => {
                            commands.entity(entity).despawn();
                            warn!("disconnected");
                        }
                    }
                    return;
                }
            }
        }
    });
}

/*
 * ============================================================================
 * Keep alive connection
 * ============================================================================
 */

#[derive(Component)]
struct KeepAlive(Timer);

fn keep_connection_alive(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &ConnectionTx, &mut KeepAlive)>,
) {
    query.for_each_mut(|(entity, tx, mut keep_alive)| {
        if keep_alive.0.tick(time.delta()).finished() {
            let protocol = Protocol::Ping;
            send(&mut commands, entity, tx, protocol);
        }
    });
}

/*
 * ============================================================================
 * Replicate
 * ============================================================================
 */

fn replicate(mut commands: Commands, query: Query<(Entity, &ConnectionTx), Added<ConnectionTx>>) {
    query.for_each(|(entity, tx)| {
        let protocol = Protocol::Replicate;
        send(&mut commands, entity, tx, protocol);
    });
}
