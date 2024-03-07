use axum::http::StatusCode;
use axum_client_ip::SecureClientIp;
use loco_rs::{controller::ErrorDetail, prelude::*};
use serde::Serialize;
use tracing::error;

use crate::models::_entities::user;

#[derive(Serialize, Debug)]
struct StatusResponse {
    voted_user: Option<String>,
}

pub async fn status(
    secure_ip: SecureClientIp,
    State(ctx): State<AppContext>,
) -> Result<impl IntoResponse> {
    let address = secure_ip.0.to_canonical().to_string();

    let voted_user = user::Model::find_voted_user_by_address(&ctx.db, &address)
        .await
        .map_err(|err| {
            error!("Internal server error while getting status: {}", err);

            Error::CustomError(
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorDetail::new("INTERNAL_ERROR", "Internal server error"),
            )
        })?;

    Ok(Json(StatusResponse {
        voted_user: voted_user.map(|u| u.username),
    }))
}
