use admin::AdminServise;
use crawler::CrawlerServise;
use meilisearch_sdk::Client;
use migration::{Migrator, MigratorTrait};
use proto::tonic::transport::Server;
use sea_orm::Database;
use search::SearchServise;

mod admin;
mod crawler;
mod search;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Connect to database

    let db =
        Database::connect("postgres://million_search:password1234@database/million_search").await?;

    Migrator::up(&db, None).await?;

    // Connect to meilisearch

    let search_client = Client::new(
        "http://search:7700",
        Some("HVIWYFQm8QVl4IcAViNjGMdqbC4tQbGbk2jtpfUqL9Y"),
    );

    // Make grpc endpoint

    let addr = "0.0.0.0:8080".parse()?;

    let search_servise = SearchServise {};
    let crawler_servise = CrawlerServise {
        db: db.clone(),
        search_client,
    };
    let admin_servise = AdminServise { db };

    Server::builder()
        .add_service(proto::search::search_server::SearchServer::new(
            search_servise,
        ))
        .add_service(proto::crawler::crawler_server::CrawlerServer::new(
            crawler_servise,
        ))
        .add_service(proto::admin::admin_server::AdminServer::new(admin_servise))
        .serve(addr)
        .await?;

    Ok(())
}
