pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;
mod m20240505_025740_more_types;
mod m20240507_082145_search_history;
mod m20240512_232002_manifest;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20240505_025740_more_types::Migration),
            Box::new(m20240507_082145_search_history::Migration),
            Box::new(m20240512_232002_manifest::Migration),
        ]
    }
}
