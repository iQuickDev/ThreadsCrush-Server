use axum::http::{HeaderMap, StatusCode};
use axum_client_ip::SecureClientIp;
use loco_rs::{controller::ErrorDetail, prelude::*};
use serde::Serialize;
use tracing::error;

use crate::{models::_entities::user, utils::get_ip::get_ip};

#[derive(Serialize, Debug)]
struct StatusResponse {
    voted_user: Option<String>,
}

pub async fn status(
    secure_ip: SecureClientIp,
    State(ctx): State<AppContext>,
    headers: HeaderMap,
) -> Result<impl IntoResponse> {
    let ip = get_ip(&secure_ip, &headers);

    let voted_user = user::Model::find_voted_user_by_address(&ctx.db, &ip)
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
