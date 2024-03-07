use loco_rs::prelude::*;

pub mod status;
pub mod unvote;
pub mod vote;

pub fn routes() -> Routes {
    Routes::new()
        .add("/vote", post(vote::vote))
        .add("/vote", delete(unvote::unvote))
        .add("/vote/status", get(status::status))
}
