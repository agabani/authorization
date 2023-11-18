use std::{
    collections::{HashMap, HashSet},
    net::SocketAddr,
    sync::Arc,
};

use authorization::{evaluate, Action, Condition, Context, Effect, Policy, Principal, Resource};
use axum::{
    extract::{ConnectInfo, Path, State},
    http::{header, Request, StatusCode},
    middleware::{self, Next},
    response::IntoResponse,
    routing, Extension, Json, Router,
};
use tokio::sync::Mutex;
use tower::ServiceBuilder;

#[tokio::main]
async fn main() {
    let database = Database {
        policies: Arc::new(Mutex::new(vec![
            Policy {
                actions: vec![
                    Action {
                        noun: "user".to_string(),
                        scope: "identity".to_string(),
                        verb: "get".to_string(),
                    },
                    Action {
                        noun: "user".to_string(),
                        scope: "identity".to_string(),
                        verb: "list".to_string(),
                    },
                ],
                conditions: vec![],
                effect: Effect::Allow,
                id: "read".to_string(),
                principals: vec![Principal {
                    id: "*".to_string(),
                    noun: "*".to_string(),
                    scope: "*".to_string(),
                }],
                resources: vec![Resource {
                    id: "*".to_string(),
                    noun: "user".to_string(),
                    scope: "identity".to_string(),
                }],
            },
            Policy {
                actions: vec![Action {
                    noun: "*".to_string(),
                    scope: "*".to_string(),
                    verb: "*".to_string(),
                }],
                conditions: vec![Condition {
                    string_equals: Some(HashMap::from([
                        (
                            "request:client_ip".to_string(),
                            HashSet::from(["127.0.0.1".to_string()]),
                        ),
                        (
                            "request:host".to_string(),
                            HashSet::from(["localhost:3000".to_string()]),
                        ),
                    ])),
                }],
                effect: Effect::Allow,
                id: "admin".to_string(),
                principals: vec![Principal {
                    id: "00000000-0000-0000-0000-000000000000".to_string(),
                    noun: "user".to_string(),
                    scope: "local".to_string(),
                }],
                resources: vec![Resource {
                    id: "*".to_string(),
                    noun: "*".to_string(),
                    scope: "*".to_string(),
                }],
            },
        ])),
    };

    let app = Router::new()
        .route("/users", routing::get(handler_users_list))
        .route(
            "/users/:user_id",
            routing::delete(handler_users_delete)
                .get(handler_users_get)
                .put(handler_users_put),
        )
        .layer(
            ServiceBuilder::new()
                .layer(middleware::from_fn(middleware_authentication_context))
                .layer(middleware::from_fn(middleware_request_context)),
        )
        .with_state(database);

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}

/// [`Database`].
///
/// Contains a searchable collection of policies.
#[derive(Clone)]
struct Database {
    policies: Arc<Mutex<Vec<Policy>>>,
}

impl Database {
    /// Returns all policies which references the [`Principal`].
    async fn find_by_principal(&self, p: &Principal) -> Vec<Policy> {
        self.policies
            .lock()
            .await
            .iter()
            .filter(|policy| {
                policy.principals.iter().any(|f| {
                    (f.scope == p.scope || f.scope == "*")
                        && (f.noun == p.noun || f.noun == "*")
                        && (f.id == p.id || f.id == "*")
                })
            })
            .cloned()
            .collect()
    }
}

/// [`RequestContext`]
///
/// Contains contextual information about the current request.
#[derive(Clone)]
struct RequestContext(HashMap<String, HashSet<String>>);

/// Deletes a user.
async fn handler_users_delete(
    State(database): State<Database>,
    Extension(context): Extension<RequestContext>,
    Extension(principal): Extension<Principal>,
    Path(user_id): Path<String>,
) -> impl IntoResponse {
    let action = Action {
        noun: "user".to_string(),
        scope: "identity".to_string(),
        verb: "delete".to_string(),
    };

    let resource = Resource {
        id: user_id,
        noun: "user".to_string(),
        scope: "identity".to_string(),
    };

    if let Some(response) =
        utility_authorize(&database, &context, &principal, action, resource).await
    {
        return response.into_response();
    }

    (StatusCode::OK, Json("Hello, World!")).into_response()
}

/// Gets a user.
async fn handler_users_get(
    State(database): State<Database>,
    Extension(context): Extension<RequestContext>,
    Extension(principal): Extension<Principal>,
    Path(user_id): Path<String>,
) -> impl IntoResponse {
    let action = Action {
        noun: "user".to_string(),
        scope: "identity".to_string(),
        verb: "get".to_string(),
    };

    let resource = Resource {
        id: user_id,
        noun: "user".to_string(),
        scope: "identity".to_string(),
    };

    if let Some(response) =
        utility_authorize(&database, &context, &principal, action, resource).await
    {
        return response.into_response();
    }

    (StatusCode::OK, Json("Hello, World!")).into_response()
}

