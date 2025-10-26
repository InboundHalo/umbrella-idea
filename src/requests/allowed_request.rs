use std::time::Duration;

use axum::{Json, extract::State, http::StatusCode};
use tokio::time::sleep;

use crate::{ActionReq, AppState, parse_user_id};

// POST /allowed
// Body: { "user_id": "0x04:04:04:04:04:04:04", "umbrella_id": 123 }
// Response: "yes" or "no" (text/plain)
pub async fn checkout(
    State(state): State<AppState>,
    Json(req): Json<ActionReq>,
) -> (StatusCode, String) {
    let user_id = match parse_user_id(&req.user_id) {
        Ok(u) => u,
        Err(e) => return (StatusCode::BAD_REQUEST, e),
    };

    let mut lookup_table = state.lookup_table.write().await;

    if !lookup_table.user_allowed_to_take_out_umbrella(&user_id, &req.umbrella_id) {
        return (StatusCode::OK, "no".to_string());
    }

    lookup_table.checked_out_by.insert(req.umbrella_id, user_id);
    lookup_table.holding.insert(user_id, req.umbrella_id);

    println!(
        "user_id: {:?} took out umbrella_id: {}",
        user_id, req.umbrella_id
    );

    tokio::spawn(async move {
        sleep(Duration::from_secs(4)).await;
        println!(
            "user_id: {:?}, it has been 4 seconds please return",
            user_id
        );
    });
    tokio::spawn(async move {
        sleep(Duration::from_secs(8)).await;
        println!(
            "user_id: {:?}, it has been 8 seconds you are charged will be charged with a late fee of $1",
            user_id
        );
    });

    (StatusCode::OK, "yes".to_string())
}
