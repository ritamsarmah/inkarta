use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::get,
    Router,
};
use maud::{html, Markup, DOCTYPE};

use crate::{db, state::AppState, utils};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(home_page))
        .route("/error/:message", get(error_page))
        .route("/x/upload", get(upload))
        .fallback(not_found_page)
}

/* Pages */

async fn home_page(State(state): State<AppState>) -> impl IntoResponse {
    match db::get_images(&state.pool).await {
        Ok(images) => {
            let html = html! {
                main {
                    h1 { "Gallery" }

                    #gallery x-init {
                        @for image in images {
                            @let href = format!("x/image/{}", image.id);
                            a href=(href) style="text-decoration:none" {
                                .image {
                                    img src="https://picsum.photos/256/256";
                                    h5 style="color:var(--text-1)" { (image.title) }
                                    h6 style="color:var(--text-2)" { (image.artist.unwrap_or_default()) }
                                }
                            }
                        }
                    }

                    dialog x-init "@dialog:open.window"="$el.showModal()" {
                        #modal { }
                    }
                }
            };

            template("Gallery", html).into_response()
        }
        Err(error) => utils::redirect_error(error.to_string()).into_response(),
    }
}

async fn not_found_page() -> Markup {
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

async fn error_page(Path(message): Path<String>) -> Markup {
    template(
        "Error",
        html! {
            main {
                b { "An error occurred processing the request" }
                p { (message) }
                a href="/" { "Back to home" }
            }
        },
    )
}

/* Partials */

async fn upload() -> Markup {
    html! {
        #modal {
            .modal-content {
                form method="post" action="/image" enctype="multipart/form-data" {
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

                    input type="submit" value="Upload Image";
                }
            }
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
