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

    let mut inner = state.lookup_table.write().await;

    if !inner.user_allowed_to_take_out_umbrella(&user_id, &req.umbrella_id) {
        return (StatusCode::OK, "no".to_string());
    }

    inner.checked_out_by.insert(req.umbrella_id, user_id);
    inner.holding.insert(user_id, req.umbrella_id);

    println!("took out umbrella_id: {}", req.umbrella_id);

    tokio::spawn(async move {
        sleep(Duration::from_secs(4)).await;
        println!("It has been 4 seconds please return");
    });
    tokio::spawn(async move {
        sleep(Duration::from_secs(8)).await;
        println!("It has been 8 seconds please return");
    });

    (StatusCode::OK, "yes".to_string())
}
