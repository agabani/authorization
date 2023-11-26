mod network;

use std::sync::{mpsc, Mutex};

use bevy::prelude::*;

use crate::network::{ConnectionsRx, Handshake};

use self::network::NetworkPlugin;

pub fn run(connections_rx: mpsc::Receiver<Handshake>) {
    let mut app = App::new();

    app.add_plugins((
        MinimalPlugins,
        bevy::log::LogPlugin {
            filter: "bevy_app=info,bevy_ecs=info,wgpu=warn".to_string(),
            level: bevy::log::Level::TRACE,
        },
    ));

    app.add_plugins(NetworkPlugin)
        .insert_resource(ConnectionsRx(Mutex::new(connections_rx)));

    app.run();
}
