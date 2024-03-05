use crate::views::leaderboard::Pagination;

use super::_entities::user::{self, ActiveModel};
use loco_rs::model::ModelResult;
use sea_orm::{entity::prelude::*, ActiveValue, Order, QueryOrder, QuerySelect, TransactionTrait};

impl ActiveModelBehavior for ActiveModel {
    // extend activemodel below (keep comment for generators)
}

impl super::_entities::user::Model {
    pub async fn add(db: &DatabaseConnection, username: &str) -> ModelResult<Self> {
        let txn = db.begin().await?;

        let user_qry = user::Entity::find()
            .filter(user::Column::Username.eq(username))
            .one(&txn)
            .await?;

        if let Some(user) = user_qry {
            let mut existing_user: user::ActiveModel = user.into();
            existing_user.votes = ActiveValue::set(existing_user.votes.unwrap() + 1);
            let existing_user: user::Model = existing_user.update(&txn).await?;
            txn.commit().await?;
            return Ok(existing_user);
        }

        // If the user doesn't exist, create a new user with 1 vote
        let new_user = user::ActiveModel {
            username: ActiveValue::set(username.to_string()),
            votes: ActiveValue::set(1),
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
    ) -> ModelResult<Vec<user::Model>> {
        let mut users = user::Entity::find()
            .order_by(user::Column::Votes, Order::Desc)
            .limit(count)
            .offset((page - 1) * count);

        if let Some(username) = username {
            users = users.filter(user::Column::Username.starts_with(username))
        }

        Ok(users.all(db).await?)
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
}
