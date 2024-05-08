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
        "http://meilisearch:7700",
        Some("-r_i6i4t88jTzlWtNIyVr0VybDBdn2it428fxr2Blcg"),
    );

    //TODO - Set Searchable feilds

    search_client
        .index("websites")
        .set_searchable_attributes(["url", "title", "description", "sections", "text_fields"])
        .await?;

    search_client
        .index("image")
        .set_searchable_attributes(["url"])
        .await?;

    search_client
        .index("video")
        .set_searchable_attributes(["url"])
        .await?;

    search_client
        .index("audio")
        .set_searchable_attributes(["url"])
        .await?;

    search_client
        .index("search_history")
        .set_searchable_attributes(["text"])
        .await?;

    // Make grpc endpoint

    let addr = "0.0.0.0:8080".parse()?;

    let search_servise = SearchServise {
        db: db.clone(),
        search_client: search_client.clone(),
    };
    let crawler_servise = CrawlerServise {
        db: db.clone(),
        search_client,
    };
    let admin_servise = AdminServise { db };

    println!("Starting");

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
