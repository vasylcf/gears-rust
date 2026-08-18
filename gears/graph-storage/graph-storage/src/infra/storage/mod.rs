//! Storage layer: entities, migrations and repositories.
//!
//! All `SeaORM` specifics are confined here; the domain layer sees only ports.

pub mod counts;
pub mod entity;
pub mod ingest_repo;
pub mod migrations;
pub mod traversal;
pub mod traversal_experiment;
