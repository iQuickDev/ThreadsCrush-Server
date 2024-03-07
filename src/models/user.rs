use crate::views::leaderboard::Pagination;

use super::_entities::user::{self, ActiveModel};
use super::_entities::voter;
use loco_rs::prelude::*;
use loco_rs::model::{ModelError, ModelResult};
use sea_orm::{entity::prelude::*, ActiveValue, Order, QueryOrder, QuerySelect, TransactionTrait};
use sea_orm::{FromQueryResult, JoinType};


impl ActiveModelBehavior for ActiveModel {
    // extend activemodel below (keep comment for generators)
}

#[derive(FromQueryResult, Debug)]
pub struct UserWithVotes {
    pub votes: i64,
    pub username: String,
}

impl super::_entities::user::Model {
    pub async fn add(db: &DatabaseConnection, username: &str) -> ModelResult<Self> {
        let txn = db.begin().await?;

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
        username: Option<String>,
        page: u64,
        count: u64,
    ) -> ModelResult<Vec<UserWithVotes>> {
        let mut users = user::Entity::find()
            .select_only()
            .column(user::Column::Username)
            .column_as(voter::Column::Id.count(), "votes")
            .join(JoinType::Join, user::Relation::Voter.def())
            .group_by(user::Column::Id)
            .order_by(voter::Column::VotedUserId.count(), Order::Desc)
            .limit(count)
            .offset((page - 1) * count);

        if let Some(username) = username {
            users = users.filter(user::Column::Username.contains(username))
        }

        let users = users.into_model::<UserWithVotes>().all(db).await?;

        Ok(users)
    }

    pub async fn get_leaderboard_pagination(
        db: &DatabaseConnection,
        page_size: u64,
    ) -> ModelResult<Pagination> {
        let entries = user::Entity::find().count(db).await?;
        let last = ((entries as f64) / (page_size as f64)).ceil() as u64;

        Ok(Pagination {
            entries,
            last,
            ..Default::default()
        })
    }

    pub async fn get_id(db: &DatabaseConnection, username: &String) -> ModelResult<i32> {
        let user = voter::Entity::find()
            .filter(voter::Column::Address.eq(username))
            .one(db)
            .await?;
        
        match user {
            Some(user) => Ok(user.id),
            None => Err(ModelError::EntityNotFound),
        }
    }
}
