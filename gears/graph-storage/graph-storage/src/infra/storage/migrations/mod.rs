//! Database migrations. Raw SQL is permitted here and nowhere else in the gear.

use sea_orm_migration::prelude::*;

mod m20260818_000001_initial;

/// Migrator handed to the platform by [`DatabaseCapability`](toolkit::DatabaseCapability).
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20260818_000001_initial::Migration)]
    }
}
