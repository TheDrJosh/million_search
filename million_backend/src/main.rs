use admin::AdminServise;
use crawler::CrawlerServise;
use search::SearchServise;
use tonic::transport::Server;

mod admin;
mod crawler;
mod search;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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
