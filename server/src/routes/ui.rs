use axum::{
    extract::{Path, State},
    routing::get,
    Router,
};
use maud::{html, Markup, DOCTYPE};

use crate::{db, model::Identifier, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(home))
        .route("/ui/upload", get(upload))
        .route("/ui/image/:id", get(image))
        .fallback(not_found)
}

/* Pages */

async fn home(State(state): State<AppState>) -> Markup {
    let headings = ["Title", "Artist", "Action"];
    let html = html! {
        main {
            h1 { "Gallery" }
            table {
                thead {
                    @for heading in headings {
                        th scope="col" { (heading) }
                    }
                }
            }
            tbody #images x-init
                "@ajax:before"="$dispatch('dialog:open')"
                "@image:updated"="$ajax('/images')" {
            }
            dialog x-init "@dialog:open.window"="$el.showModal()" {
                #image { }
            }
        }
    };

    template("Gallery", html)
}

async fn not_found() -> Markup {
    template(
        "Page Not Found",
        html! {
            main .spaced {
                h1 { "404" }
                p { "Page not found." }
                a href="/" { "Back to home" }
            }
        },
    )
}

/* Partials */

async fn image(State(state): State<AppState>, Path(id): Path<Identifier>) -> Markup {
    match db::get_image(&state.pool, id).await {
        Ok(image) => html! {
            p { (image.title) }
            p { (image.artist.unwrap_or_else(|| "Anonymous".into())) }
        },
        Err(error) => html! {
            p { (error) }
        },
    }
}

async fn images(State(state): State<AppState>) -> Markup {
    match db::get_images(&state.pool).await {
        Ok(images) => html! {
            @for image in images {
                li { "hello" }
            }
        },
        Err(error) => html! {
            p { (error) }
        },
    }
}

async fn upload(State(state): State<AppState>) -> Markup {
    html! {
        form hx-encoding="multipart/form-data" hx-post="/image" {
            label for="title" { "Title: " }
            input type="text" id="title" name="title" required="true";
            br;

            label for="artist" { "Artist: " }
            input type="text" id="artist" name="artist";
            br;

            label for="color" { "Prefers Dark Background: " }
            input type="checkbox" id="dark" name="dark";
            br;

            label for="data" { "Select Image: " }
            input type="file" id="data" name="data" accept="image/*" required="true";
            br;

            input type="submit" value="Upload Image" _="on click trigger close_modal";
        }
    }
}

/* Template */

fn template(title: &str, body: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width,initial-scale=1";
                meta name="color-scheme" content="light dark";

                title { (format!("Inkarta | {title}")) }

                link rel="stylesheet" href="/styles.css";
                script defer src="https://cdn.jsdelivr.net/npm/@imacrayon/alpine-ajax@0.7.0/dist/cdn.min.js" {}
                script defer src="https://cdn.jsdelivr.net/npm/alpinejs@3.11.1/dist/cdn.min.js" {}
            }
            body { (body) }
        }
    }
}
