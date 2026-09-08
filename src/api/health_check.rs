use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use tracing::instrument;

#[instrument(name = "GET: Health Check", skip_all)]
pub async fn router(State(state): State<crate::startup::AppState>) -> impl IntoResponse {
    StatusCode::OK
}
