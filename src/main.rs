use axum::{Router, routing::post};
use serde::Deserialize;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::requests::allowed_request::checkout;
use crate::requests::return_request::return_umbrella;

mod requests;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct UserId([u8; 7]);

#[derive(Clone)]
struct AppState {
    lookup_table: Arc<RwLock<LookupTable>>,
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
            println!("User already has an umbrella taken out!");
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
        lookup_table: Arc::new(RwLock::new(LookupTable {
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
