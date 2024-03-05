#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use axum::http::StatusCode;
use axum_client_ip::SecureClientIp;
use loco_rs::{controller::ErrorDetail, prelude::*};
use serde::Deserialize;
use tracing::error;

use crate::{
    app::REQWEST_CLIENT,
    models::{_entities::{user, voter}, voter::VoterError},
};

async fn vote(
    secure_ip: SecureClientIp,
    State(ctx): State<AppContext>,
    Json(params): Json<VoteRequest>,
) -> Result<impl IntoResponse> {
    check_recaptcha_token(&params.recaptcha_token).await.map_err(|err| {
        let status_code;
        let err_shorthand;

        match &err {
            CheckTokenError::GoogleNotWorking(e) => {
                error!("Google not working: {}", e);
                status_code = StatusCode::SERVICE_UNAVAILABLE;
                err_shorthand = "GOOGLE_NOT_WORKING";
            }
            CheckTokenError::FailedToParse(e) => {
                error!("Failed to parse recaptcha response: {}", e);
                status_code = StatusCode::INTERNAL_SERVER_ERROR;
                err_shorthand = "FAILED_TO_PARSE";
            }
            CheckTokenError::RecaptchaFailed => {
                status_code = StatusCode::FORBIDDEN;
                err_shorthand = "RECAPTCHA_FAILED";
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

    voter::Model::add(&ctx.db, &secure_ip.0.to_canonical().to_string()).await.map_err(|err| {
        match err {
            VoterError::AlreadyVoted => {
                Error::CustomError(
                    StatusCode::CONFLICT,
                    ErrorDetail::new("ALREADY_VOTED", "You have already voted"),
                )
            }
            _ => {
                Error::CustomError(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorDetail::new("INTERNAL_ERROR", "Internal server error"),
                )
            }
        }
    })?;

    Ok(StatusCode::OK)
}

#[derive(Deserialize, Debug)]
pub struct VoteRequest {
    pub username: String,
    pub recaptcha_token: String
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

#[derive(thiserror::Error, Debug)]
pub enum CheckTokenError {
    #[error("Google not working")]
    GoogleNotWorking(#[from] reqwest::Error),

    #[error("Failed to parse recaptcha response")]
    FailedToParse(#[from] serde_json::Error),

    #[error("Recaptcha failed")]
    RecaptchaFailed,
}

#[derive(Deserialize, Debug)]
pub struct RecaptchaResponse {
    success: bool,
    _challenge_ts: Option<String>,
    _hostname: Option<String>,
    _error_codes: Option<Vec<String>>,
}

pub async fn check_recaptcha_token(token: &String) -> std::result::Result<(), CheckTokenError> {
    let secret = std::env::var("RECAPTCHA_SECRET").unwrap();

    let request = REQWEST_CLIENT
        .client
        .post(format!("https://www.google.com/recaptcha/api/siteverify?response={}&secret={}", token, secret))
        .header("Content-Length", "0")
        .build()?;
    
    let result = REQWEST_CLIENT.client.execute(request).await?.text().await?;
    
    let result: RecaptchaResponse = serde_json::from_str(&result)?;

    if !result.success {
        return Err(CheckTokenError::RecaptchaFailed);
    }

    Ok(())
}


pub fn routes() -> Routes {
    Routes::new().add("/vote", post(vote))
}
