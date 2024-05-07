use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    sync::Arc,
    time::Duration,
};

use axum::{extract::State, http::StatusCode, routing::get, Form, Router};
use clap::Parser;
use home::home_search_page;
use maud::Markup;
use proto::search::search_client::SearchClient;
use search::search_page;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tonic::transport::Channel;
use tower_http::services::ServeDir;
use tower_livereload::LiveReloadLayer;
use tracing_subscriber::EnvFilter;
mod home;
mod search;
mod utils;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = String::from("http://backend:8080"))]
    endpoint: String,
    #[arg(short, long, default_value_t = 3000)]
    port: u16,
}

struct AppState {
    client: Mutex<SearchClient<Channel>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let args = Args::parse();

    let client = SearchClient::connect(args.endpoint).await?;

    let state = Arc::new(AppState {
        client: Mutex::new(client),
    });
 
    let app = Router::new() 
        .route("/", get(home_search_html))
        .route("/image", get(home_search_image))
        .route("/video", get(home_search_video))
        .route("/audio", get(home_search_audio))
        .route("/search", get(search_html))
        .route("/image/search", get(search_image))
        .route("/video/search", get(search_video))
        .route("/audio/search", get(search_audio))
        .nest_service("/public", ServeDir::new("public"))
        .with_state(state);

    let app = app.layer(LiveReloadLayer::new().reload_interval(Duration::from_millis(200)));

    let listener = tokio::net::TcpListener::bind(SocketAddr::V4(SocketAddrV4::new(
        Ipv4Addr::new(0, 0, 0, 0),
        args.port,
    )))
    .await?;

    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Debug, PartialEq)]
enum SearchType {
    Html,
    Image,
    Video,
    Audio,
}

async fn home_search_html() -> Result<Markup, StatusCode> {
    home_search_page(SearchType::Html).await
}
async fn home_search_image() -> Result<Markup, StatusCode> {
    home_search_page(SearchType::Image).await
}
async fn home_search_video() -> Result<Markup, StatusCode> {
    home_search_page(SearchType::Video).await
}
async fn home_search_audio() -> Result<Markup, StatusCode> {
    home_search_page(SearchType::Audio).await
}

fn default_search_length() -> u32 {
    20
}

#[derive(Deserialize, Serialize)]
struct SearchQuery {
    query: String,
    #[serde(default)]
    start: u32,
    #[serde(default = "default_search_length")]
    length: u32,
    #[serde(default)]
    extra: Option<ExtraSearchQuery>,
}
#[derive(Deserialize, Serialize)]
enum ExtraSearchQuery {
    Image { size: Size },
    Video { size: Size, length: Duration },
    Audio { length: Duration },
}

#[derive(Deserialize, Serialize)]
struct Size {
    width: u32,
    height: u32,
}

async fn search_html(
    State(state): State<Arc<AppState>>,
    Form(query): Form<SearchQuery>,
) -> Result<Markup, StatusCode> {
    match query.extra {
        None => search_page(SearchType::Html, query, state).await,
        Some(_) => Err(StatusCode::BAD_REQUEST),
    }
}
async fn search_image(
    State(state): State<Arc<AppState>>,
    Form(query): Form<SearchQuery>,
) -> Result<Markup, StatusCode> {
    match query.extra {
        Some(ExtraSearchQuery::Image { size: _ }) | None => {
            search_page(SearchType::Image, query, state).await
        }
        Some(_) => Err(StatusCode::BAD_REQUEST),
    }
}
async fn search_video(
    State(state): State<Arc<AppState>>,
    Form(query): Form<SearchQuery>,
) -> Result<Markup, StatusCode> {
    match query.extra {
        Some(ExtraSearchQuery::Video { size: _, length: _ }) | None => {
            search_page(SearchType::Video, query, state).await
        }
        Some(_) => Err(StatusCode::BAD_REQUEST),
    }
}
async fn search_audio(
    State(state): State<Arc<AppState>>,
    Form(query): Form<SearchQuery>,
) -> Result<Markup, StatusCode> {
    match query.extra {
        Some(ExtraSearchQuery::Audio { length: _ }) | None => {
            search_page(SearchType::Audio, query, state).await
        }
        Some(_) => Err(StatusCode::BAD_REQUEST),
    }
}
