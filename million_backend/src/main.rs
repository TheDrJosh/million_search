use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use admin::AdminServise;
use clap::Parser;
use crawler::CrawlerServise;
use meilisearch_sdk::client::Client;
use migration::{Migrator, MigratorTrait};
use proto::tonic::transport::Server;
use sea_orm::Database;
use search::SearchServise;
use tracing_subscriber::EnvFilter;
use url::Url;

mod admin;
mod crawler;
mod search;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, env)]
    database_url: Url,

    #[arg(short, long, env)]
    meilisearch_url: Url,

    #[arg(short, long, env, default_value_t = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)))]
    host_address: IpAddr,

    #[arg(short, long, env, default_value_t = 8080)]
    port: u16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let args = Args::parse();

    // Connect to database
    let db = Database::connect(args.database_url).await?;

    Migrator::up(&db, None).await?;

    // Connect to meilisearch
    let search_client = Client::new(args.meilisearch_url, Option::<String>::None)?;

    search_client
        .index("websites")
        .set_searchable_attributes([
            "url",
            "title",
            "description",
            "sections",
            "text_fields",
            "site_name",
            "site_short_name",
            "site_description",
            "site_categories",
        ])
        .await?;

    search_client
        .index("image")
        .set_searchable_attributes(["url", "alt_text", "source_url"])
        .await?;

    search_client
        .index("search_history")
        .set_searchable_attributes(["text"])
        .await?;

    let ranking_rules = [
        "words",
        "typo",
        "proximity",
        "attribute",
        "sort",
        "exactness",
        "count:desc",
    ];

    search_client
        .index("search_history")
        .set_ranking_rules(ranking_rules)
        .await?;

    // Make grpc endpoint

    let addr = SocketAddr::new(args.host_address, args.port);

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
