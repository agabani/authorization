use authorization::{Action, Context, Effect, Policy, Principal, Resource};
use bevy::prelude::*;

use crate::{
    Audit, AuthorizationEventPlugin, AuthorizationPlugin, Authorized, Database,
    IntoUnauthorizedContext, Unauthorized,
};

#[test]
fn authorize_should_send_event_when_explicit_allow() {
    // Arrange
    let mut app = App::new();

    app.add_plugins((
        MinimalPlugins,
        AuthorizationPlugin,
        AuthorizationEventPlugin::<TestDatabase, TestEvent>::default(),
    ))
    .insert_resource(TestDatabase {
        policy: Some(test_policy(Effect::Allow)),
    });

    let entity = app.world.spawn(()).id();

    app.world.send_event(Unauthorized {
        actor: entity,
        data: TestEvent,
    });

    // Act
    app.update();

    // Assert
    let events = app
        .world
        .get_resource_mut::<Events<Authorized<TestEvent>>>()
        .unwrap();
    let mut reader = events.get_reader();
    let mut test_events = reader.read(&events);
    let test_event = test_events.next().unwrap();

    assert_eq!(test_event.actor, entity);
}

#[test]
fn authorize_should_not_send_event_when_explicit_deny() {
    // Arrange
    let mut app = App::new();

    app.add_plugins((
        MinimalPlugins,
        AuthorizationPlugin,
        AuthorizationEventPlugin::<TestDatabase, TestEvent>::default(),
    ))
    .insert_resource(TestDatabase {
        policy: Some(test_policy(Effect::Deny)),
    });

    let entity = app.world.spawn(()).id();

    app.world.send_event(Unauthorized {
        actor: entity,
        data: TestEvent,
    });

    // Act
    app.update();

    // Assert
    let events = app
        .world
        .get_resource_mut::<Events<Authorized<TestEvent>>>()
        .unwrap();
    let mut reader = events.get_reader();
    let mut test_events = reader.read(&events);
    let test_event = test_events.next();

    assert!(test_event.is_none());
}

#[test]
fn authorize_should_not_send_event_when_implicit_deny() {
    // Arrange
    let mut app = App::new();

    app.add_plugins((
        MinimalPlugins,
        AuthorizationPlugin,
        AuthorizationEventPlugin::<TestDatabase, TestEvent>::default(),
    ))
    .insert_resource(TestDatabase { policy: None });

    let entity = app.world.spawn(()).id();

    app.world.send_event(Unauthorized {
        actor: entity,
        data: TestEvent,
    });

    // Act
    app.update();

    // Assert
    let events = app
        .world
        .get_resource_mut::<Events<Authorized<TestEvent>>>()
        .unwrap();
    let mut reader = events.get_reader();
    let mut test_events = reader.read(&events);
    let test_event = test_events.next();

    assert!(test_event.is_none());
}

#[test]
fn authorize_should_send_audit_when_explicit_allow() {
    // Arrange
    let mut app = App::new();

    app.add_plugins((
        MinimalPlugins,
        AuthorizationPlugin,
        AuthorizationEventPlugin::<TestDatabase, TestEvent>::default(),
    ))
    .insert_resource(TestDatabase {
        policy: Some(test_policy(Effect::Allow)),
    });

    let entity = app.world.spawn(()).id();

    app.world.send_event(Unauthorized {
        actor: entity,
        data: TestEvent,
    });

    // Act
    app.update();

    // Assert
    let events = app.world.get_resource_mut::<Events<Audit>>().unwrap();
    let mut reader = events.get_reader();
    let mut audits = reader.read(&events);
    let audit = audits.next().unwrap();

    assert_eq!(audit.context, test_context());
    assert_eq!(audit.policy, Some(test_policy(Effect::Allow)));
}

#[test]
fn authorize_should_not_send_audit_when_explicit_deny() {
    // Arrange
    let mut app = App::new();

    app.add_plugins((
        MinimalPlugins,
        AuthorizationPlugin,
        AuthorizationEventPlugin::<TestDatabase, TestEvent>::default(),
    ))
    .insert_resource(TestDatabase {
        policy: Some(test_policy(Effect::Deny)),
    });

    let entity = app.world.spawn(()).id();

    app.world.send_event(Unauthorized {
        actor: entity,
        data: TestEvent,
    });

    // Act
    app.update();

    // Assert
    let events = app.world.get_resource_mut::<Events<Audit>>().unwrap();
    let mut reader = events.get_reader();
    let mut audits = reader.read(&events);
    let audit = audits.next().unwrap();

    assert_eq!(audit.context, test_context());
    assert_eq!(audit.policy, Some(test_policy(Effect::Deny)));
}

#[test]
fn authorize_should_not_send_audit_when_implicit_deny() {
    // Arrange
    let mut app = App::new();

    app.add_plugins((
        MinimalPlugins,
        AuthorizationPlugin,
        AuthorizationEventPlugin::<TestDatabase, TestEvent>::default(),
    ))
    .insert_resource(TestDatabase { policy: None });

    let entity = app.world.spawn(()).id();

    app.world.send_event(Unauthorized {
        actor: entity,
        data: TestEvent,
    });

    // Act
    app.update();

    // Assert
    let events = app.world.get_resource_mut::<Events<Audit>>().unwrap();
    let mut reader = events.get_reader();
    let mut audits = reader.read(&events);
    let audit = audits.next().unwrap();

    assert_eq!(audit.context, test_context());
    assert_eq!(audit.policy, None);
}

#[derive(Resource)]
struct TestDatabase {
    policy: Option<Policy>,
}

impl Database for TestDatabase {
    fn query_by_principal(
        &self,
        principal: &authorization::Principal,
    ) -> Vec<authorization::Policy> {
        assert_eq!(principal.id, "id");
        assert_eq!(principal.noun, "noun");
        assert_eq!(principal.scope, "scope");

        if let Some(policy) = &self.policy {
            vec![policy.clone()]
        } else {
            vec![]
        }
    }
}

#[derive(Clone)]
struct TestEvent;

impl IntoUnauthorizedContext for TestEvent {
    fn into_unauthorized_context(
        event: &crate::Unauthorized<Self>,
        _query: &Query<&crate::Identifier>,
    ) -> Option<Context> {
        assert_eq!(event.actor.index(), 0);
        Some(test_context())
    }
}

fn test_context() -> Context {
    Context {
        action: Action {
            noun: "noun".to_string(),
            scope: "scope".to_string(),
            verb: "verb".to_string(),
        },
        data: Default::default(),
        principal: Principal {
            id: "id".to_string(),
            noun: "noun".to_string(),
            scope: "scope".to_string(),
        },
        resource: Resource {
            id: "id".to_string(),
            noun: "noun".to_string(),
            scope: "scope".to_string(),
        },
    }
}

fn test_policy(effect: Effect) -> Policy {
    Policy {
        actions: vec![Action {
            noun: "noun".to_string(),
            scope: "scope".to_string(),
            verb: "verb".to_string(),
        }],
        conditions: Default::default(),
        effect,
        id: "id".to_string(),
        principals: vec![Principal {
            id: "id".to_string(),
            noun: "noun".to_string(),
            scope: "scope".to_string(),
        }],
        resources: vec![Resource {
            id: "id".to_string(),
            noun: "noun".to_string(),
            scope: "scope".to_string(),
        }],
    }
}
