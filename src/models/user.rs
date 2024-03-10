use loco_rs::{model::ModelResult, prelude::*};
use sea_orm::{
    entity::prelude::*, ActiveValue, DatabaseBackend, FromQueryResult, JoinType, QuerySelect,
    Statement, TransactionTrait,
};

use super::_entities::{
    user::{self, ActiveModel},
    voter,
};
use crate::views::leaderboard::Pagination;

impl ActiveModelBehavior for ActiveModel {
    // extend activemodel below (keep comment for generators)
}

#[derive(FromQueryResult, Debug)]
pub struct UserWithVotes {
    pub votes: i64,
    pub username: String,
    pub rank: i64,
}

impl super::_entities::user::Model {
    pub async fn add(db: &DatabaseConnection, username: &str) -> ModelResult<Self> {
        let txn = db.begin().await?;

        let existing_user = user::Entity::find()
            .filter(user::Column::Username.eq(username))
            .one(&txn)
            .await?;

        if let Some(existing_user) = existing_user {
            txn.commit().await?;
            return Ok(existing_user);
        }

        let new_user = user::ActiveModel {
            username: ActiveValue::set(username.to_string()),
            ..Default::default()
        }
        .insert(&txn)
        .await?;

        txn.commit().await?;

        Ok(new_user)
    }

    pub async fn find_leaderboard(
        db: &DatabaseConnection,
        username: &Option<String>,
        page: u64,
        count: u64,
    ) -> ModelResult<Vec<UserWithVotes>> {
        let username = username.as_ref().map(|s| s.as_str());

        let leaderboard_query = user::Entity::find().from_raw_sql(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            r#"SELECT
            user_votes_rank."username",
            user_votes_rank."votes",
            user_votes_rank."rank"
          FROM (
            SELECT
              u."username",
              COUNT(v."id") AS "votes",
              ROW_NUMBER() OVER (ORDER BY COUNT(v."voted_user_id") DESC) AS "rank"
            FROM
              "user" u
              JOIN "voter" v ON (u."id" = v."voted_user_id")
            GROUP BY
              u."id"
          ) AS user_votes_rank
          WHERE
            user_votes_rank."username" LIKE CONCAT($1, '%')
          ORDER BY
            user_votes_rank."rank"
          LIMIT $2
          OFFSET $3"#,
            [username.into(), count.into(), ((page - 1) * count).into()],
        ));

        let users = leaderboard_query
            .into_model::<UserWithVotes>()
            .all(db)
            .await?;

        Ok(users)
    }

    pub async fn get_leaderboard_pagination(
        db: &DatabaseConnection,
        page_size: u64,
        username: &Option<String>,
    ) -> ModelResult<Pagination> {
        let username = username.clone().unwrap_or("".to_string());

        let entries = user::Entity::find()
            .filter(user::Column::Username.starts_with(username))
            .count(db)
            .await?;

        let last = ((entries as f64) / (page_size as f64)).ceil() as u64;

        Ok(Pagination {
            entries,
            last,
            ..Default::default()
        })
    }

    pub async fn find_voted_user_by_address(
        db: &DatabaseConnection,
        address: &String,
    ) -> ModelResult<Option<user::Model>> {
        let user = user::Entity::find()
            .join(JoinType::Join, user::Relation::Voter.def())
            .filter(voter::Column::Address.eq(address))
            .one(db)
            .await?;

        Ok(user)
    }
}
