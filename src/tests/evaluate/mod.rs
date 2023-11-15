mod action;
mod condition;
mod effect;
mod principal;
mod resource;

use std::collections::HashMap;

use crate::{evaluate, Action, Context, Principal, Resource};

#[test]
fn should_return_none_when_no_policies_have_been_provided() {
    // Arrange
    let context = Context {
        action: Action {
            noun: "user".to_string(),
            scope: "identity".to_string(),
            verb: "get".to_string(),
        },
        data: HashMap::new(),
        principal: Principal {
            id: "1".to_string(),
            noun: "user".to_string(),
            scope: "first-party".to_string(),
        },
        resource: Resource {
            id: "1".to_string(),
            noun: "user".to_string(),
            scope: "identity".to_string(),
        },
    };
    let policies = [];

    // Act
    let result = evaluate(&context, &policies);

    // Assert
    assert_eq!(result, None);
}
