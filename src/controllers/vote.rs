#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use axum::http::StatusCode;
use axum_client_ip::SecureClientIp;
use loco_rs::{controller::ErrorDetail, prelude::*};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{
    app::REQWEST_CLIENT,
    models::{_entities::user, _entities::voter},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Params {
    pub username: Option<String>,
}

async fn vote(
    secure_ip: SecureClientIp,
    State(ctx): State<AppContext>,
    Json(params): Json<VoteRequest>,
) -> Result<impl IntoResponse> {
    check_username(&params.username).await.map_err(|err| {
        let status_code;
        let err_shorthand;

        match &err {
            UsernameCheckError::ThreadsNotWorking(e) => {
                error!("Threads not working: {}", e);
                status_code = StatusCode::SERVICE_UNAVAILABLE;
                err_shorthand = "THREADS_NOT_WORKING";
            }
            UsernameCheckError::UserNotFound => {
                status_code = StatusCode::NOT_FOUND;
                err_shorthand = "USER_NOT_FOUND";
            }
            UsernameCheckError::LengthInvalid => {
                status_code = StatusCode::BAD_REQUEST;
                err_shorthand = "LENGTH_INVALID";
            }
        };

        Error::CustomError(
            status_code,
            ErrorDetail {
                error: Some(err_shorthand.to_string()),
                description: Some(err.to_string()),
            },
        )
    })?;

    user::Model::add(&ctx.db, &params.username).await?;

    if let Err(_) = voter::Model::add(&ctx.db, &secure_ip.0.to_canonical().to_string()).await {
        return Err(Error::CustomError(
            StatusCode::FORBIDDEN,
            ErrorDetail::new("ALREADY_VOTED", "You have already voted"),
        ));
    }

    Ok(StatusCode::OK)
}

#[derive(Deserialize, Debug)]
pub struct VoteRequest {
    pub username: String,
}

#[derive(thiserror::Error, Debug)]
pub enum UsernameCheckError {
    #[error("Threads not working")]
    ThreadsNotWorking(#[from] reqwest::Error),

    #[error("User not found")]
    UserNotFound,

    #[error("Username is too long/short")]
    LengthInvalid,
}

pub async fn check_username(username: &String) -> std::result::Result<(), UsernameCheckError> {
    if username.is_empty() || username.len() > 30 {
        return Err(UsernameCheckError::LengthInvalid);
    }

    let request = REQWEST_CLIENT
        .client
        .get(format!("https://threads.net/@{}", username))
        .build()?;
    let result = REQWEST_CLIENT.client.execute(request).await?.text().await?;

    match result.find(username) {
        Some(_) => {}
        None => return Err(UsernameCheckError::UserNotFound),
    }

    Ok(())
}

pub fn routes() -> Routes {
    Routes::new().add("/vote", post(vote))
}
