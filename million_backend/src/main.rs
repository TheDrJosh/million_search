use admin::AdminServise;

use crawler::CrawlerServise;
use sea_orm::Database;
use search::SearchServise;

use tonic::transport::Server;

mod admin;
mod crawler;
mod search;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Connect to database

    let db = Database::connect("postgres://million_search:password1234@database/million_search").await?;

    // Make grpc endpoint

    let addr = "0.0.0.0:8080".parse()?;

    let search_servise = SearchServise::default();
    let crawler_servise = CrawlerServise::default();
    let admin_servise = AdminServise { };

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
