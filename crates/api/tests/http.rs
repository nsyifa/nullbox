//! HTTP tests: the whole router against in-memory SQLite, via `oneshot`.
//! Copy a test, change the requests, done.

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use nullbox_api::{AppState, Config, db, http};
use serde_json::{Value, json};
use tower::ServiceExt;

const WORKER_TOKEN: &str = "synthetic-worker-token";
const HOT_TOKEN: &str = "synthetic-hot-token";

async fn app() -> Router {
    app_with_worker_tokens(Some(WORKER_TOKEN), Some(HOT_TOKEN)).await
}

async fn app_with_worker_tokens(worker: Option<&str>, hot: Option<&str>) -> Router {
    let pool = db::connect_in_memory().await.expect("in-memory db");
    let config = Config {
        bind_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite::memory:".into(),
        cors_allowed_origins: vec!["http://localhost:5173".into()],
        public_web_url: "http://localhost:5173".into(),
        admin_token: "test-admin-token".into(),
        mcp_token: Some(MCP_TOKEN.into()),
        worker_token: worker.map(str::to_owned),
        hot_worker_token: hot.map(str::to_owned),
    };
    http::router(AppState::new(pool, config))
}

/// Send a request; returns status and JSON body (`Null` if empty).
async fn send(
    app: &Router,
    method: &str,
    uri: &str,
    headers: &[(&str, &str)],
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut req = Request::builder().method(method).uri(uri);
    for (name, value) in headers {
        req = req.header(*name, *value);
    }
    let body = match body {
        Some(json) => {
            req = req.header("content-type", "application/json");
            Body::from(json.to_string())
        }
        None => Body::empty(),
    };
    let res = app.clone().oneshot(req.body(body).unwrap()).await.unwrap();
    let status = res.status();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

const MCP_TOKEN: &str = "test-mcp-token";
const WORKER: (&str, &str) = ("authorization", "Bearer synthetic-worker-token");
const HOT: (&str, &str) = ("authorization", "Bearer synthetic-hot-token");
const FORGED_NAME: (&str, &str) = ("x-actor-name", "%E5%B1%B1%E7%94%B0"); // 山田, percent-encoded

#[tokio::test]
async fn customer_and_worker_round_trip() {
    let app = app().await;

    // Anonymous customer creates a ticket.
    let (status, created) = send(
        &app,
        "POST",
        "/api/tickets",
        &[],
        Some(json!({ "subject": "Printer on fire", "body": "Again.", "severity": "p1" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let id = created["ticket"]["id"].as_str().unwrap().to_owned();
    let token = created["access_token"].as_str().unwrap().to_owned();
    assert_eq!(created["ticket"]["number"], 1);
    assert!(created["ticket"]["anon"]["animal"].is_string());
    assert!(created["access_link"].as_str().unwrap().ends_with(&format!("/t/{id}#{token}")));

    // The secret link works; a wrong token looks like a missing ticket.
    let (status, thread) =
        send(&app, "GET", &format!("/api/public/tickets/{id}"), &[("x-ticket-token", &token)], None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(thread["ticket"]["subject"], "Printer on fire");
    let (status, error) =
        send(&app, "GET", &format!("/api/public/tickets/{id}"), &[("x-ticket-token", "nope")], None).await;
    assert_eq!((status, &error["code"]), (StatusCode::NOT_FOUND, &json!("not_found")));

    // The worker sees it in the inbox, notes internally and yeets it.
    let (_, inbox) = send(&app, "GET", "/api/tickets", &[WORKER], None).await;
    assert_eq!(inbox.as_array().unwrap().len(), 1);
    let (status, _) = send(
        &app,
        "POST",
        &format!("/api/tickets/{id}/messages"),
        &[WORKER],
        Some(json!({ "body": "it's always DNS", "internal": true })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let (status, yeeted) = send(&app, "POST", &format!("/api/tickets/{id}/yeet"), &[WORKER], None).await;
    assert_eq!((status, &yeeted["status"]), (StatusCode::OK, &json!("resolved")));

    let (_, stats) = send(&app, "GET", "/api/stats/backlog", &[WORKER], None).await;
    assert_eq!(stats, json!({ "open_count": 0, "beast_mood": "asleep", "oldest_open_age_secs": null }));

    // The customer never sees internal notes, and replying reopens the ticket.
    let (status, _) = send(
        &app,
        "POST",
        &format!("/api/public/tickets/{id}/messages"),
        &[("x-ticket-token", &token)],
        Some(json!({ "body": "Still on fire" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let (_, thread) =
        send(&app, "GET", &format!("/api/public/tickets/{id}"), &[("x-ticket-token", &token)], None).await;
    assert_eq!(thread["ticket"]["status"], "open");
    let messages = thread["messages"].as_array().unwrap();
    assert!(messages.iter().all(|m| m["internal"] == false));
    assert!(
        messages
            .iter()
            .any(|m| m["system_event"] == json!({ "kind": "status_changed", "from": "resolved", "to": "open" }))
    );

    // The worker view does include the note, written by the identity behind the token.
    let (_, thread) = send(&app, "GET", &format!("/api/tickets/{id}"), &[WORKER], None).await;
    let note = thread["messages"].as_array().unwrap().iter().find(|m| m["internal"] == true).unwrap();
    assert_eq!(note["author_name"], "worker");
}

#[tokio::test]
async fn errors_share_one_shape() {
    let app = app().await;

    let (status, error) = send(&app, "GET", "/api/tickets", &[], None).await;
    assert_eq!((status, &error["code"]), (StatusCode::UNAUTHORIZED, &json!("missing_actor")));

    let (status, error) =
        send(&app, "POST", "/api/tickets", &[], Some(json!({ "subject": " ", "body": "x", "severity": "p4" }))).await;
    assert_eq!((status, &error["code"]), (StatusCode::UNPROCESSABLE_ENTITY, &json!("validation_failed")));

    let (status, error) = send(&app, "GET", "/api/tickets/not-a-uuid", &[WORKER], None).await;
    assert_eq!((status, &error["code"]), (StatusCode::NOT_FOUND, &json!("not_found")));

    let (status, error) = send(&app, "GET", "/api/teapot", &[], None).await;
    assert_eq!((status, &error["code"]), (StatusCode::IM_A_TEAPOT, &json!("im_a_teapot")));
}

#[tokio::test]
async fn admin_export_needs_the_admin_token() {
    let app = app().await;
    let ticket = json!({ "subject": "Export me", "body": "x", "severity": "p3", "customer_email": "a@example.com" });
    send(&app, "POST", "/api/tickets", &[], Some(ticket)).await;

    let (status, error) = send(&app, "GET", "/api/admin/export", &[], None).await;
    assert_eq!((status, &error["code"]), (StatusCode::FORBIDDEN, &json!("forbidden")));
    let (status, _) = send(&app, "GET", "/api/admin/export", &[("x-admin-token", "nope")], None).await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let admin = [("x-admin-token", "test-admin-token")];
    let (status, tickets) = send(&app, "GET", "/api/admin/export", &admin, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(tickets[0]["customer_email"], "a@example.com");
}

#[tokio::test]
async fn customer_link_only_opens_its_own_ticket() {
    let app = app().await;

    let mut tickets = Vec::new();
    for subject in ["Ticket A", "Ticket B"] {
        let (status, created) =
            send(&app, "POST", "/api/tickets", &[], Some(json!({ "subject": subject, "body": "x", "severity": "p3" })))
                .await;
        assert_eq!(status, StatusCode::CREATED);
        let id = created["ticket"]["id"].as_str().unwrap().to_owned();
        let token = created["access_token"].as_str().unwrap().to_owned();
        tickets.push((id, token));
    }
    let [(a_id, a_token), (b_id, b_token)] = <[_; 2]>::try_from(tickets).unwrap();
    assert_ne!(a_token, b_token);

    let worker_view = |id: String| {
        let app = app.clone();
        async move { send(&app, "GET", &format!("/api/tickets/{id}"), &[WORKER], None).await.1 }
    };
    let before = (worker_view(a_id.clone()).await, worker_view(b_id.clone()).await);

    // A token with B's ID, and B token with A's ID: both look like a missing ticket.
    for (token, other_id) in [(&a_token, &b_id), (&b_token, &a_id)] {
        let (status, error) =
            send(&app, "GET", &format!("/api/public/tickets/{other_id}"), &[("x-ticket-token", token)], None).await;
        assert_eq!((status, &error["code"]), (StatusCode::NOT_FOUND, &json!("not_found")));

        let (status, error) = send(
            &app,
            "POST",
            &format!("/api/public/tickets/{other_id}/messages"),
            &[("x-ticket-token", token)],
            Some(json!({ "body": "not my ticket" })),
        )
        .await;
        assert_eq!((status, &error["code"]), (StatusCode::NOT_FOUND, &json!("not_found")));
    }

    // Rejected replies left both tickets untouched.
    let after = (worker_view(a_id.clone()).await, worker_view(b_id.clone()).await);
    assert_eq!(before, after);

    // Each link still reads and replies to its own ticket.
    for (id, token, subject) in [(&a_id, &a_token, "Ticket A"), (&b_id, &b_token, "Ticket B")] {
        let (status, thread) =
            send(&app, "GET", &format!("/api/public/tickets/{id}"), &[("x-ticket-token", token)], None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(thread["ticket"]["subject"], subject);

        let (status, _) = send(
            &app,
            "POST",
            &format!("/api/public/tickets/{id}/messages"),
            &[("x-ticket-token", token)],
            Some(json!({ "body": format!("reply to {subject}") })),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);

        let thread = worker_view(id.clone()).await;
        let bodies: Vec<_> = thread["messages"].as_array().unwrap().iter().map(|m| m["body"].clone()).collect();
        assert!(bodies.contains(&json!(format!("reply to {subject}"))));
        assert!(!bodies.contains(&json!("not my ticket")));
    }
}

#[tokio::test]
async fn mcp_requires_bearer_and_lists_tools() {
    let app = app().await;
    let init = json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
        "protocolVersion": "2025-11-25", "capabilities": {}, "clientInfo": {"name": "test", "version": "1"}
    }});

    let (status, _) = send(&app, "POST", "/mcp", &[], Some(init.clone())).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _) = send(&app, "POST", "/mcp", &[("authorization", "Bearer nope")], Some(init.clone())).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let auth = ("authorization", "Bearer test-mcp-token");
    let (status, res) = send(&app, "POST", "/mcp", &[auth], Some(init)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(res["result"]["protocolVersion"], "2025-11-25");

    let (status, _) =
        send(&app, "POST", "/mcp", &[auth], Some(json!({"jsonrpc": "2.0", "method": "notifications/initialized"}))).await;
    assert_eq!(status, StatusCode::ACCEPTED);

    let (_, res) = send(&app, "POST", "/mcp", &[auth], Some(json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list"}))).await;
    assert!(!res["result"]["tools"].as_array().unwrap().is_empty());

    let call = json!({"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {"name": "list_tickets", "arguments": {}}});
    let (_, res) = send(&app, "POST", "/mcp", &[auth], Some(call)).await;
    assert_eq!(res["result"]["isError"], false);
}

/// Every worker route, as (method, path, body). `{id}` is replaced with a real ticket.
fn worker_routes() -> Vec<(&'static str, &'static str, Option<Value>)> {
    vec![
        ("GET", "/api/tickets", None),
        ("GET", "/api/tickets/{id}", None),
        ("POST", "/api/tickets/{id}/messages", Some(json!({ "body": "hi" }))),
        ("PATCH", "/api/tickets/{id}", Some(json!({ "severity": "p2" }))),
        ("POST", "/api/tickets/{id}/yeet", None),
        ("GET", "/api/stats/backlog", None),
    ]
}

async fn create_ticket(app: &Router) -> (String, String) {
    let (status, created) =
        send(app, "POST", "/api/tickets", &[], Some(json!({ "subject": "Locked", "body": "x", "severity": "p3" })))
            .await;
    assert_eq!(status, StatusCode::CREATED);
    (created["ticket"]["id"].as_str().unwrap().to_owned(), created["access_token"].as_str().unwrap().to_owned())
}

async fn assert_all_worker_routes_denied(app: &Router, id: &str, headers: &[(&str, &str)], case: &str) {
    for (method, path, body) in worker_routes() {
        let uri = path.replace("{id}", id);
        let (status, error) = send(app, method, &uri, headers, body).await;
        assert_eq!(
            (status, &error["code"]),
            (StatusCode::UNAUTHORIZED, &json!("missing_actor")),
            "{case}: {method} {uri} must be refused"
        );
        assert!(!error.to_string().contains("synthetic-"), "{case}: error must not echo a token");
    }
}

#[tokio::test]
async fn worker_routes_refuse_missing_forged_and_bad_credentials() {
    let app = app().await;
    let (id, ticket_token) = create_ticket(&app).await;

    assert_all_worker_routes_denied(&app, &id, &[], "no credentials").await;
    assert_all_worker_routes_denied(&app, &id, &[FORGED_NAME], "X-Actor-Name alone").await;
    assert_all_worker_routes_denied(&app, &id, &[("authorization", "Bearer made-up-token")], "made-up bearer").await;
    assert_all_worker_routes_denied(&app, &id, &[("authorization", WORKER_TOKEN)], "token without scheme").await;
    assert_all_worker_routes_denied(&app, &id, &[("authorization", "Bearer ")], "empty bearer").await;
    assert_all_worker_routes_denied(&app, &id, &[("x-ticket-token", &ticket_token)], "customer token").await;
    assert_all_worker_routes_denied(&app, &id, &[("x-admin-token", "test-admin-token")], "admin token").await;

    // The 401 tells the client which scheme to use.
    let res = app.clone().oneshot(Request::builder().uri("/api/tickets").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(res.headers().get("www-authenticate").unwrap(), "Bearer");

    // Nothing was changed by the refused requests.
    let (_, thread) = send(&app, "GET", &format!("/api/tickets/{id}"), &[WORKER], None).await;
    assert_eq!(thread["ticket"]["status"], "open");
    assert_eq!(thread["ticket"]["severity"], "p3");
    assert!(thread["messages"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn worker_and_hot_tokens_are_separate_identities() {
    let app = app().await;
    let (id, _) = create_ticket(&app).await;

    for (creds, name) in [(WORKER, "worker"), (HOT, "HOT")] {
        let (status, inbox) = send(&app, "GET", "/api/tickets", &[creds], None).await;
        assert_eq!(status, StatusCode::OK, "{name} reads the inbox");
        assert_eq!(inbox.as_array().unwrap().len(), 1);

        // A forged name next to a valid token changes nothing: the token decides.
        let (status, message) = send(
            &app,
            "POST",
            &format!("/api/tickets/{id}/messages"),
            &[creds, FORGED_NAME],
            Some(json!({ "body": format!("reply from {name}") })),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED, "{name} replies");
        assert_eq!(message["author_name"], name);

        let (status, _) = send(&app, "GET", "/api/stats/backlog", &[creds], None).await;
        assert_eq!(status, StatusCode::OK);
    }

    let (status, ticket) =
        send(&app, "PATCH", &format!("/api/tickets/{id}"), &[HOT], Some(json!({ "severity": "p1" }))).await;
    assert_eq!((status, &ticket["severity"]), (StatusCode::OK, &json!("p1")));
    let (status, ticket) = send(&app, "POST", &format!("/api/tickets/{id}/yeet"), &[WORKER], None).await;
    assert_eq!((status, &ticket["status"]), (StatusCode::OK, &json!("resolved")));

    // Lower-case scheme is accepted too.
    let (status, _) = send(&app, "GET", "/api/tickets", &[("authorization", "bearer synthetic-hot-token")], None).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn worker_access_fails_closed_without_configured_tokens() {
    let app = app_with_worker_tokens(None, None).await;
    let (id, ticket_token) = create_ticket(&app).await;

    assert_all_worker_routes_denied(&app, &id, &[WORKER], "worker token, none configured").await;
    assert_all_worker_routes_denied(&app, &id, &[HOT], "HOT token, none configured").await;

    // Customers are unaffected.
    let (status, thread) =
        send(&app, "GET", &format!("/api/public/tickets/{id}"), &[("x-ticket-token", &ticket_token)], None).await;
    assert_eq!((status, &thread["ticket"]["subject"]), (StatusCode::OK, &json!("Locked")));

    // Only HOT configured: the worker token is refused, HOT works.
    let app = app_with_worker_tokens(None, Some(HOT_TOKEN)).await;
    let (status, _) = send(&app, "GET", "/api/tickets", &[WORKER], None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _) = send(&app, "GET", "/api/tickets", &[HOT], None).await;
    assert_eq!(status, StatusCode::OK);
}
