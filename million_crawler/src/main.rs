use std::{io::Cursor, time::Duration};

use clap::Parser;
use exponential_backoff::Backoff;
use futures::future::join_all;
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
use tokio::task::spawn_blocking;
use tracing::info;
use url::Url;

use crate::selector_set::SelectorSet;

mod selector_set;

lazy_static! {
    static ref SELECTOR: SelectorSet = SelectorSet::new();
}

#[derive(Parser, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = String::from("http://localhost:8080"))]
    endpoint: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    let args = Args::parse();

    tokio::select! {
        _ = run_many(args, 1) => {}
        _ = tokio::time::sleep(Duration::from_secs(60 * 30)) => {}
    }

    Ok(())
}

async fn run_many(args: Args, parallel_tasks: usize) -> anyhow::Result<()> {
    let mut tasks = Vec::new();

    for _ in 0..parallel_tasks {
        tasks.push(run(args.clone()));
    }

    join_all(tasks.into_iter())
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;

    Ok(())
}

async fn run(args: Args) -> anyhow::Result<()> {
    let mut client = CrawlerClient::connect(args.endpoint).await?;

    loop {
        let job = get_job(&mut client).await?;
        info!("Crawling {}", job.url);

        let start_time = std::time::Instant::now();

        match do_job(&job).await {
            Ok(res) => {
                let ret = ReturnJobRequest {
                    id: job.id,
                    url: job.url.clone(),
                    result: Some(return_job_request::Result::Ok(res)),
                };

                let _ = client.return_job(ret).await;
            }
            Err(err) => {
                let ret = ReturnJobRequest {
                    id: job.id,
                    url: job.url.clone(),
                    result: Some(return_job_request::Result::Err(return_job_request::Err {})),
                };

                let _ = client.return_job(ret).await;

                tracing::error!("Url {} errored with: {}", job.url, err.to_string());
            }
        }

        let end_time = std::time::Instant::now();

        let e_time = (end_time - start_time).as_millis();

        info!(
            "Finished Crawling {} | Finished in {} milliseconds",
            job.url, e_time
        );
    }
}

async fn do_job(job: &GetJobResponse) -> anyhow::Result<return_job_request::Ok> {
    let res = reqwest::get(&job.url).await?.error_for_status()?;
    let status = res.status();

    let headers = res.headers();

    let mime_type = headers
        .get("Content-Type")
        .map(|mt| mt.to_str().ok().map(|mt| mt.to_owned()))
        .flatten()
        .unwrap_or_default();

    Ok(if mime_type.is_empty() || mime_type.contains("html") {
        let text = res.text().await?;

        let job_url: Url = job.url.parse()?;

        let job_url_other = job_url.clone();

        let (html, urls, manifest_url) = spawn_blocking(move || {
            let html = scraper::Html::parse_document(&text);
            let urls = SELECTOR.select_urls(&html, &job_url_other);
            let manifest_url = SELECTOR.select_manifest_url(&html, &job_url_other);
            (html, urls, manifest_url)
        })
        .await?;

        let manifest = if let Some(manifest_url) = manifest_url {
            let manifest_res = reqwest::get(manifest_url).await?.error_for_status()?;

            let text = manifest_res.text().await?;

            let manifest = serde_json::from_str::<Manifest>(&text)?;

            Some(return_job_request::ok::body::Manifest {
                categories: manifest.categories.unwrap_or_default(),
                description: manifest.description,
                name: manifest.name,
                short_name: manifest.short_name,
            })
        } else {
            None
        };

        let images = join_all(SELECTOR.select_images(&html, &job_url).into_iter().map(
            |(image_url, image_alt_text)| async move {
                let img_res = reqwest::get(image_url.clone()).await?;
                let img_bytes = img_res.bytes().await?;

                let size = spawn_blocking(move || {
                    image::io::Reader::new(Cursor::new(img_bytes))
                        .with_guessed_format()
                        .ok()
                        .map(|img| img.decode().ok())
                        .flatten()
                        .map(|img| return_job_request::ok::body::image::Size {
                            width: img.width() as i32,
                            height: img.height() as i32,
                        })
                })
                .await?;

                anyhow::Result::Ok(return_job_request::ok::body::Image {
                    image_url: image_url.to_string(),
                    size,
                    alt_text: image_alt_text,
                })
            },
        ))
        .await
        .into_iter()
        .collect::<anyhow::Result<Vec<_>>>()?;

        return_job_request::Ok {
            status: status.as_u16() as i32,
            mime_type,
            linked_urls: urls.into_iter().map(|url| url.to_string()).collect(),

            body: Some(return_job_request::ok::Body {
                title: SELECTOR.select_title(&html),
                description: SELECTOR.select_description(&html),
                icon_url: SELECTOR
                    .select_icon_url(&html, &job_url)
                    .map(|url| url.to_string()),
                text_fields: SELECTOR.select_text_fields(&html),
                sections: SELECTOR.select_sections(&html),
                manifest,
                images,
            }),
        }
    } else {
        return_job_request::Ok {
            status: status.as_u16() as i32,
            mime_type,
            body: None,
            linked_urls: vec![],
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
    categories: Option<Vec<String>>,
}
