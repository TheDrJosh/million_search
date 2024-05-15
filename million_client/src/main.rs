use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    sync::Arc,
};

use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Form, Router,
};
use clap::Parser;
use home::home_search_page;
use maud::Markup;
use proto::search::search_client::SearchClient;
use search::{search_page, search_page_results};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tonic::transport::Channel;
use tower_http::services::ServeDir;
use tracing_subscriber::EnvFilter;
use utils::search_suggestions;
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
        .route("/search", get(search_html).post(search_html_results))
        .route(
            "/image/search",
            get(search_image).post(search_image_results),
        )
        .route("/search-suggestions", post(search_suggestions))
        .nest_service("/public", ServeDir::new("public"))
        .with_state(state);

    // let app = app.layer(LiveReloadLayer::new().reload_interval(Duration::from_millis(200)));

    let listener = tokio::net::TcpListener::bind(SocketAddr::V4(SocketAddrV4::new(
        Ipv4Addr::new(0, 0, 0, 0),
        args.port,
    )))
    .await?;

    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
enum SearchType {
    Html,
    Image,
}

async fn home_search_html() -> Result<Markup, StatusCode> {
    home_search_page(SearchType::Html).await
}
async fn home_search_image() -> Result<Markup, StatusCode> {
    home_search_page(SearchType::Image).await
}

#[derive(Deserialize, Serialize)]
struct SearchQuery {
    query: String,
    extra: Option<ExtraSearchQuery>,
}
#[derive(Deserialize, Serialize)]
enum ExtraSearchQuery {
    Image { size: Size },
}

#[derive(Deserialize, Serialize)]
struct SearchQueryList {
    query: String,
    page: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    extra: Option<ExtraSearchQuery>,
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
    match Option::<ExtraSearchQuery>::None {
        None => search_page(SearchType::Html, query, state).await,
        Some(_) => Err(StatusCode::BAD_REQUEST),
    }
}
async fn search_image(
    State(state): State<Arc<AppState>>,
    Form(query): Form<SearchQuery>,
) -> Result<Markup, StatusCode> {
    match Option::<ExtraSearchQuery>::None {
        Some(ExtraSearchQuery::Image { size: _ }) | None => {
            search_page(SearchType::Image, query, state).await
        } // Some(_) => Err(StatusCode::BAD_REQUEST),
    }
}

async fn search_html_results(
    State(state): State<Arc<AppState>>,
    Form(query): Form<SearchQueryList>,
) -> Result<Markup, (StatusCode, String)> {
    match query.extra {
        None => search_page_results(SearchType::Html, query, state).await,
        Some(_) => Err((
            StatusCode::BAD_REQUEST,
            String::from("incorrect query params for search type"),
        )),
    }
}
async fn search_image_results(
    State(state): State<Arc<AppState>>,
    Form(query): Form<SearchQueryList>,
) -> Result<Markup, (StatusCode, String)> {
    match query.extra {
        Some(ExtraSearchQuery::Image { size: _ }) | None => {
            search_page_results(SearchType::Image, query, state).await
        } // Some(_) => Err((
          //     StatusCode::BAD_REQUEST,
          //     String::from("incorrect query params for search type"),
          // )),
    }
}
