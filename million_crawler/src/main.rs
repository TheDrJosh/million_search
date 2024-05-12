use std::time::Duration;

use clap::Parser;
use exponential_backoff::Backoff;
use lazy_static::lazy_static;
use proto::{
    crawler::{
        crawler_client::CrawlerClient,
        return_job_request::{self},
        GetJobRequest, GetJobResponse, ReturnJobRequest,
    },
    tonic::{transport::Channel, Code, Status},
};
use serde::Deserialize;
use tracing::info;

use crate::selector_set::SelectorSet;

mod selector_set;

lazy_static! {
    static ref SELECTOR: SelectorSet = SelectorSet::new();
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = String::from("http://localhost:8080"))]
    endpoint: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    let args = Args::parse();

    let mut client = CrawlerClient::connect(args.endpoint).await?;

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
    let res = reqwest::get(&job.url).await?;
    let status = res.status();

    let headers = res.headers();

    let mime_type = headers
        .get("Content-Type")
        .map(|mt| mt.to_str().ok().map(|mt| mt.to_owned()))
        .flatten()
        .unwrap_or_default();

    Ok(if mime_type.is_empty() || mime_type.contains("html") {
        let text = res.text().await?;

        let html = scraper::Html::parse_document(&text);

        let job_url = job.url.parse()?;

        let urls = SELECTOR.select_urls(&html, &job_url);

        return_job_request::Ok {
            status: status.as_u16() as i32,
            mime_type,
            linked_urls: urls.into_iter().map(|url| url.to_string()).collect(),
            body: Some(return_job_request::ok::Body::Html(
                return_job_request::ok::Html {
                    title: SELECTOR.select_title(&html),
                    description: SELECTOR.select_description(&html),
                    icon_url: SELECTOR
                        .select_icon_url(&html, &job_url)
                        .map(|url| url.to_string()),
                    text_fields: SELECTOR.select_text_fields(&html),
                    sections: SELECTOR.select_sections(&html),
                },
            )),
        }
    } else if mime_type.starts_with("image/") {
        return_job_request::Ok {
            status: status.as_u16() as i32,
            mime_type,
            linked_urls: vec![],
            body: Some(return_job_request::ok::Body::Image(
                return_job_request::ok::Image { size: None },
            )),
        }
    } else if mime_type.starts_with("video/") {
        return_job_request::Ok {
            status: status.as_u16() as i32,
            mime_type,
            linked_urls: vec![],
            body: Some(return_job_request::ok::Body::Video(
                return_job_request::ok::Video {
                    size: None,
                    length: None,
                },
            )),
        }
    } else if mime_type.starts_with("audio/") {
        return_job_request::Ok {
            status: status.as_u16() as i32,
            mime_type,
            linked_urls: vec![],
            body: Some(return_job_request::ok::Body::Audio(
                return_job_request::ok::Audio { length: None },
            )),
        }
    } else if mime_type == "application/manifest+json" {
        let text = res.text().await?;

        let manifest: Manifest = serde_json::from_str(&text)?;

        return_job_request::Ok {
            status: status.as_u16() as i32,
            mime_type,
            linked_urls: vec![],
            body: Some(return_job_request::ok::Body::Manifest(
                return_job_request::ok::Manifest {
                    categories: manifest.categories,
                    description: manifest.description,
                    name: manifest.name,
                    short_name: manifest.short_name,
                },
            )),
        }
    } else {
        return_job_request::Ok {
            status: status.as_u16() as i32,
            mime_type,
            linked_urls: vec![],
            body: None,
        }
    })
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

#[derive(Debug, Deserialize)]
struct Manifest {
    name: Option<String>,
    short_name: Option<String>,
    description: Option<String>,
    categories: Vec<String>,
}
