use axum::{Json, extract::State, http::StatusCode};

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
    (StatusCode::OK, "yes".to_string())
}
