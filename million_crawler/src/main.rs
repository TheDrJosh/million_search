use std::time::Duration;

use backoff::exponential::ExponentialBackoff;
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

    let mime_type = headers.get("Content-Type").unwrap().to_str()?.to_owned();

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

async fn fetch_job(
    client: &mut CrawlerClient<Channel>,
) -> Result<GetJobResponse, backoff::Error<Status>> {
    match client.get_job(GetJobRequest {}).await {
        Ok(res) => {
            let res = res.into_inner();

            Ok(res)
        }
        Err(err) if err.code() == Code::ResourceExhausted => Err(backoff::Error::transient(err)),
        Err(err) => Err(backoff::Error::permanent(err)),
    }
}

async fn get_job(client: &mut CrawlerClient<Channel>) -> anyhow::Result<GetJobResponse> {
    let job = backoff::future::retry(ExponentialBackoff::default(), || async {
        fetch_job(client).await
    })
    .await;

    Ok(job.unwrap())
}
