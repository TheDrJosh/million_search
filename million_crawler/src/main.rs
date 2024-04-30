use std::time::Duration;

use proto::{
    crawler::{crawler_client::CrawlerClient, GetJobRequest, GetJobResponse, ReturnJobRequest},
    tonic::{transport::Channel, Code},
};
use tracing::info;

use crate::selector_set::SelectorSet;

mod selector_set;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // TODO: Make config

    tracing_subscriber::fmt().init();

    let mut client = CrawlerClient::connect("http://localhost:8080").await?;

    info!("Getting Job");

    let job = get_job(&mut client).await?;

    info!("Got Job");

    // let job = GetJobResponse {
    //     id: 0,
    //     url: "https://ziglang.org/".to_owned(),
    // };

    info!("Getting Webpage");

    let res = reqwest::get(job.url.clone()).await?.error_for_status()?;

    info!("Got Webpage");

    let headers = res.headers();

    let mime_type = headers.get("Content-Type").unwrap().to_str()?.to_owned();

    let text = res.text().await?;

    info!("Starting Parse");

    let html = scraper::Html::parse_document(&text);

    info!("Parsed Finished");

    let selector = SelectorSet::new();

    info!("Starting Sellect");

    let tags = selector.select(&html, &job.url.parse()?);

    info!("Sellect Finished");

    let ret = ReturnJobRequest {
        id: job.id,
        url: job.url,
        mime_type: mime_type,
        icon_url: None,
        linked_urls: tags.into_iter().map(|url| url.to_string()).collect(),
    };

    // println!("{:#?}", ret);

    let mut client = CrawlerClient::connect("http://localhost:8080").await?;

    info!("Starting Job Return");

    client.return_job(ret).await?.into_inner();

    info!("Job Return Finished");

    Ok(())
}

async fn get_job(client: &mut CrawlerClient<Channel>) -> anyhow::Result<GetJobResponse> {
    let mut job = None;

    while job.is_none() {
        match client.get_job(GetJobRequest {}).await {
            Ok(res) => {
                let res = res.into_inner();

                job = Some(res);
            }
            Err(err) if err.code() == Code::ResourceExhausted => {
                tokio::time::sleep(Duration::from_secs(10)).await;
            }
            Err(err) => {
                Err(err)?;
            }
        }
    }

    Ok(job.unwrap())
}
