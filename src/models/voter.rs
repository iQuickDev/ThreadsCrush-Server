use super::_entities::voter::{self, ActiveModel};
use loco_rs::model::{ModelError, ModelResult};
use sea_orm::{entity::prelude::*, ActiveValue, TransactionTrait};

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

    pub async fn add(db: &DatabaseConnection, address: &str) -> Result<Self, VoterError> {
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
            ..Default::default()
        }
        .insert(&txn)
        .await
        .map_err(ModelError::from)?;

        txn.commit().await.map_err(ModelError::from)?;

        Ok(voter)
    }
}
