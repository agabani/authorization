mod network;

use std::sync::mpsc;

use bevy::prelude::*;

use crate::{
    artificial_intelligence::ArtificialIntelligencePlugin,
    authority::AuthorityPlugin,
    identity::{Identifiers, Principal},
    network::{ConnectionsTx, Handshake},
    player::PlayerPlugin,
};

use self::network::NetworkPlugin;

pub fn run(connections_tx: mpsc::Sender<Handshake>, principal: authorization::Principal) {
    let mut app = App::new();

    app.add_plugins((
        MinimalPlugins,
        bevy::log::LogPlugin {
            filter: "bevy_app=info,bevy_ecs=info,wgpu=warn".to_string(),
            level: bevy::log::Level::TRACE,
        },
    ));

    if principal.noun == "artificial_intelligence" {
        app.add_plugins(ArtificialIntelligencePlugin);
    }

    if principal.noun == "authority" {
        app.add_plugins(AuthorityPlugin);
    }

    app.add_plugins(NetworkPlugin)
        .insert_resource(ConnectionsTx(connections_tx))
        .insert_resource(Principal(principal));

    app.insert_resource(Identifiers(Default::default()));

    app.add_plugins(PlayerPlugin);

    app.run();
}
