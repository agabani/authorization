use bevy::{prelude::*, utils::HashMap};

#[derive(Clone, Component, PartialEq, Eq, Hash)]
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

impl From<Identifier> for authorization::Resource {
    fn from(value: Identifier) -> Self {
        authorization::Resource {
            id: value.id,
            noun: value.noun,
            scope: value.scope,
        }
    }
}

#[derive(Component, Resource)]
pub struct Principal(pub authorization::Principal);

#[derive(Resource)]
pub struct Identifiers(pub HashMap<Identifier, Entity>);
