use std::sync::Arc;

use axum::http::StatusCode;
use maud::{html, Markup};
use proto::search::{SearchWebRequest, SearchWebResult};

use crate::{utils::basic_page, AppState, SearchQuery, SearchType};

pub async fn search_page(
    search_type: SearchType,
    query: SearchQuery,
    state: Arc<AppState>,
) -> Result<Markup, StatusCode> {
    let url_params =
        serde_url_params::to_string(&query).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    println!("{}", url_params);

    Ok(basic_page(html! {
        div class="min-h-lvh flex flex-col items-start dark:bg-zinc-800 dark:text-zinc-50 overflow-hidden" {

            header class="flex flex-col pt-6 pb-2 border-b-2 border-neutral-200 dark:border-zinc-700 w-full items-center" {
                div class="flex flex-row w-full items-center" {
                    @match search_type {
                        SearchType::Html => {
                            a href="/" class="flex flex-col items-center" {
                                h1 class="font-bold tracking-tight text-3xl ml-6 mr-12 text-center" {
                                    "Million Search"
                                }
                            }
                        }
                        SearchType::Image => {
                            a href="/image" class="flex flex-col items-center" {
                                h1 class="font-bold tracking-tight text-3xl ml-6 mr-12 text-center" {
                                    "Million Search"
                                }
                                span class="trackinng-tight text-lg text-center" {
                                    "Images"
                                }
                            }
                        }
                        SearchType::Video => {
                            a href="/video" class="flex flex-col items-center" {
                                h1 class="font-bold tracking-tight text-3xl ml-6 mr-12 text-center" {
                                    "Million Search"
                                }
                                span class="trackinng-tight text-lg text-center" {
                                    "Videos"
                                }
                            }
                        }
                        SearchType::Audio => {
                            a href="/audio" class="flex flex-col items-center" {
                                h1 class="font-bold tracking-tight text-3xl ml-6 mr-12 text-center" {
                                    "Million Search"
                                }
                                span class="trackinng-tight text-lg text-center" {
                                    "Audio"
                                }
                            }
                        }
                    }

                    form action="/search" autocomplete="off" class="flex flex-row items-center" {
                        object data="/public/search.svg" type="image/svg+xml" class="h-4 -mr-8 z-10 filter dark:invert" {}

                        input class="border-black border resize-none pl-10 pr-4 py-2 rounded-full hover:bg-neutral-100 dark:bg-zinc-800 dark:border-zinc-700 dark:hover:bg-zinc-700"
                            type="search" name="query" id="query" size="60" value=(query.query) {}
                    }
                }
                div class="flex flex-row gap-4 self-start pl-4 pt-2" {
                    a href=("/search?".to_owned() + &url_params) {
                        "All"
                    }
                    a href=("/image/search?".to_owned() + &url_params) {
                        "Images"
                    }
                    a href=("/video/search?".to_owned() + &url_params) {
                        "Videos"
                    }
                    a href=("/audio/search?".to_owned() + &url_params) {
                        "Audio"
                    }
                }
            }

            div class="flex-1 mx-6 my-4 overflow-y-scroll w-full" {
                (fetch_search_html(query, state).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?)
            }

            footer class="bg-neutral-100 grid dark:bg-zinc-900 grid-cols-3 w-full px-4 py-2 gap-2" {
                a href="https://dryicons.com/icon/search-2621" {"Icon by Dryicons"}
            }
        }
    }))
}

async fn fetch_search_html(query: SearchQuery, state: Arc<AppState>) -> Result<Markup, StatusCode> {
    if query.extra.is_some() {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let results = state
        .client
        .lock()
        .await
        .search_web(SearchWebRequest {
            query: Some(proto::search::SearchQuery {
                query: query.query.clone(),
                start: query.start,
                length: query.length,
            }),
        })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_inner()
        .results;

    Ok(html! {
        div class="flex flex-col" {
            @for result in &results {
                (render_web_results(result))
            }
            @if results.len() == query.length as usize {
                div hx-post="/search" hx-trigger="intersect once" hx-swap="outerHTML" hx-vals={"{\"query\": " (query.query) "\", start\": " (query.start + query.length) "}"} {}
            }
        }
    })
}

fn render_web_results(result: &SearchWebResult) -> Markup {
    html! {
        div {
            div {
                span {
                    (result.title())
                }
                span {
                    (result.url)
                }
            }

        }
    }
}
