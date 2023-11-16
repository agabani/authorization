use std::collections::HashMap;

use crate::{evaluate, Action, Context, Effect, Policy, Principal, Resource};

#[test]
fn should_return_policy_when_id() {
    // Arrange
    let context = Context {
        action: given_action(),
        data: HashMap::new(),
        principal: Principal {
            id: "2".to_string(),
            noun: "user".to_string(),
            scope: "first-party".to_string(),
        },
        resource: given_resource(),
    };
    let policies = [
        given_policy(),
        Policy {
            actions: vec![given_action()],
            conditions: vec![],
            effect: Effect::Allow,
            id: "policy-2".to_string(),
            principals: vec![Principal {
                id: "*".to_string(),
                noun: "user".to_string(),
                scope: "first-party".to_string(),
            }],
            resources: vec![given_resource()],
        },
    ];

    // Act
    let result = evaluate(&context, &policies);

    // Assert
    assert_eq!(result.unwrap().id, "policy-2");
}

#[test]
fn should_return_policy_when_noun() {
    // Arrange
    let context = Context {
        action: given_action(),
        data: HashMap::new(),
        principal: Principal {
            id: "1".to_string(),
            noun: "role".to_string(),
            scope: "first-party".to_string(),
        },
        resource: given_resource(),
    };
    let policies = [
        given_policy(),
        Policy {
            actions: vec![given_action()],
            conditions: vec![],
            effect: Effect::Allow,
            id: "policy-2".to_string(),
            principals: vec![Principal {
                id: "1".to_string(),
                noun: "*".to_string(),
                scope: "first-party".to_string(),
            }],
            resources: vec![given_resource()],
        },
    ];

    // Act
    let result = evaluate(&context, &policies);

    // Assert
    assert_eq!(result.unwrap().id, "policy-2");
}

#[test]
fn should_return_policy_when_scope() {
    // Arrange
    let context = Context {
        action: given_action(),
        data: HashMap::new(),
        principal: Principal {
            id: "1".to_string(),
            noun: "role".to_string(),
            scope: "third-party".to_string(),
        },
        resource: given_resource(),
    };
    let policies = [
        given_policy(),
        Policy {
            actions: vec![given_action()],
            conditions: vec![],
            effect: Effect::Allow,
            id: "policy-2".to_string(),
            principals: vec![Principal {
                id: "1".to_string(),
                noun: "role".to_string(),
                scope: "*".to_string(),
            }],
            resources: vec![given_resource()],
        },
    ];

    // Act
    let result = evaluate(&context, &policies);

    // Assert
    assert_eq!(result.unwrap().id, "policy-2");
}

fn given_action() -> Action {
    Action {
        noun: "user".to_string(),
        scope: "identity".to_string(),
        verb: "get".to_string(),
    }
}

fn given_policy() -> Policy {
    Policy {
        actions: vec![Action {
            noun: "user".to_string(),
            scope: "identity".to_string(),
            verb: "get".to_string(),
        }],
        conditions: vec![],
        effect: Effect::Allow,
        id: "policy-1".to_string(),
        principals: vec![Principal {
            id: "1".to_string(),
            noun: "user".to_string(),
            scope: "first-party".to_string(),
        }],
        resources: vec![given_resource()],
    }
}

fn given_resource() -> Resource {
    Resource {
        id: "1".to_string(),
        noun: "user".to_string(),
        scope: "identity".to_string(),
    }
}
