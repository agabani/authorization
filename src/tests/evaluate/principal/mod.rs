mod wildcard;

use std::collections::HashMap;

use crate::{evaluate, Action, Context, Effect, Policy, Principal, Resource};

#[test]
fn should_return_policy() {
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
    let policies = given_policies();

    // Act
    let result = evaluate(&context, &policies);

    // Assert
    assert_eq!(result.unwrap().id, "policy-2");
}

#[test]
fn should_return_none_when_id_not_match() {
    // Arrange
    let context = Context {
        action: given_action(),
        data: HashMap::new(),
        principal: Principal {
            id: "3".to_string(),
            noun: "user".to_string(),
            scope: "first-party".to_string(),
        },
        resource: given_resource(),
    };
    let policies = given_policies();

    // Act
    let result = evaluate(&context, &policies);

    // Assert
    assert_eq!(result, None);
}

#[test]
fn should_none_when_noun_not_match() {
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
    let policies = given_policies();

    // Act
    let result = evaluate(&context, &policies);

    // Assert
    assert_eq!(result, None);
}

#[test]
fn should_none_when_scope_not_match() {
    // Arrange
    let context = Context {
        action: given_action(),
        data: HashMap::new(),
        principal: Principal {
            id: "1".to_string(),
            noun: "user".to_string(),
            scope: "second-party".to_string(),
        },
        resource: given_resource(),
    };
    let policies = given_policies();

    // Act
    let result = evaluate(&context, &policies);

    // Assert
    assert_eq!(result, None);
}

fn given_action() -> Action {
    Action {
        noun: "user".to_string(),
        scope: "identity".to_string(),
        verb: "get".to_string(),
    }
}

fn given_policies() -> [Policy; 2] {
    [
        Policy {
            actions: vec![given_action()],
            conditions: vec![],
            effect: Effect::Allow,
            id: "policy-1".to_string(),
            principals: vec![
                Principal {
                    id: "1".to_string(),
                    noun: "user".to_string(),
                    scope: "first-party".to_string(),
                },
                Principal {
                    id: "1".to_string(),
                    noun: "group".to_string(),
                    scope: "third-party".to_string(),
                },
            ],
            resources: vec![given_resource()],
        },
        Policy {
            actions: vec![given_action()],
            conditions: vec![],
            effect: Effect::Allow,
            id: "policy-2".to_string(),
            principals: vec![
                Principal {
                    id: "2".to_string(),
                    noun: "user".to_string(),
                    scope: "first-party".to_string(),
                },
                Principal {
                    id: "2".to_string(),
                    noun: "group".to_string(),
                    scope: "third-party".to_string(),
                },
            ],
            resources: vec![given_resource()],
        },
    ]
}

fn given_resource() -> Resource {
    Resource {
        id: "1".to_string(),
        noun: "user".to_string(),
        scope: "identity".to_string(),
    }
}
