use maud::{html, Markup, DOCTYPE};

pub fn basic_page(body: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html {
            head {
                title {
                    "Tree Search"
                }
                link rel="stylesheet" href="/public/main.css" {}
            }
            body {
                (body)
            }
        }
    }
}
