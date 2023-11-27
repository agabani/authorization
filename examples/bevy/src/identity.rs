pub use bevy::prelude::*;

#[derive(Component)]
pub struct Identifier {
    pub id: String,

    pub noun: String,

    pub scope: String,
}

impl From<authorization::Resource> for Identifier {
    fn from(value: authorization::Resource) -> Self {
        Identifier {
            id: value.id,
            noun: value.noun,
            scope: value.scope,
        }
    }
}

#[derive(Component, Resource)]
pub struct Principal(pub authorization::Principal);
