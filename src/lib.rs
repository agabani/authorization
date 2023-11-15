#![deny(clippy::pedantic, missing_docs)]
#![forbid(unsafe_code)]

//! Authorization.

#[cfg(test)]
mod tests;

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

/// Action.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Action {
    /// Noun.
    #[serde(rename = "noun")]
    pub noun: String,

    /// Scope.
    #[serde(rename = "scope")]
    pub scope: String,

    /// Verb.
    #[serde(rename = "verb")]
    pub verb: String,
}

/// Condition.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct Condition {
    /// String Equals.
    #[serde(rename = "string_equals")]
    pub string_equals: Option<HashMap<String, HashSet<String>>>,
}

/// Context.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Context {
    /// Action.
    #[serde(rename = "action")]
    pub action: Action,

    /// Data.
    #[serde(rename = "data")]
    pub data: HashMap<String, HashSet<String>>,

    /// Principal.
    #[serde(rename = "principal")]
    pub principal: Principal,

    /// Resource.
    #[serde(rename = "resource")]
    pub resource: Resource,
}

/// Effect.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum Effect {
    /// Allow.
    #[serde(rename = "allow")]
    Allow,

    /// Deny.
    #[serde(rename = "deny")]
    Deny,
}

/// Policy.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Policy {
    /// Actions.
    #[serde(rename = "actions")]
    pub actions: Vec<Action>,

    /// Conditions.
    #[serde(rename = "conditions")]
    pub conditions: Vec<Condition>,

    /// Effect.
    #[serde(rename = "effect")]
    pub effect: Effect,

    /// Id.
    #[serde(rename = "id")]
    pub id: String,

    /// Principals.
    #[serde(rename = "principals")]
    pub principals: Vec<Principal>,

    /// Resources.
    #[serde(rename = "resources")]
    pub resources: Vec<Resource>,
}

/// Principal.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Principal {
    /// Id.
    #[serde(rename = "id")]
    pub id: String,

    /// Noun.
    #[serde(rename = "noun")]
    pub noun: String,

    /// Scope.
    #[serde(rename = "scope")]
    pub scope: String,
}

/// Resource.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Resource {
    /// Id.
    #[serde(rename = "id")]
    pub id: String,

    /// Noun.
    #[serde(rename = "noun")]
    pub noun: String,

    /// Scope.
    #[serde(rename = "scope")]
    pub scope: String,
}

/// Evaluate.
///
/// Returns the first policy that matches the context.
///
/// Returns `None` if no matches were found.
#[must_use]
pub fn evaluate<'a>(context: &Context, policies: &'a [Policy]) -> Option<&'a Policy> {
    let deny_policies = policies
        .iter()
        .filter(|policy| policy.effect == Effect::Deny);

    if let Some(policy) = find(context, deny_policies) {
        return Some(policy);
    }

    let allow_policies = policies
        .iter()
        .filter(|policy| policy.effect == Effect::Allow);

    if let Some(policy) = find(context, allow_policies) {
        return Some(policy);
    }

    None
}

/// Find.
///
/// Searches for a policy of a policy iterator that satisfies a context.
fn find<'a>(
    context: &Context,
    policies: impl IntoIterator<Item = &'a Policy>,
) -> Option<&'a Policy> {
    policies.into_iter().find(|policy| {
        let any = policy
            .actions
            .iter()
            .any(|action| matches_action(context, action));
        if !any {
            return false;
        }

        let any = policy
            .principals
            .iter()
            .any(|principal| matches_principal(context, principal));
        if !any {
            return false;
        }

        let any = policy
            .resources
            .iter()
            .any(|resource| match_resource(context, resource));
        if !any {
            return false;
        }

        if !policy.conditions.is_empty() {
            let any = policy
                .conditions
                .iter()
                .any(|condition| matches_condition(context, condition));
            if !any {
                return false;
            }
        }

        true
    })
}

/// Returns true if [`Action`] matches [`Context`].
fn matches_action(context: &Context, action: &Action) -> bool {
    if action.noun != "*" && action.noun != context.action.noun {
        return false;
    }

    if action.scope != "*" && action.scope != context.action.scope {
        return false;
    }

    if action.verb != "*" && action.verb != context.action.verb {
        return false;
    }

    true
}

/// Returns true if [`Condition`] matches [`Context`].
fn matches_condition(context: &Context, condition: &Condition) -> bool {
    if let Some(string_equals) = &condition.string_equals {
        if !matches_condition_string_equals(context, string_equals) {
            return false;
        }
    }

    true
}

/// Returns true if [`Context`] is a subset of criteria.
fn matches_condition_string_equals(
    context: &Context,
    criteria: &HashMap<String, HashSet<String>>,
) -> bool {
    criteria.iter().all(|(key, criteria)| {
        context
            .data
            .get(key)
            .map(|context| context.is_subset(criteria))
            .unwrap_or_default()
    })
}

/// Returns true if [`Principal`] matches [`Context`].
fn matches_principal(context: &Context, principal: &Principal) -> bool {
    if principal.id != "*" && principal.id != context.principal.id {
        return false;
    }

    if principal.noun != "*" && principal.noun != context.principal.noun {
        return false;
    }

    if principal.scope != "*" && principal.scope != context.principal.scope {
        return false;
    }

    true
}

/// Returns true if [`Resource`] matches [`Context`].
fn match_resource(context: &Context, resource: &Resource) -> bool {
    if resource.id != "*" && resource.id != context.resource.id {
        return false;
    }

    if resource.noun != "*" && resource.noun != context.resource.noun {
        return false;
    }

    if resource.scope != "*" && resource.scope != context.resource.scope {
        return false;
    }

    true
}
