use std::sync::Arc;

use axum::http::StatusCode;
use maud::{html, Markup};

use crate::{utils::basic_page, AppState, SearchQuery, SearchType};

pub async fn search_page(
    search_type: SearchType,
    query: SearchQuery,
    state: Arc<AppState>,
) -> Result<Markup, StatusCode> {
    Ok(basic_page(html! {
        div class="min-h-lvh flex flex-col items-start dark:bg-zinc-800 dark:text-zinc-50 overflow-hidden" {

            header class="flex flex-row py-6 border-b-2 border-neutral-200 dark:border-zinc-700 w-full" {
                @match search_type {
                    SearchType::Html => {
                        a href="/" {
                            h1 class="font-bold tracking-tight text-3xl ml-6 mr-12 text-center" {
                                "Million Search"
                            }
                        }
                    }
                    SearchType::Image => {
                        a href="/image" {
                            h1 class="font-bold tracking-tight text-3xl ml-6 mr-12 text-center" {
                                "Million Search"
                            }
                            span {
                                "Images"
                            }
                        }
                    }
                    SearchType::Video => {
                        a href="/video" {
                            h1 class="font-bold tracking-tight text-3xl ml-6 mr-12 text-center" {
                                "Million Search"
                            }
                            span {
                                "Videos"
                            }
                        }
                    }
                    SearchType::Audio => {
                        a href="/audio" {
                            h1 class="font-bold tracking-tight text-3xl ml-6 mr-12 text-center" {
                                "Million Search"
                            }
                            span {
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

            div class="flex-1 mx-6 my-4 overflow-y-scroll w-full" {
                // @for result in &response {
                //     (render_result(result))
                // }
                // @if response.len() == INITIAL_QUERY_LENGTH {
                //     div hx-post="/search" hx-trigger="intersect once" hx-swap="outerHTML" hx-vals={"{\"query\": " (query.query) "\", start\": " (INITIAL_QUERY_LENGTH) "}"} {}
                // }
            }

            footer class="bg-neutral-100 grid dark:bg-zinc-900 grid-cols-3 w-full px-4 py-2 gap-2" {
                a href="https://dryicons.com/icon/search-2621" {"Icon by Dryicons"}
            }
        }
    }))
}
