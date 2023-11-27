use std::sync::{mpsc, Mutex};

use bevy::prelude::*;

use crate::{
    identity::{Identifiers, Principal},
    network::{ConnectionsRx, Handshake},
    network_server::NetworkServerPlugin,
    player::PlayerPlugin,
};

pub fn run(connections_rx: mpsc::Receiver<Handshake>, principal: authorization::Principal) {
    let mut app = App::new();

    app.add_plugins((
        MinimalPlugins,
        bevy::log::LogPlugin {
            filter: "bevy_app=info,bevy_ecs=info,wgpu=warn".to_string(),
            level: bevy::log::Level::TRACE,
        },
    ));

    app.add_plugins(NetworkServerPlugin)
        .insert_resource(ConnectionsRx(Mutex::new(connections_rx)))
        .insert_resource(Principal(principal));

    app.insert_resource(Identifiers(Default::default()));

    app.add_plugins(PlayerPlugin);

    app.run();
}
