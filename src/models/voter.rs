use super::_entities::voter::{self, ActiveModel};
use loco_rs::model::{ModelError, ModelResult};
use sea_orm::{entity::prelude::*, ActiveValue, TransactionTrait};

impl ActiveModelBehavior for ActiveModel {
    // extend activemodel below (keep comment for generators)
}

impl super::_entities::voter::Model {
    /// finds a voter by ip address
    ///
    /// # Errors
    ///
    /// When could not find user by the given address or DB query error
    pub async fn find_by_address(db: &DatabaseConnection, address: &str) -> ModelResult<Self> {
        let voter = voter::Entity::find()
            .filter(voter::Column::Address.eq(address))
            .one(db)
            .await?;
        voter.ok_or_else(|| ModelError::EntityNotFound)
    }

    pub async fn add(db: &DatabaseConnection, address: &str) -> ModelResult<Self> {
        let txn = db.begin().await?;

        if voter::Entity::find()
            .filter(voter::Column::Address.eq(address))
            .one(&txn)
            .await?
            .is_some()
        {
            return Err(ModelError::EntityAlreadyExists {});
        }

        let voter = voter::ActiveModel {
            address: ActiveValue::set(address.to_string()),
            ..Default::default()
        }
        .insert(&txn)
        .await?;

        txn.commit().await?;

        Ok(voter)
    }
}
