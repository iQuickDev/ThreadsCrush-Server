use axum::http::{HeaderMap, StatusCode};
use axum_client_ip::SecureClientIp;
use loco_rs::{controller::ErrorDetail, prelude::*};
use tracing::error;

use crate::{
    models::{_entities::voter, voter::DeleteVoterError},
    utils::get_ip::get_ip,
};

pub async fn unvote(
    secure_ip: SecureClientIp,
    State(ctx): State<AppContext>,
    headers: HeaderMap,
) -> Result<impl IntoResponse> {
    let ip = get_ip(&secure_ip, &headers);

    voter::Model::delete(&ctx.db, &ip).await.map_err(|err| {
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
