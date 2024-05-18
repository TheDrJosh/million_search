use std::sync::Arc;

use axum::{
    extract::State,
    http::{HeaderMap, HeaderName, StatusCode, Uri},
    Json,
};
use maud::{html, Markup};

use proto::search::{SearchImageRequest, SearchImageResult, SearchWebRequest, SearchWebResult};
use serde::{Deserialize, Serialize};

use crate::{
    utils::{basic_page, search_bar},
    AppState, SearchQuery, SearchType,
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

    let search_params = serde_json::to_string(&SearchQuery {
        query: query.query.clone(),
        page: None,
        size_range: query.size_range,
    })
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let grab_list = html! {
        div hx-post=(search_url) hx-trigger="intersect once" hx-swap="outerHTML" hx-vals=(search_params) {}
    };

    let surrounding_formating = match &search_type {
        SearchType::Html => {
            html! {
                div class="flex flex-col h-full overflow-y-scroll" {
                    (grab_list)
                }
            }
        }
        SearchType::Image => html! {
            div class="flex flex-row h-full overflow-hidden" {
                div class="flex flex-row flex-wrap flex-[2] overflow-y-scroll" {
                    (grab_list)
                }
                div id="image-view" hx-post="/image/search/view" hx-target="#image-view" hx-swap="outerHTML" {}
            }
        },
    };

    Ok(basic_page(html! {
        div class="h-lvh flex flex-col items-start dark:bg-zinc-800 dark:text-zinc-50 overflow-hidden" {//min-h-lvh
            header class="flex flex-col pt-6 pb-2 border-b-2 border-neutral-200 dark:border-zinc-700 w-full items-center " {
                div class="flex flex-row w-full items-center" {
                    @match &search_type {
                        SearchType::Html => {
                            a href="/" class="flex flex-col items-center"  {
                                h1 class="font-bold tracking-tight text-3xl ml-6 mr-12 text-center" {
                                    "Million Search"
                                }
                            }
                        }
                        SearchType::Image => {
                            a href="/image" class="flex flex-col items-center"  {
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
                        (search_bar(&query.query, &search_type).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?)
                    }
                }
                div class="flex flex-row gap-4 self-start pl-4 pt-2" {


                    @match &search_type {
                        SearchType::Html => {
                            a href=("/image/search?".to_owned() + &url_params)  {
                                "Images"
                            }
                        }
                        SearchType::Image => {
                            a href=("/search?".to_owned() + &url_params)  {
                                "Web"
                            }
                        }
                    }
                }
            }

            div class="flex-1 px-6 pt-4 w-full overflow-hidden" {//overflow-y-scroll
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
    query: SearchQuery,
    state: Arc<AppState>,
) -> Result<Markup, (StatusCode, String)> {
    // tokio::time::sleep(Duration::from_secs(2)).await; // use for loading spinner testing
    match search_type {
        SearchType::Html => {
            search_page_results_html(query.query, query.page.unwrap_or(1), state).await
        }
        SearchType::Image => {
            search_page_results_image(query.query, query.page.unwrap_or(1), state).await
        }
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
                page,
            }),
        })
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?
        .into_inner()
        .results;

    let search_params = serde_json::to_string(&SearchQuery {
        query: query.clone(),
        page: Some(page + 1),
        size_range: None,
    })
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    Ok(html! {
        @for result in &results {
            (render_html_result(result))
        }
        @if !results.is_empty() {
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
                page,
            }),
            size: None,
        })
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?
        .into_inner()
        .results;

    let search_params = serde_json::to_string(&SearchQuery {
        query: query.clone(),
        page: Some(page + 1),
        size_range: None,
    })
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    Ok(html! {
        @for (i, result) in results.iter().enumerate() {
            (render_image_result(result, page, i as u32).map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?)
        }
        @if !results.is_empty() {
            div hx-post="/image/search" hx-trigger="intersect once" hx-swap="outerHTML" hx-vals=(search_params) {}
        }
    })
}

fn render_image_result(result: &SearchImageResult, page: u32, i: u32) -> anyhow::Result<Markup> {
    let view_data = serde_json::to_string(&ViewData {
        page,
        item: i as i32,
    })?;

    Ok(html! {
        div class="flex flex-col m-4 w-fit max-w-48" {
            img src=(result.url) class="min-h-12 max-h-36 object-contain rounded-md" alt=(result.alt_text())
                hx-post="/image/search/view" hx-target="#image-view" hx-swap="outerHTML" hx-vals=(view_data) hx-ext="json-enc" {}

            a href=(result.source.as_ref().unwrap().url) class="min-w-0 flex flex-col" {
                div class="flex flex-row items-center min-w-0" {
                    img src=(result.source.as_ref().unwrap().icon_url.as_deref().unwrap_or("/public/gloabe.svg")) class="w-4 h-4 bg-white rounded-full mr-2 p-0.5" {}
                    span class="text-ellipsis min-w-0 overflow-hidden whitespace-nowrap" {
                        (display_site_name(result.source.as_ref().unwrap()))
                    }
                }
                span class="text-ellipsis min-w-0 overflow-hidden whitespace-nowrap" {
                    (result.alt_text())
                }
            }
        }
    })
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ViewData {
    page: u32,
    item: i32,
}

pub async fn image_view(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    view_state: Option<Json<ViewData>>,
) -> Result<Markup, (StatusCode, String)> {
    let current_url = headers
        .get(HeaderName::from_static("hx-current-url"))
        .ok_or((
            StatusCode::BAD_REQUEST,
            "need hx-current_url header".to_owned(),
        ))?
        .to_str()
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?
        .parse::<Uri>()
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    let current_query = current_url.query().ok_or((
        StatusCode::BAD_REQUEST,
        "hx-current-url header should have query params".to_owned(),
    ))?;

    let search_query = serde_qs::from_str::<SearchQuery>(current_query)
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    Ok(match view_state {
        Some(Json(view_data)) => {
            let img_list = state
                .client
                .lock()
                .await
                .search_image(SearchImageRequest {
                    query: Some(proto::search::SearchQuery {
                        query: search_query.query.clone(),
                        page: view_data.page,
                    }),
                    size: search_query
                        .size_range
                        .as_ref()
                        .map(|range| proto::search::SizeRange {
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

            let item = if view_data.item < 0 {
                img_list.len() as i32 + view_data.item - 1
            } else {
                view_data.item
            };

            let img = img_list.get(item as usize).ok_or((
                StatusCode::BAD_REQUEST,
                format!(
                    "item ({}) outside of page range, max: {}",
                    item,
                    img_list.len()
                ),
            ))?;

            let next_view = serde_json::to_string(&ViewData {
                page: view_data.page
                    + if item + 1 >= img_list.len() as i32 {
                        1
                    } else {
                        0
                    },
                item: if item + 1 >= img_list.len() as i32 {
                    0
                } else {
                    item + 1
                },
            })
            .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
            let prev_view = serde_json::to_string(&ViewData {
                page: view_data.page - if item - 1 < 0 { 1 } else { 0 },
                item: if item - 1 < 0 { -1 } else { item - 1 },
            })
            .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

            html! {
                div id="image-view" class="flex flex-col border-l border-neutral-200 dark:border-zinc-700 transition-all flex-1 overflow-y-scroll overflow-x-hidden p-4" {
                    div class="flex flex-row items-center pb-8" {
                        div class="flex flex-1 flex-row items-center" {
                            img class="self-center w-4 h-4 rounded-full mr-2 bg-white" src=(img.source.as_ref().unwrap().icon_url.as_deref().unwrap_or("/public/gloabe.svg")) alt=(img.alt_text.as_deref().unwrap_or_default()) {}
                            span class="text" {
                                (display_site_name(img.source.as_ref().unwrap()))
                            }
                        }
                        div class="flex flex-row pl-8 items-center font-semibold" {
                            button class="px-4" hx-post="/image/search/view" hx-target="#image-view" hx-swap="outerHTML" hx-vals=(prev_view) hx-ext="json-enc" {"<"}
                            button class="px-4" hx-post="/image/search/view" hx-target="#image-view" hx-swap="outerHTML" hx-vals=(next_view) hx-ext="json-enc" {">"}
                            button class="px-4" hx-post="/image/search/view" hx-target="#image-view" hx-swap="outerHTML" {"X"}
                        }
                    }

                    img class="self-center m-2 w-full rounded" src=(img.url) alt=(img.alt_text.as_deref().unwrap_or_default()) {}

                    div class="flex flex-row pt-4 items-center" {
                        span class="flex-1" {
                            (img.alt_text())
                        }
                        a href=(img.source.as_ref().unwrap().url) class="px-2 py-1 rounded-xl bg-sky-200 text-black font-semibold self-start h-fit" {
                            "Visit >"
                        }
                    }
                }
            }
        }
        None => html! {
            div id="image-view" class="flex flex-col border-l-0 transition-all flex-none" {}
        },
    })
}

fn display_site_name(website: &SearchWebResult) -> String {
    website
        .site_name
        .as_deref()
        .or(website.title.as_deref())
        .unwrap_or(website.url.as_str())
        .to_owned()
}
