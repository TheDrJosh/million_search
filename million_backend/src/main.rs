use admin::AdminServise;
use amqprs::{
    channel::{BasicPublishArguments, QueueDeclareArguments},
    connection::{Connection, OpenConnectionArguments},
    BasicProperties,
};
use crawler::CrawlerServise;
use search::SearchServise;
use tonic::transport::Server;

mod admin;
mod crawler;
mod search;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Connect to queue

    let connection = Connection::open(&OpenConnectionArguments::new(
        "0.0.0.0", 5672, "username", "password",
    ))
    .await?;

    let channel = connection.open_channel(None).await?;

    let (queue_name, _message_count, _consumer_count) = channel
        .queue_declare(QueueDeclareArguments::default())
        .await?
        .unwrap();

    channel.basic_publish(
        BasicProperties::default(),
        "Hello World".as_bytes().into_iter().map(|i| *i).collect::<Vec<u8>>(),
        BasicPublishArguments::default()
            .routing_key("hello".to_string())
            .finish(),
    ).await?;

    // Connect to database

    // Make grpc endpoint
    let addr = "[::1]:8080".parse()?;

    let search_servise = SearchServise::default();
    let crawler_servise = CrawlerServise::default();
    let admin_servise = AdminServise::default();

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
