use std::sync::Arc;

use axum::http::StatusCode;
use maud::{html, Markup};

use proto::search::{SearchWebRequest, SearchWebResult};

use crate::{
    utils::{basic_page, search_bar},
    AppState, SearchQuery, SearchQueryList, SearchType,
};

pub async fn search_page(
    search_type: SearchType,
    query: SearchQuery,
    _state: Arc<AppState>,
) -> Result<Markup, StatusCode> {
    let url_params =
        serde_url_params::to_string(&query).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let search_url = match search_type {
        SearchType::Html => "/search",
        SearchType::Image => "/image/search",
        SearchType::Video => "/video/search",
        SearchType::Audio => "/audio/search",
    };

    let search_params = serde_json::to_string(&SearchQueryList {
        query: query.query.clone(),
        page: 0,
        extra: query.extra,
    })
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let grab_list = html! {
        div hx-post=(search_url) hx-trigger="intersect once" hx-swap="outerHTML" hx-vals=(search_params) {}
    };

    let surrounding_formating = match search_type {
        SearchType::Html => {
            html! {
                div class = "flex flex-col" {
                    (grab_list)
                }
            }
        }
        SearchType::Image => todo!(),
        SearchType::Video => todo!(),
        SearchType::Audio => todo!(),
    };

    Ok(basic_page(html! {
        div class="min-h-lvh flex flex-col items-start dark:bg-zinc-800 dark:text-zinc-50 overflow-hidden" {

            header class="flex flex-col pt-6 pb-2 border-b-2 border-neutral-200 dark:border-zinc-700 w-full items-center " {
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

                    form action=(search_url) autocomplete="off" class="flex flex-row items-center" {
                        (search_bar(&query.query, search_type).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?)
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

            div class="flex-1 px-6 pt-4 overflow-y-scroll w-full" {
                (surrounding_formating)
            }


            footer class="bg-neutral-100 grid dark:bg-zinc-900 grid-cols-3 w-full px-4 py-2 gap-2" {
                a href="https://dryicons.com/icon/search-2621" {"Icon by Dryicons"}
            }
        }
    }))
}

pub async fn search_page_results(
    search_type: SearchType,
    query: SearchQueryList,
    state: Arc<AppState>,
) -> Result<Markup, (StatusCode, String)> {
    // tokio::time::sleep(Duration::from_secs(2)).await; // use for loading spinner testing
    match search_type {
        SearchType::Html => search_page_results_html(query.query, query.page, state).await,
        SearchType::Image => todo!(),
        SearchType::Video => todo!(),
        SearchType::Audio => todo!(),
    }
}

async fn search_page_results_html(
    query: String,
    page: u32,
    state: Arc<AppState>,
) -> Result<Markup, (StatusCode, String)> {
    let results: Vec<SearchWebResult> = state
        .client
        .lock()
        .await
        .search_web(SearchWebRequest {
            query: Some(proto::search::SearchQuery {
                query: query.clone(),
                page,
            }),
        })
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?
        .into_inner()
        .results;

    let search_params = serde_json::to_string(&SearchQueryList {
        query: query.clone(),
        page,
        extra: None,
    })
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    Ok(html! {
        @for result in &results {
            (render_html_result(result))
        }
        @if results.len() != 0 as usize {
            div hx-post="/search" hx-trigger="intersect once" hx-swap="outerHTML" hx-vals=(search_params) {}
        }
    })
}

fn render_html_result(result: &SearchWebResult) -> Markup {
    html! {
        div class="my-4 flex flex-col" {
            a href=(result.url) {
                div class="flex flex-row items-center" {
                    img class="w-8 h-8 bg-white rounded-full p-1" src=(result.icon_url.as_deref().unwrap_or("/public/gloabe.svg")) {}

                    div class="flex flex-col ml-4" {
                        span class="font-semibold truncate" {
                            (result.title.as_deref().unwrap_or("Site Name Placeholder"))
                        }
                        span class="text-sm truncate" {
                            (&result.url)
                        }
                    }
                }
                h2 class="text-lg font-bold truncate" {
                    (result.title.as_deref().unwrap_or(&result.url))
                }
            }
            @if result.inner_text_match.is_some() || result.description.is_some() {
                p class="w-full sm:w-1/2" {
                    (result.inner_text_match.as_deref().or(result.description.as_deref()).unwrap())
                }
            }
        }
    }
}
