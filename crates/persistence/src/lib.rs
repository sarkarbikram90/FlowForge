pub mod memory;
pub mod postgres;
pub mod repository;

pub use memory::InMemoryDatabase;
pub use postgres::PostgresDatabase;
pub use repository::{OutboxRecord, Repository};
