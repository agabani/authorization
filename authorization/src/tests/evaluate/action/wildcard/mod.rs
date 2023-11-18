use std::collections::HashMap;

use crate::{evaluate, Action, Context, Effect, Policy, Principal, Resource};

#[test]
fn should_return_policy_when_noun() {
    // Arrange
    let context = Context {
        action: Action {
            noun: "group".to_string(),
            scope: "identity".to_string(),
            verb: "get".to_string(),
        },
        data: HashMap::new(),
        principal: given_principal(),
        resource: given_resource(),
    };
    let policies = [
        given_policy(),
        Policy {
            actions: vec![Action {
                noun: "*".to_string(),
                scope: "identity".to_string(),
                verb: "get".to_string(),
            }],
            conditions: vec![],
            effect: Effect::Allow,
            id: "policy-2".to_string(),
            principals: vec![given_principal()],
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
        action: Action {
            noun: "user".to_string(),
            scope: "credential".to_string(),
            verb: "get".to_string(),
        },
        data: HashMap::new(),
        principal: given_principal(),
        resource: given_resource(),
    };
    let policies = [
        given_policy(),
        Policy {
            actions: vec![Action {
                noun: "user".to_string(),
                scope: "*".to_string(),
                verb: "get".to_string(),
            }],
            conditions: vec![],
            effect: Effect::Allow,
            id: "policy-2".to_string(),
            principals: vec![given_principal()],
            resources: vec![given_resource()],
        },
    ];

    // Act
    let result = evaluate(&context, &policies);

    // Assert
    assert_eq!(result.unwrap().id, "policy-2");
}

#[test]
fn should_return_policy_when_verb() {
    // Arrange
    let context = Context {
        action: Action {
            noun: "user".to_string(),
            scope: "identity".to_string(),
            verb: "delete".to_string(),
        },
        data: HashMap::new(),
        principal: given_principal(),
        resource: given_resource(),
    };
    let policies = [
        given_policy(),
        Policy {
            actions: vec![Action {
                noun: "user".to_string(),
                scope: "identity".to_string(),
                verb: "*".to_string(),
            }],
            conditions: vec![],
            effect: Effect::Allow,
            id: "policy-2".to_string(),
            principals: vec![given_principal()],
            resources: vec![given_resource()],
        },
    ];

    // Act
    let result = evaluate(&context, &policies);

    // Assert
    assert_eq!(result.unwrap().id, "policy-2");
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
        principals: vec![given_principal()],
        resources: vec![given_resource()],
    }
}

fn given_principal() -> Principal {
    Principal {
        id: "1".to_string(),
        noun: "user".to_string(),
        scope: "first-party".to_string(),
    }
}

fn given_resource() -> Resource {
    Resource {
        id: "1".to_string(),
        noun: "user".to_string(),
        scope: "identity".to_string(),
    }
}
