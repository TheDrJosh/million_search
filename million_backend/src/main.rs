use admin::AdminServise;

use amqprs::{
    callbacks::{DefaultChannelCallback, DefaultConnectionCallback},
    channel::{QueueBindArguments, QueueDeclareArguments},
    connection::{Connection, OpenConnectionArguments},
};
use crawler::CrawlerServise;
use search::SearchServise;

use tonic::transport::Server;

mod admin;
mod crawler;
mod search;

const ROUTING_KEY: &str = "million.backend";
const EXCHANGE_NAME: &str = "milion.tasks";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Connect to queue

    // let mut connection = Connection::insecure_open("amqp://guest:guest@rabbitmq:5672")?;

    // let channel = connection.open_channel(None)?;

    // channel.queue_declare(
    //     TASK_QUEUE,
    //     QueueDeclareOptions {
    //         durable: true,
    //         ..Default::default()
    //     },
    // )?;

    // let exchange = Exchange::direct(&channel);

    let connection = Connection::open(&OpenConnectionArguments::new(
        "rabbitmq", 5672, "guest", "guest",
    ))
    .await?;

    connection
        .register_callback(DefaultConnectionCallback)
        .await?;

    let channel = connection.open_channel(None).await?;

    channel.register_callback(DefaultChannelCallback).await?;

    let (queue_name, _, _) = channel
        .queue_declare(QueueDeclareArguments::default())
        .await?
        .unwrap();

    channel
        .queue_bind(QueueBindArguments::new(
            &queue_name,
            &EXCHANGE_NAME,
            &ROUTING_KEY,
        ))
        .await?;

    // Connect to database

    // Make grpc endpoint

    let addr = "0.0.0.0:8080".parse()?;

    let search_servise = SearchServise::default();
    let crawler_servise = CrawlerServise::default();
    let admin_servise = AdminServise { channel };

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
