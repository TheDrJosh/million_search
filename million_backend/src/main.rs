use admin::AdminServise;

use amiquip::{Connection, Exchange};
use crawler::CrawlerServise;
use search::SearchServise;
use tokio::sync::Mutex;
use tonic::transport::Server;

mod admin;
mod crawler;
mod search;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Connect to queue

    let connection = Mutex::new(Connection::insecure_open("amqp://guest:guest@rabbitmq:5672")?);

    // Connect to database

    // Make grpc endpoint

    let addr = "0.0.0.0:8080".parse()?;

    let search_servise = SearchServise::default();
    let crawler_servise = CrawlerServise::default();
    let admin_servise = AdminServise{connection};

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
