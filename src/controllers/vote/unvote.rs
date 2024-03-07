use axum::http::StatusCode;
use axum_client_ip::SecureClientIp;
use loco_rs::{controller::ErrorDetail, prelude::*};
use tracing::error;

use crate::models::_entities::voter;
use crate::models::voter::DeleteVoterError;

pub async fn unvote(
    secure_ip: SecureClientIp,
    State(ctx): State<AppContext>,
) -> Result<impl IntoResponse> {
    let address = secure_ip.0.to_canonical().to_string();

    voter::Model::delete(&ctx.db, &address)
        .await
        .map_err(|err| {
            let status_code;
            let err_shorthand;

            match err {
                DeleteVoterError::NotFound => {
                    status_code = StatusCode::NOT_FOUND;
                    err_shorthand = "NOT_FOUND";
                }
                _ => {
                    error!("Error unvoting: {:?}", err);
                    status_code = StatusCode::INTERNAL_SERVER_ERROR;
                    err_shorthand = "INTERNAL_SERVER_ERROR";
                }
            }

            Error::CustomError(
                status_code,
                ErrorDetail {
                    error: Some(err_shorthand.to_string()),
                    description: Some(err.to_string()),
                },
            )
        })?;

    Ok(StatusCode::OK)
}
