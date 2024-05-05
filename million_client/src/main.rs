use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    sync::Arc,
    time::Duration,
};

use axum::{extract::State, http::StatusCode, routing::get, Form, Router};
use clap::Parser;
use maud::Markup;
use proto::search::search_client::SearchClient;
use serde::Deserialize;
use tokio::sync::Mutex;
use tonic::transport::Channel;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = String::from("http://localhost:8080"))]
    endpoint: String,
    #[arg(short, long, default_value_t = 3000)]
    port: u16,
}

struct AppState {
    client: Mutex<SearchClient<Channel>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

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
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(SocketAddr::V4(SocketAddrV4::new(
        Ipv4Addr::new(0, 0, 0, 0),
        args.port,
    )))
    .await?;

    axum::serve(listener, app).await?;

    Ok(())
}

enum SearchType {
    Html,
    Image,
    Video,
    Audio,
}

async fn home_search_html() -> Result<Markup, StatusCode> {
    home_search(SearchType::Html).await
}
async fn home_search_image() -> Result<Markup, StatusCode> {
    home_search(SearchType::Image).await
}
async fn home_search_video() -> Result<Markup, StatusCode> {
    home_search(SearchType::Video).await
}
async fn home_search_audio() -> Result<Markup, StatusCode> {
    home_search(SearchType::Audio).await
}

async fn home_search(search_type: SearchType) -> Result<Markup, StatusCode> {
    todo!()
}

#[derive(Deserialize)]
struct SearchQuery {
    query: String,
    extra: Option<ExtraSearchQuery>,
}
#[derive(Deserialize)]
enum ExtraSearchQuery {
    Image {
        width: u32,
        height: u32,
    },
    Video {
        width: u32,
        height: u32,
        length: Duration,
    },
    Audio {
        length: Duration,
    },
}
async fn search_html(
    State(state): State<Arc<AppState>>,
    Form(query): Form<SearchQuery>,
) -> Result<Markup, StatusCode> {
    match query.extra {
        None => search(SearchType::Html, query, state).await,
        Some(_) => Err(StatusCode::BAD_REQUEST),
    }
}
async fn search_image(
    State(state): State<Arc<AppState>>,
    Form(query): Form<SearchQuery>,
) -> Result<Markup, StatusCode> {
    match query.extra {
        Some(ExtraSearchQuery::Image {
            width: _,
            height: _,
        })
        | None => search(SearchType::Html, query, state).await,
        Some(_) => Err(StatusCode::BAD_REQUEST),
    }
}
async fn search_video(
    State(state): State<Arc<AppState>>,
    Form(query): Form<SearchQuery>,
) -> Result<Markup, StatusCode> {
    match query.extra {
        Some(ExtraSearchQuery::Video {
            width: _,
            height: _,
            length: _,
        })
        | None => search(SearchType::Html, query, state).await,
        Some(_) => Err(StatusCode::BAD_REQUEST),
    }
}
async fn search_audio(
    State(state): State<Arc<AppState>>,
    Form(query): Form<SearchQuery>,
) -> Result<Markup, StatusCode> {
    match query.extra {
        Some(ExtraSearchQuery::Audio { length: _ }) | None => {
            search(SearchType::Html, query, state).await
        }
        Some(_) => Err(StatusCode::BAD_REQUEST),
    }
}

async fn search(
    search_type: SearchType,
    query: SearchQuery,
    state: Arc<AppState>,
) -> Result<Markup, StatusCode> {
    todo!()
}
