use std::collections::{HashMap, HashSet};

use crate::{evaluate, Action, Condition, Context, Effect, Policy, Principal, Resource};

#[test]
fn should_return_policy() {
    // Arrange
    let context = Context {
        action: given_action(),
        data: HashMap::from([
            (
                "geography:city".to_string(),
                HashSet::from(["atlantis".to_string(), "olympus".to_string()]),
            ),
            (
                "geography:planet".to_string(),
                HashSet::from(["earth".to_string(), "mars".to_string()]),
            ),
        ]),
        principal: given_principal(),
        resource: given_resource(),
    };
    let policies = [
        given_policy(),
        Policy {
            actions: vec![given_action()],
            conditions: vec![Condition {
                string_equals: Some(HashMap::from([
                    (
                        "geography:city".to_string(),
                        HashSet::from(["atlantis".to_string(), "olympus".to_string()]),
                    ),
                    (
                        "geography:planet".to_string(),
                        HashSet::from([
                            "earth".to_string(),
                            "mars".to_string(),
                            "venus".to_string(),
                        ]),
                    ),
                ])),
                ..Default::default()
            }],
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
fn should_return_none_when_not_subset() {
    // Arrange
    let context = Context {
        action: given_action(),
        data: HashMap::from([
            (
                "geography:city".to_string(),
                HashSet::from(["atlantis".to_string(), "olympus".to_string()]),
            ),
            (
                "geography:planet".to_string(),
                HashSet::from(["earth".to_string(), "mars".to_string(), "venus".to_string()]),
            ),
        ]),
        principal: given_principal(),
        resource: given_resource(),
    };
    let policies = [
        given_policy(),
        Policy {
            actions: vec![given_action()],
            conditions: vec![Condition {
                string_equals: Some(HashMap::from([
                    (
                        "geography:city".to_string(),
                        HashSet::from(["atlantis".to_string(), "olympus".to_string()]),
                    ),
                    (
                        "geography:planet".to_string(),
                        HashSet::from(["earth".to_string(), "mars".to_string()]),
                    ),
                ])),
                ..Default::default()
            }],
            effect: Effect::Allow,
            id: "policy-2".to_string(),
            principals: vec![given_principal()],
            resources: vec![given_resource()],
        },
    ];

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

fn given_policy() -> Policy {
    Policy {
        actions: vec![given_action()],
        conditions: vec![Condition {
            string_equals: Some(HashMap::from([(
                "geography:country".to_string(),
                HashSet::from(["jotunheim".to_string()]),
            )])),
            ..Default::default()
        }],
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
