use std::time::Duration;

use exponential_backoff::Backoff;
use proto::{
    crawler::{
        crawler_client::CrawlerClient,
        return_job_request::{self},
        GetJobRequest, GetJobResponse, ReturnJobRequest,
    },
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
        let job = get_job(&mut client).await?;
        info!("Crawling {}", job.url);

        match do_job(&job).await {
            Ok(res) => {
                let ret = ReturnJobRequest {
                    id: job.id,
                    url: job.url,
                    result: Some(return_job_request::Result::Ok(res)),
                };

                let _ = client.return_job(ret).await;
            }
            Err(_) => {
                let ret = ReturnJobRequest {
                    id: job.id,
                    url: job.url,
                    result: Some(return_job_request::Result::Err(return_job_request::Err {})),
                };

                let _ = client.return_job(ret).await;
            }
        }
    }
}

async fn do_job(job: &GetJobResponse) -> anyhow::Result<return_job_request::Ok> {
    let res = reqwest::get(job.url.clone()).await?;
    let status = res.status();

    let headers = res.headers();

    let mime_type = headers
        .get("Content-Type")
        .map(|mt| mt.to_str().ok().map(|mt| mt.to_owned()))
        .flatten()
        .unwrap_or_default();

    if !(mime_type.is_empty() || mime_type.contains("html")) {
        return Ok(return_job_request::Ok {
            status: status.as_u16() as i32,
            mime_type,
            linked_urls: vec![],
            body: None,
        });
    }

    let text = res.text().await?;

    let html = scraper::Html::parse_document(&text);

    //TODO: Make Global
    let selector = SelectorSet::new();

    let job_url = job.url.parse()?;

    let urls = selector.select_urls(&html, &job_url);

    let ret = return_job_request::Ok {
        status: status.as_u16() as i32,
        mime_type,
        linked_urls: urls.into_iter().map(|url| url.to_string()).collect(),
        body: Some(return_job_request::ok::Body::Html(
            return_job_request::ok::Html {
                title: selector.select_title(&html),
                description: selector.select_description(&html),
                icon_url: selector
                    .select_icon_url(&html, &job_url)
                    .map(|url| url.to_string()),
                text_fields: selector.select_text_fields(&html),
                sections: selector.select_sections(&html),
            },
        )),
    };

    Ok(ret)
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
