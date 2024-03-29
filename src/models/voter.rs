use loco_rs::model::{ModelError, ModelResult};
use sea_orm::{entity::prelude::*, ActiveValue, TransactionTrait};

use super::_entities::voter::{self, ActiveModel};

impl ActiveModelBehavior for ActiveModel {
    // extend activemodel below (keep comment for generators)
}

#[derive(thiserror::Error, Debug)]
pub enum VoterError {
    #[error("Already voted")]
    AlreadyVoted,

    #[error(transparent)]
    ModelError(#[from] ModelError),
}

#[derive(thiserror::Error, Debug)]
pub enum DeleteVoterError {
    #[error("Voter not found")]
    NotFound,

    #[error(transparent)]
    ModelError(#[from] ModelError),
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

    /// Adds a new voter to the db
    pub async fn add(
        db: &DatabaseConnection,
        address: &str,
        voted_user_id: i32,
    ) -> Result<Self, VoterError> {
        let txn = db.begin().await.map_err(ModelError::from)?;

        if voter::Entity::find()
            .filter(voter::Column::Address.eq(address))
            .one(&txn)
            .await
            .map_err(ModelError::from)?
            .is_some()
        {
            return Err(VoterError::AlreadyVoted);
        }

        let voter = voter::ActiveModel {
            address: ActiveValue::set(address.to_string()),
            voted_user_id: ActiveValue::set(voted_user_id),
            ..Default::default()
        }
        .insert(&txn)
        .await
        .map_err(ModelError::from)?;

        txn.commit().await.map_err(ModelError::from)?;

        Ok(voter)
    }

    /// Deletes a voter from the db
    pub async fn delete(db: &DatabaseConnection, address: &str) -> Result<(), DeleteVoterError> {
        let voter = voter::Entity::find()
            .filter(voter::Column::Address.eq(address))
            .one(db)
            .await
            .map_err(ModelError::from)?
            .ok_or(DeleteVoterError::NotFound)?;

        voter.delete(db).await.map_err(ModelError::from)?;

        Ok(())
    }
}
