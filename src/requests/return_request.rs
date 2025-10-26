use axum::{Json, extract::State, http::StatusCode};

use crate::{ActionReq, AppState, UmbrellaId, parse_user_id};

// POST /return
// Body: { "user_id": "0x04:04:04:04:04:04:04", "umbrella_id": 123 }
// Response: "confirmed" or "failed" (text/plain)
pub async fn return_umbrella(
    State(state): State<AppState>,
    Json(req): Json<ActionReq>,
) -> (StatusCode, String) {
    let user_id = match parse_user_id(&req.user_id) {
        Ok(user_id) => user_id,
        Err(e) => return (StatusCode::BAD_REQUEST, e),
    };

    let umbrella_id = UmbrellaId(req.umbrella_id);

    let mut lookup_table = state.lookup_table.write().await;

    match lookup_table.checked_out_by.get(&umbrella_id) {
        Some(holder) => {
            if holder == &user_id {
                lookup_table.checked_out_by.remove(&umbrella_id);
                lookup_table.holding.remove(&user_id);
                println!("Returned umbrella_id: {:?}", umbrella_id);
                (StatusCode::OK, "confirmed".to_string())
            } else {
                if let Some(user_id_that_checked_it_out) =
                    lookup_table.checked_out_by.remove(&umbrella_id)
                {
                    lookup_table.holding.remove(&user_id_that_checked_it_out);
                    println!("Returned umbrella_id: {:?}", umbrella_id);
                    (StatusCode::OK, "confirmed".to_string())
                } else {
                    println!("How did we get here?");
                    (StatusCode::OK, "failed".to_string())
                }
            }
        }
        _ => (StatusCode::OK, "failed".to_string()),
    }
}
