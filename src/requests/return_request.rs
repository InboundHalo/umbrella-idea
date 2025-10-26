use axum::{Json, extract::State, http::StatusCode};

use crate::{ActionReq, AppState, parse_user_id};

// POST /return
// Body: { "user_id": "0x04:04:04:04:04:04:04", "umbrella_id": 123 }
// Response: "confirmed" or "failed" (text/plain)
pub async fn return_umbrella(
    State(state): State<AppState>,
    Json(req): Json<ActionReq>,
) -> (StatusCode, String) {
    let uid = match parse_user_id(&req.user_id) {
        Ok(u) => u,
        Err(e) => return (StatusCode::BAD_REQUEST, e),
    };

    let mut inner = state.lookup_table.write().await;

    match inner.checked_out_by.get(&req.umbrella_id) {
        Some(holder) if holder == &uid => {
            inner.checked_out_by.remove(&req.umbrella_id);
            inner.holding.remove(&uid);
            println!("Returned umbrella_id: {}", req.umbrella_id);
            (StatusCode::OK, "confirmed".to_string())
        }
        _ => (StatusCode::OK, "failed".to_string()),
    }
}
