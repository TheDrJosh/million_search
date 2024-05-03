use admin::AdminServise;
use crawler::CrawlerServise;
use migration::{Migrator, MigratorTrait};
use proto::tonic::transport::Server;
use s3::{creds::Credentials, Bucket, BucketConfiguration, Region};
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

    // Connect to Minio

    let bucket_name = "million_search";
    let region = Region::Custom {
        region: "minio".to_owned(),
        endpoint: "http://object_store:9000".to_owned(),
    };
    let credentials = Credentials::default()?;

    let bucket = Bucket::new(bucket_name, region.clone(), credentials.clone())?.with_path_style();

    // Make grpc endpoint

    let addr = "0.0.0.0:8080".parse()?;

    let search_servise = SearchServise {};
    let crawler_servise = CrawlerServise { db: db.clone(), bucket };
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
