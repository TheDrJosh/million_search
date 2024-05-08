use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Form};
use maud::{html, Markup, DOCTYPE};
use proto::search::CompleteSearchRequest;
use serde::{Deserialize, Serialize};

use crate::AppState;

pub fn basic_page(body: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html {
            head {
                title {
                    "Tree Search"
                }
                link rel="stylesheet" href="/public/main.css" {}
                script src="https://unpkg.com/htmx.org@1.9.12" integrity="sha384-ujb1lZYygJmzgSwoxRggbCHcjc0rB2XoQrxeTUQyRjrOnlCoYta87iKBWq3EsdM2" crossorigin="anonymous" {}
            }
            body {
                (body)
            }
        }
    }
}

pub fn search_bar(query: &str) -> Markup {
    html! {
        input class="border-black border resize-none pl-10 pr-4 py-2 rounded-full hover:bg-neutral-100 dark:bg-zinc-800 dark:border-zinc-700 dark:hover:bg-zinc-700 min-w-0"
            type="search" name="query" id="query" size="60" value=(query) list="search-suggestions"
            hx-post="/search-suggestions" hx-target="#search-suggestions" hx-trigger="keyup changed throttle:500ms" {}
        datalist id="search-suggestions" {}
    }
}

#[derive(Deserialize, Serialize)]
pub struct SearchSuggestionQuery {
    query: String,
}

pub async fn search_suggestions(
    State(state): State<Arc<AppState>>,
    Form(query): Form<SearchSuggestionQuery>,
) -> Result<Markup, StatusCode> {
    let suggestions = state
        .client
        .lock()
        .await
        .complete_search(CompleteSearchRequest {
            current: query.query,
        })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_inner();

    let possibilities = suggestions.possibilities;

    Ok(html! {
        @for possibilitie in possibilities {
            option value=(possibilitie) {}
        }
    })
}
