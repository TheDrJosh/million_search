use std::sync::Arc;

use axum::http::StatusCode;
use maud::Markup;

use crate::{AppState, SearchQuery, SearchType};

pub async fn search_page(
    search_type: SearchType,
    query: SearchQuery,
    state: Arc<AppState>,
) -> Result<Markup, StatusCode> {
    todo!()
}
