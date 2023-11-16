use std::collections::HashMap;

use crate::{evaluate, Action, Context, Effect, Policy, Principal, Resource};

#[test]
fn should_return_allow_policy() {
    // Arrange
    let context = Context {
        action: given_action(),
        data: HashMap::new(),
        principal: given_principal(),
        resource: given_resource(),
    };
    let policies = [given_policy_allow()];

    // Act
    let result = evaluate(&context, &policies);

    // Assert
    assert_eq!(result.unwrap().id, "allow-id");
}

#[test]
fn should_return_deny_policy() {
    // Arrange
    let context = Context {
        action: given_action(),
        data: HashMap::new(),
        principal: given_principal(),
        resource: given_resource(),
    };
    let policies = [given_policy_deny()];

    // Act
    let result = evaluate(&context, &policies);

    // Assert
    assert_eq!(result.unwrap().id, "deny-id");
}

#[test]
fn should_return_deny_policy_when_allow_and_deny_policy_match() {
    // Arrange
    let context = Context {
        action: given_action(),
        data: HashMap::new(),
        principal: given_principal(),
        resource: given_resource(),
    };
    let policies = [given_policy_allow(), given_policy_deny()];

    // Act
    let result = evaluate(&context, &policies);

    // Assert
    assert_eq!(result.unwrap().id, "deny-id");
}

fn given_action() -> Action {
    Action {
        noun: "user".to_string(),
        scope: "identity".to_string(),
        verb: "get".to_string(),
    }
}

fn given_policy_allow() -> Policy {
    Policy {
        actions: vec![given_action()],
        conditions: vec![],
        effect: Effect::Allow,
        id: "allow-id".to_string(),
        principals: vec![given_principal()],
        resources: vec![given_resource()],
    }
}

fn given_policy_deny() -> Policy {
    Policy {
        actions: vec![given_action()],
        conditions: vec![],
        effect: Effect::Deny,
        id: "deny-id".to_string(),
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
