#![allow(elided_lifetimes_in_paths)]
#![allow(clippy::wildcard_imports)]
pub use sea_orm_migration::prelude::*;

mod m20240301_000001_users;

mod m20240302_125640_voter;
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240301_000001_users::Migration),
            Box::new(m20240302_125640_voter::Migration),
        ]
    }
}
