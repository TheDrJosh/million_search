use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Form};
use maud::{html, Markup, DOCTYPE};
use proto::search::CompleteSearchRequest;
use serde::{Deserialize, Serialize};

use crate::{AppState, ExtraSearchQuery, SearchQuery, SearchType};

pub fn basic_page(body: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html {
            head {
                title {
                    "Million Search"
                }
                meta charset="UTF-8" {}
                meta name="viewport" content="width=device-width, initial-scale=1.0" {}

                link rel="stylesheet" href="/public/main.css" {}
                script src="https://unpkg.com/htmx.org@1.9.12" integrity="sha384-ujb1lZYygJmzgSwoxRggbCHcjc0rB2XoQrxeTUQyRjrOnlCoYta87iKBWq3EsdM2" crossorigin="anonymous" defer {}
                script src="https://unpkg.com/htmx.org@1.9.12/dist/ext/json-enc.js" defer {}
            }
            body class="dark:bg-zinc-800 dark:text-zinc-50" {
                (body)
            }
        }
    }
}

pub fn search_bar(query: &str, search_type: SearchType) -> anyhow::Result<Markup> {
    let vals = serde_json::to_string(&serde_json::json!({
        "search_type": search_type
    }))?;

    Ok(html! {
        div class="flex flex-col h-16 group" {
            div class="flex flex-row items-center" {
                object data="/public/search.svg" type="image/svg+xml" class="h-4 filter dark:invert -mr-7 pl-3" {}
                input class="resize-none min-w-0 pl-10 px-4 py-2 focus:outline-none border-black border rounded-xl group-focus-within:rounded-b-none
                    group-hover:bg-neutral-100 group-focus-within:bg-neutral-100 
                    dark:bg-zinc-800 dark:border-zinc-700
                    dark:group-hover:bg-zinc-700 dark:group-focus-within:bg-zinc-700"
                    type="search" name="query" id="query" size="60" value=(query)
                    hx-post="/search-suggestions" hx-target="#search-suggestions" hx-trigger="keyup changed throttle:100ms, mouseover once" hx-vals=(vals) {}
            }

            div id="search-suggestions" class="flex flex-col border-black border pb-2 rounded-b-xl bg-neutral-100 invisible w-full z-50
                group-focus-within:visible
                dark:border-zinc-700 dark:bg-zinc-700" {}
        }
    })
}

#[derive(Deserialize, Serialize)]
pub struct SearchSuggestionQuery {
    query: String,
    search_type: SearchType,
    current_params: Option<ExtraSearchQuery>,
}

pub async fn search_suggestions(
    State(state): State<Arc<AppState>>,
    Form(query): Form<SearchSuggestionQuery>,
) -> Result<Markup, (StatusCode, String)> {
    let suggestions = state
        .client
        .lock()
        .await
        .complete_search(CompleteSearchRequest {
            current: query.query,
        })
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?
        .into_inner();

    let possibilities = suggestions.possibilities;

    let search_url = match query.search_type {
        SearchType::Html => "/search",
        SearchType::Image => "/image/search",
    };

    Ok(html! {
        @for possibility in &possibilities {
            @let search_params = serde_url_params::to_string(&SearchQuery {
                query: possibility.clone(),
                page: None,
                extra: None,
            })
            .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

            a href=(search_url.to_owned() + "?" + &search_params) class="px-2 py-1 hover:bg-neutral-200 dark:hover:bg-zinc-600" {
                (possibility)
            }
        }
    })
}
