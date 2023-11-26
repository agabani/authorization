mod network;

use std::sync::mpsc;

use bevy::prelude::*;

use crate::network::{ConnectionsTx, Handshake};

use self::network::NetworkPlugin;

pub fn run(connections_tx: mpsc::Sender<Handshake>) {
    let mut app = App::new();

    app.add_plugins((
        MinimalPlugins,
        bevy::log::LogPlugin {
            filter: "bevy_app=info,bevy_ecs=info,wgpu=warn".to_string(),
            level: bevy::log::Level::TRACE,
        },
    ));

    app.add_plugins(NetworkPlugin)
        .insert_resource(ConnectionsTx(connections_tx));

    app.run();
}
