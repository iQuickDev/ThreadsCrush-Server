use crate::models::user::UserWithVotes;
use serde::Serialize;

#[derive(Serialize, Default)]
pub struct LeaderboardResponse {
    pub pagination: Pagination,
    pub users: Vec<User>,
}

#[derive(Serialize, Default)]
pub struct Pagination {
    pub current: u64,
    pub last: u64,
    pub entries: u64,
}

#[derive(Serialize, Default)]
pub struct User {
    username: String,
    votes: i64,
    rank: i64
}

impl LeaderboardResponse {
    pub fn new(users: Vec<UserWithVotes>, pagination: Pagination) -> Self {
        let users = users.into_iter().map(|user| user.into()).collect();

        LeaderboardResponse { pagination, users }
    }
}

impl From<UserWithVotes> for User {
    fn from(user: UserWithVotes) -> Self {
        User {
            username: user.username,
            votes: user.votes,
            rank: user.rank
        }
    }
}
