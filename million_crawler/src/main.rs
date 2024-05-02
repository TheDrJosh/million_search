use std::time::Duration;

use exponential_backoff::Backoff;
use proto::{
    crawler::{crawler_client::CrawlerClient, GetJobRequest, GetJobResponse, ReturnJobRequest},
    tonic::{transport::Channel, Code, Status},
};
use tracing::info;

use crate::selector_set::SelectorSet;

mod selector_set;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // TODO: Make config

    tracing_subscriber::fmt().init();

    let mut client = CrawlerClient::connect("http://localhost:8080").await?;

    loop {
        do_job(&mut client).await?;
    }
}

async fn do_job(client: &mut CrawlerClient<Channel>) -> anyhow::Result<()> {
    let job = get_job(client).await?;

    info!("Crawling {}", job.url);

    let res = reqwest::get(job.url.clone()).await?.error_for_status()?;

    let headers = res.headers();

    let mime_type = headers
        .get("Content-Type")
        .map(|mt| mt.to_str().ok().map(|mt| mt.to_owned()))
        .flatten()
        .unwrap_or_default();

    if !(mime_type.is_empty() || mime_type.contains("html")) {
        let ret = ReturnJobRequest {
            id: job.id,
            url: job.url,
            mime_type,
            icon_url: None,
            linked_urls: vec![],
        };

        client.return_job(ret).await?.into_inner();

        return Ok(());
    }

    let text = res.text().await?;

    let html = scraper::Html::parse_document(&text);

    let selector = SelectorSet::new();

    let tags = selector.select(&html, &job.url.parse()?);

    let ret = ReturnJobRequest {
        id: job.id,
        url: job.url,
        mime_type,
        icon_url: None,
        linked_urls: tags.into_iter().map(|url| url.to_string()).collect(),
    };

    client.return_job(ret).await?.into_inner();

    Ok(())
}

async fn get_job(client: &mut CrawlerClient<Channel>) -> Result<GetJobResponse, Status> {
    let backoff = Backoff::new(
        128,
        Duration::from_millis(100),
        Some(Duration::from_secs(10 * 60)),
    );

    for duration in &backoff {
        match client.get_job(GetJobRequest {}).await {
            Ok(res) => {
                let res = res.into_inner();

                return Ok(res);
            }
            Err(err) if err.code() == Code::ResourceExhausted => {
                info!("Waiting for {} seconds", duration.as_secs_f32());
                tokio::time::sleep(duration).await;
            }
            Err(err) => return Err(err),
        }
    }

    return Err(Status::unavailable("cant get job from server"));
}
