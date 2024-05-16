use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Form};
use maud::{html, Markup};

use proto::search::{SearchImageRequest, SearchImageResult, SearchWebRequest, SearchWebResult};
use serde::{Deserialize, Serialize};

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
                div class="flex flex-col" {
                    (grab_list)
                }
            }
        }
        SearchType::Image => html! {
            div class="flex flex-row" {
                div class="flex flex-row flex-wrap items-center flex-[2] overflow-y-scroll" {
                    (grab_list)
                }
                div id="image-view" hx-post="/image/search/view" hx-target="#image-view" hx-swap="outerHTML" {}
            }
        },
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

                    }

                    form action=(search_url) autocomplete="off" class="flex flex-row items-center" {
                        (search_bar(&query.query, search_type).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?)
                    }
                }
                div class="flex flex-row gap-4 self-start pl-4 pt-2" {
                    a href=("/search?".to_owned() + &url_params) {
                        "Web"
                    }
                    a href=("/image/search?".to_owned() + &url_params) {
                        "Images"
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
        SearchType::Image => search_page_results_image(query.query, query.page, state).await,
    }
}

async fn search_page_results_html(
    query: String,
    page: u32,
    state: Arc<AppState>,
) -> Result<Markup, (StatusCode, String)> {
    let results = state
        .client
        .lock()
        .await
        .search_web(SearchWebRequest {
            query: Some(proto::search::SearchQuery {
                query: query.clone(),
                page: page + 1,
            }),
        })
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?
        .into_inner()
        .results;

    let search_params = serde_json::to_string(&SearchQueryList {
        query: query.clone(),
        page: page + 1,
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
    let site_name = result
        .site_name
        .as_deref()
        .or(result.title.as_deref())
        .unwrap_or(result.url.as_str())
        .to_owned();

    html! {
        div class="my-4 flex flex-col" {
            a href=(result.url) {
                div class="flex flex-row items-center" {
                    img class="w-8 h-8 rounded-full p-1 bg-white" src=(result.icon_url.as_deref().unwrap_or("/public/gloabe.svg")) {}

                    div class="flex flex-col ml-4" {
                        span class="font-semibold truncate" {
                            (site_name)
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

async fn search_page_results_image(
    query: String,
    page: u32,
    state: Arc<AppState>,
) -> Result<Markup, (StatusCode, String)> {
    let results = state
        .client
        .lock()
        .await
        .search_image(SearchImageRequest {
            query: Some(proto::search::SearchQuery {
                query: query.clone(),
                page: page + 1,
            }),
            size: None,
        })
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?
        .into_inner()
        .results;

    let search_params = serde_json::to_string(&SearchQueryList {
        query: query.clone(),
        page: page + 1,
        extra: None,
    })
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    Ok(html! {
        @for (i, result) in results.iter().enumerate() {
            (render_image_result(result, &query, None, page, i as u32).map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?)
        }
        @if results.len() != 0 as usize {
            div hx-post="/image/search" hx-trigger="intersect once" hx-swap="outerHTML" hx-vals=(search_params) {}
        }
    })
}

fn render_image_result(
    result: &SearchImageResult,
    query: &str,
    size_range: Option<SizeRange>,
    page: u32,
    i: u32,
) -> anyhow::Result<Markup> {
    let view_data = serde_json::to_string(&ViewData {
        query: query.to_owned(),
        size_range,
        page,
        item: i,
    })?;

    Ok(html! {
        div class="flex flex-col m-4 w-fit max-w-48" {
            img src=(result.url) class="min-h-12 max-h-36 object-contain rounded-md" alt=(result.alt_text())
                hx-post="/image/search/view" hx-target="#image-view" hx-swap="outerHTML" hx-vals=(view_data) {}

            a href=(result.source_url) class="min-w-0 flex flex-col" {
                div class="flex flex-row items-center min-w-0" {
                    img src=(result.source_icon_url.as_deref().unwrap_or("/public/gloabe.svg")) class="w-4 h-4 bg-white rounded-full mr-2 p-0.5" {}
                    span class="text-ellipsis min-w-0 overflow-hidden whitespace-nowrap" {
                        (result.source_title)
                    }
                }
                span class="text-ellipsis min-w-0 overflow-hidden whitespace-nowrap" {
                    (result.alt_text())
                }
            }
        }
    })
}

//TODO - Get this info from previus url
#[derive(Serialize, Deserialize, Debug)]
pub struct ViewData {
    query: String,
    size_range: Option<SizeRange>,
    page: u32,
    item: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SizeRange {
    pub min_width: u32,
    pub min_height: u32,
    pub max_width: u32,
    pub max_height: u32,
}

pub async fn image_view(
    State(state): State<Arc<AppState>>,
    view_state: Option<Form<ViewData>>,
) -> Result<Markup, (StatusCode, String)> {
    Ok(match view_state {
        Some(Form(view_data)) => {
            //get data

            let img_list = state
                .client
                .lock()
                .await
                .search_image(SearchImageRequest {
                    query: Some(proto::search::SearchQuery {
                        query: view_data.query,
                        page: view_data.page,
                    }),
                    size: view_data.size_range.map(|range| proto::search::SizeRange {
                        min_width: range.min_width,
                        min_height: range.min_height,
                        max_width: range.max_width,
                        max_height: range.max_height,
                    }),
                })
                .await
                .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?
                .into_inner()
                .results;
            let img = img_list.get(view_data.item as usize).ok_or((
                StatusCode::BAD_REQUEST,
                "item outside of page range".to_owned(),
            ))?;

            html! {
                div id="image-view" class="flex flex-col border-l transition-all flex-1" {
                    button class="self-end" hx- post="/image/search/view" hx-target="#image-view" hx-swap="outerHTML" {"X"}

                    img src=(img.url) alt=(img.alt_text.as_deref().unwrap_or_default()) {}


                }
            }
        }
        None => html! {
            div id="image-view" class="flex flex-col border-l-0 transition-all flex-none" {}
        },
    })
}
