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
    tonic::{codec::CompressionEncoding, transport::Channel, Code, Status},
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
    #[arg(short, long, env, default_value_t = String::from("http://localhost:8080"))]
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
    let mut client = CrawlerClient::connect(args.endpoint)
        .await?
        .send_compressed(CompressionEncoding::Zstd)
        .accept_compressed(CompressionEncoding::Zstd);

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
        .and_then(|mt| mt.to_str().ok().map(|mt| mt.to_owned()))
        .unwrap_or_default();

    Ok(if mime_type.is_empty() || mime_type.contains("html") {
        let text = res.text().await?;

        let job_url: Url = job.url.parse()?;

        let job_url_other = job_url.clone();

        let (html, urls, manifest_url, keywords) = spawn_blocking(move || {
            let html = scraper::Html::parse_document(&text);
            let urls = SELECTOR.select_urls(&html, &job_url_other);
            let manifest_url = SELECTOR.select_manifest_url(&html, &job_url_other);
            let keywords = SELECTOR.select_keywords(&html);
            (html, urls, manifest_url, keywords)
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
                // let svg_text = String::from_utf8(img_bytes.to_vec()).ok();

                let img = spawn_blocking(move || {
                    image::io::Reader::new(Cursor::new(&img_bytes))
                        .with_guessed_format()
                        .ok()
                        .and_then(|img| img.decode().ok())
                })
                .await?;

                let size = img
                    .as_ref()
                    .map(|img| return_job_request::ok::body::image::Size {
                        width: img.width() as i32,
                        height: img.height() as i32,
                    });

                // let luminance_range = spawn_blocking(move || {
                //     img.as_ref()
                //         .and_then(get_luminance_range_image)
                //         .or_else(|| {
                //             svg_text
                //                 .as_deref()
                //                 .and_then(get_luminance_range_svg)
                //         })
                // })
                // .await?;

                anyhow::Result::Ok(return_job_request::ok::body::Image {
                    image_url: image_url.to_string(),
                    size,
                    alt_text: image_alt_text,
                    // luminance_range,
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
                keywords,
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

    Err(Status::unavailable("cant get job from server"))
}

#[derive(Debug, Deserialize)]
struct Manifest {
    name: Option<String>,
    short_name: Option<String>,
    description: Option<String>,
    categories: Option<Vec<String>>,
}

// fn get_luminance_range_image(img: &DynamicImage) -> Option<LuminanceRange> {
//     let mut min_luminance = f32::INFINITY;
//     let mut max_luminance = -f32::INFINITY;
//     let mut contact = false;

//     for j in 0..img.height() {
//         for i in 0..img.width() {
//             let p = img.get_pixel(i, j);

//             if p[3] > 127 {
//                 contact = true;
//                 let luminance = get_luminance(
//                     p[0] as f32 / 255f32,
//                     p[1] as f32 / 255f32,
//                     p[2] as f32 / 255f32,
//                 );
//                 min_luminance = f32::min(min_luminance, luminance);
//                 max_luminance = f32::max(max_luminance, luminance);
//             }
//         }
//     }

//     if contact {
//         Some(LuminanceRange {
//             min: min_luminance,
//             max: max_luminance,
//         })
//     } else {
//         None
//     }
// }

// fn get_luminance_range_svg(img: &str) -> Option<LuminanceRange> {
//     let parsed_svg = resvg::usvg::Tree::from_str(
//         img,
//         &resvg::usvg::Options::default(),
//         &resvg::usvg::fontdb::Database::default(),
//     )
//     .ok()?;

//     let mut pix_map = resvg::tiny_skia::Pixmap::new(1024, 1024).unwrap();

//     resvg::render(
//         &parsed_svg,
//         resvg::usvg::Transform::default(),
//         &mut pix_map.as_mut(),
//     );

//     let img = image::RgbaImage::from_vec(1024, 1024, pix_map.data().to_vec()).unwrap();

//     img.save("test.png").unwrap();

//     get_luminance_range_image(&img.into())
// }

// fn get_luminance(r: f32, g: f32, b: f32) -> f32 {
//     let r = srgb_to_linear(r);
//     let g = srgb_to_linear(g);
//     let b = srgb_to_linear(b);

//     0.2126 * r + 0.7152 * g + 0.0722 * b
// }

// fn srgb_to_linear(c: f32) -> f32 {
//     if c <= 0.04045 {
//         c / 12.92
//     } else {
//         f32::powf((c + 0.055) / 1.055, 2.4)
//     }
// }