/// Lists users.
async fn handler_users_list(
    State(database): State<Database>,
    Extension(context): Extension<RequestContext>,
    Extension(principal): Extension<Principal>,
) -> impl IntoResponse {
    let action = Action {
        noun: "user".to_string(),
        scope: "identity".to_string(),
        verb: "list".to_string(),
    };

    let resource = Resource {
        id: "".to_string(),
        noun: "user".to_string(),
        scope: "identity".to_string(),
    };

    if let Some(response) =
        utility_authorize(&database, &context, &principal, action, resource).await
    {
        return response.into_response();
    }

    (StatusCode::OK, Json("Hello, World!")).into_response()
}

/// Updates a user.
async fn handler_users_put(
    State(database): State<Database>,
    Extension(context): Extension<RequestContext>,
    Extension(principal): Extension<Principal>,
    Path(user_id): Path<String>,
) -> impl IntoResponse {
    let action = Action {
        noun: "user".to_string(),
        scope: "identity".to_string(),
        verb: "put".to_string(),
    };

    let resource = Resource {
        id: user_id,
        noun: "user".to_string(),
        scope: "identity".to_string(),
    };

    if let Some(response) =
        utility_authorize(&database, &context, &principal, action, resource).await
    {
        return response.into_response();
    }

    (StatusCode::OK, Json("Hello, World!")).into_response()
}

/// Middleware to add [`Principal`] to the request.
async fn middleware_authentication_context<B>(
    mut req: Request<B>,
    next: Next<B>,
) -> impl IntoResponse {
    let principal = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| header.split_once(' '))
        .filter(|(scheme, _)| *scheme == "Example")
        .map(|(_, value)| value.splitn(3, ':'))
        .and_then(
            |mut parts| match (parts.next(), parts.next(), parts.next()) {
                (Some(scope), Some(noun), Some(id)) => Some(Principal {
                    id: id.to_string(),
                    noun: noun.to_string(),
                    scope: scope.to_string(),
                }),
                _ => None,
            },
        )
        .unwrap_or_else(|| Principal {
            id: "".to_string(),
            noun: "".to_string(),
            scope: "anonymous".to_string(),
        });

    req.extensions_mut().insert(principal);

    next.run(req).await
}

/// Middleware to add [`RequestContext`] to the request.
async fn middleware_request_context<B>(mut req: Request<B>, next: Next<B>) -> impl IntoResponse {
    let mut request_context = HashMap::new();

    if let Some(ConnectInfo(socket_addr)) = req.extensions().get::<ConnectInfo<SocketAddr>>() {
        request_context.insert(
            "request:client_ip".to_string(),
            HashSet::from([socket_addr.ip().to_string()]),
        );
    }

    if let Some(value) = req
        .headers()
        .get(header::HOST)
        .and_then(|value| value.to_str().ok())
    {
        request_context.insert(
            "request:host".to_string(),
            HashSet::from([value.to_string()]),
        );
    }

    if let Some(value) = req
        .headers()
        .get(header::USER_AGENT)
        .and_then(|value| value.to_str().ok())
    {
        request_context.insert(
            "request:user_agent".to_string(),
            HashSet::from([value.to_string()]),
        );
    }

    req.extensions_mut().insert(RequestContext(request_context));

    next.run(req).await
}

/// Utility to check for authorization and logs the reason for the decision.
///
/// Returns [`IntoResponse`] if the request is not authorized.
///
/// Returns [`None`] if the request is authorized.
async fn utility_authorize(
    database: &Database,
    context: &RequestContext,
    principal: &Principal,
    action: Action,
    resource: Resource,
) -> Option<impl IntoResponse> {
    let context = Context {
        action,
        data: context.0.clone(),
        principal: principal.clone(),
        resource,
    };

    let policies = database.find_by_principal(principal).await;

    println!();
    match evaluate(&context, &policies) {
        Some(policy) => match policy.effect {
            Effect::Allow => {
                println!("explicit allow: {context:?} {policy:?}");
                None
            }
            Effect::Deny => {
                println!("explicit deny: {context:?} {policy:?}");
                Some((StatusCode::FORBIDDEN).into_response())
            }
        },
        None => {
            println!("implicit deny: {context:?}");
            Some((StatusCode::FORBIDDEN).into_response())
        }
    }
}
