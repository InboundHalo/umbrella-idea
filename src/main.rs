use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
use serde::Deserialize;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct UserId([u8; 7]);

#[derive(Clone)]
struct AppState {
    inner: Arc<RwLock<LookupTable>>,
}

// TODO: time based logging, send an email after checkout, 1 hour of use, and 24, with a email with
// a intent of fine after that
struct LookupTable {
    // umbrella_id -> user_id
    checked_out_by: HashMap<u32, UserId>,
    // user_id -> umbrella_id
    holding: HashMap<UserId, u32>,
}

impl LookupTable {
    fn user_allowed_to_take_out_umbrella(&self, user_id: &UserId, umbrella_id: &u32) -> bool {
        if self.holding.contains_key(user_id) {
            false
        } else if self.checked_out_by.contains_key(umbrella_id) {
            println!("Error: umbrella should not be able to offered if it is already checked out!");
            false
        } else {
            true
        }
    }
}

#[derive(Deserialize)]
struct ActionReq {
    user_id: String,
    umbrella_id: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let state = AppState {
        inner: Arc::new(RwLock::new(LookupTable {
            checked_out_by: HashMap::new(),
            holding: HashMap::new(),
        })),
    };

    let app = Router::new()
        .route("/allowed", post(checkout))
        .route("/return", post(return_umbrella))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

// POST /allowed
// Body: { "user_id": "0x04:04:04:04:04:04:04", "umbrella_id": 123 }
// Response: "yes" or "no" (text/plain)
async fn checkout(
    State(state): State<AppState>,
    Json(req): Json<ActionReq>,
) -> (StatusCode, String) {
    let user_id = match parse_user_id(&req.user_id) {
        Ok(u) => u,
        Err(e) => return (StatusCode::BAD_REQUEST, e),
    };

    let mut inner = state.inner.write().await;

    if !inner.user_allowed_to_take_out_umbrella(&user_id, &req.umbrella_id) {
        return (StatusCode::OK, "no".to_string());
    }

    inner.checked_out_by.insert(req.umbrella_id, user_id);
    inner.holding.insert(user_id, req.umbrella_id);
    (StatusCode::OK, "yes".to_string())
}

// POST /return
// Body: { "user_id": "0x04:04:04:04:04:04:04", "umbrella_id": 123 }
// Response: "confirmed" or "failed" (text/plain)
async fn return_umbrella(
    State(state): State<AppState>,
    Json(req): Json<ActionReq>,
) -> (StatusCode, String) {
    let uid = match parse_user_id(&req.user_id) {
        Ok(u) => u,
        Err(e) => return (StatusCode::BAD_REQUEST, e),
    };

    let mut inner = state.inner.write().await;

    match inner.checked_out_by.get(&req.umbrella_id) {
        Some(holder) if holder == &uid => {
            inner.checked_out_by.remove(&req.umbrella_id);
            inner.holding.remove(&uid);
            (StatusCode::OK, "confirmed".to_string())
        }
        _ => (StatusCode::OK, "failed".to_string()),
    }
}

// Accepts forms:
// - "0x04040404040404"
// - "04040404040404"
// - "0x04:04:04:04:04:04:04"
// - "04:04:04:04:04:04:04"
// Returns error if not exactly 14 hex digits after normalization.
fn parse_user_id(raw: &str) -> Result<UserId, String> {
    let mut string = raw.trim(); // Remove any \n

    // 0x may be at the start, remove it
    if let Some(rest) = string.strip_prefix("0x") {
        string = rest;
    }

    // Remove : from string
    let hex: String = string.chars().filter(|c| *c != ':').collect();

    // hex now does not have the 0x or any : so it should have 7 * 2 characters, 2 per hex value
    if hex.len() != 14 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("user_id must be 14 hex digits (7 bytes), optional 0x and colons".to_string());
    }

    // Build output 2 characters at a time
    let mut out = [0u8; 7];
    for i in 0..7 {
        let hi = i * 2;
        let b = u8::from_str_radix(&hex[hi..hi + 2], 16)
            .map_err(|_| "user_id contains invalid hex".to_string())?;
        out[i] = b;
    }

    Ok(UserId(out))
}

