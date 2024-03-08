use crate::views::leaderboard::LeaderboardResponse;
use crate::{common, models::_entities::user};
use axum::{extract::Query, http::StatusCode};
use loco_rs::{controller::ErrorDetail, prelude::*};
use serde::Deserialize;

#[derive(Deserialize)]
struct LeaderboardRequest {
    username: Option<String>,
    page: u64,
}

async fn leaderboard(
    State(ctx): State<AppContext>,
    Query(params): Query<LeaderboardRequest>,
) -> Result<impl IntoResponse> {
    let settings = &ctx.config.settings.unwrap();
    let settings = common::settings::Settings::from_json(settings)?;

    let mut pagination = user::Model::get_leaderboard_pagination(&ctx.db, settings.page_size, &params.username).await?;
    pagination.current = params.page;

    if params.page > pagination.last {
        return Err(Error::CustomError(
            StatusCode::NOT_FOUND,
            ErrorDetail::new("PAGE_NOT_FOUND", "Page does not exist"),
        ));
    }

    let username = &params.username.map(|u| u.to_lowercase());

    let users = user::Model::find_leaderboard(
        &ctx.db,
        username,
        params.page.try_into().unwrap(),
        settings.page_size,
    )
    .await?;

    let res = format::json(LeaderboardResponse::new(users, pagination))?;

    Ok(res)
}

pub fn routes() -> Routes {
    Routes::new().add("/leaderboard", get(leaderboard))
}
