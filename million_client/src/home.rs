use axum::http::StatusCode;
use maud::{html, Markup};

use crate::{
    utils::{basic_page, search_bar},
    SearchType,
};

pub async fn home_search_page(search_type: SearchType) -> Result<Markup, StatusCode> {
    let sub_text = match search_type {
        SearchType::Html => None,
        SearchType::Image => Some("Images"),
        SearchType::Video => Some("Videos"),
        SearchType::Audio => Some("Audio"),
    };

    let search_url = match search_type {
        SearchType::Html => "/search",
        SearchType::Image => "/image/search",
        SearchType::Video => "/video/search",
        SearchType::Audio => "/audio/search",
    };

    Ok(basic_page(html! {
        div class="h-lvh flex flex-col items-center justify-center dark:bg-zinc-800 dark:text-zinc-50" {
            div class="ml-2 flex flex-row gap-4 self-start" {
                @if search_type != SearchType::Html {
                    a href="/" {
                        "<"
                    }
                }
                a href="/image" {
                    "Images"
                }
                a href="video" {
                    "Videos"
                }
                a href="audio" {
                    "Audio"
                }
            }
            div class="flex-1" {}

            div class="flex-1 flex flex-col" {
                h1 class="font-bold tracking-tight text-6xl mb-2 text-center" {
                    "Million Search"
                }
                span class="trackinng-tight text-2xl mb-10 text-center" {
                    @if let Some(sub_text) = sub_text {
                        (sub_text)
                    }
                }
                form action=(search_url) autocomplete="off" class="flex flex-row items-center" {
                    // object data="public/search.svg" type="image/svg+xml" class="h-4 -mr-8 z-10 filter dark:invert" {}

                    // input class="border-black border resize-none pl-10 pr-4 py-2 rounded-full hover:bg-neutral-100 dark:bg-zinc-800 dark:border-zinc-700 dark:hover:bg-zinc-700"
                    //     type="search" name="query" id="query" size="60" value="" {}
                    (search_bar(""))
                }
            }

            div class="flex-[2]" {}

            footer class="bg-neutral-100 grid dark:bg-zinc-900 grid-cols-3 w-full px-4 py-2 gap-2" {
                a href="https://dryicons.com/icon/search-2621" {"Icon by Dryicons"}
            }
        }

    }))
}
