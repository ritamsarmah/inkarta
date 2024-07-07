use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::get,
    Router,
};
use maud::{html, Markup, DOCTYPE};

use crate::{db, model::Identifier, state::AppState, utils};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(home_page))
        .route("/error/:message", get(error_page))
        .route("/x/image/:id", get(image_modal))
        .route("/x/upload", get(upload_modal))
        .fallback(not_found_page)
}

/* Pages */

async fn home_page(State(state): State<AppState>) -> impl IntoResponse {
    match db::get_images(&state.pool).await {
        Ok(images) => {
            let html = html! {
                main {
                    h1 { "Gallery" }

                    #gallery x-init "@ajax:before"="$dispatch('dialog:open')" {
                        @for image in images {
                            @let href = format!("x/image/{}", image.id);
                            a href=(href) x-target="modal" style="text-decoration:none" {
                                .image {
                                    // img src="https://picsum.photos/256/256";
                                    h5 style="color:var(--text-1)" { (image.title) }
                                    h6 style="color:var(--text-2)" { (image.artist.unwrap_or_default()) }
                                }
                            }
                        }
                    }

                    form action="/x/upload" x-init x-target="modal" "@ajax:before"="$dispatch('dialog:open')" {
                        button .btn.upload {
                            "+"
                        }
                    }

                    dialog x-init
                        "@dialog:open.window"="$el.showModal()"
                        "@dialog:close.window"="$el.close()" {
                            #modal { }
                    }
                }
            };

            page_template("Gallery", html).into_response()
        }
        Err(_) => utils::redirect_error().into_response(),
    }
}

async fn not_found_page() -> Markup {
    page_template(
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

async fn error_page() -> Markup {
    page_template(
        "Error",
        html! {
            main {
                b { "An error occurred processing the request" }
                a href="/" { "Back to home" }
            }
        },
    )
}

/* Partials */

async fn image_modal(
    Path(id): Path<Identifier>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match db::get_image(&state.pool, id).await {
        Ok(image) => modal_template(html! {
            #image-preview {
                h4 { (image.title) }
                h5 { (image.artist.unwrap_or_default()) }

                img src="https://picsum.photos/500/500" width="100%";
            }
        })
        .into_response(),
        Err(_) => utils::redirect_error().into_response(),
    }
}

async fn upload_modal() -> Markup {
    let on_change_image = r#"
        file = $event.target.files[0];
        if (file) {
            previewURL = URL.createObjectURL(file);
        }
    "#;

    modal_template(html! {
        form #upload-form method="post" action="/image" enctype="multipart/form-data" x-init x-data="{ file: null, previewURL: '' }" {
            label {
                input type="file"
                    #input-image
                    .hidden
                    name="data"
                    accept="image/*"
                    required="true"
                    "@change"=(on_change_image);

                .flex-center style="height:256px" {
                    img ":src"="previewURL" x-cloak x-show="previewURL"
                        style="max-width:100%; max-height:100%; cursor:pointer;";

                    .upload-image-btn x-show="!previewURL" {
                        svg xmlns="http://www.w3.org/2000/svg"
                            fill="none"
                            viewBox="0 0 24 24"
                            stroke-width="1"
                            stroke="currentColor" {
                                path stroke-linecap="round"
                                    stroke-linejoin="round"
                                    d="m2.25 15.75 5.159-5.159a2.25 2.25 0 0 1 3.182 0l5.159 5.159m-1.5-1.5 1.409-1.409a2.25 2.25 0 0 1 3.182 0l2.909 2.909m-18 3.75h16.5a1.5 1.5 0 0 0 1.5-1.5V6a1.5 1.5 0 0 0-1.5-1.5H3.75A1.5 1.5 0 0 0 2.25 6v12a1.5 1.5 0 0 0 1.5 1.5Zm10.5-11.25h.008v.008h-.008V8.25Zm.375 0a.375.375 0 1 1-.75 0 .375.375 0 0 1 .75 0Z";
                        }
                        div { "Select Image" }
                    }
                }
            }

            label {
                "Title: "
                input type="text" id="title" name="title" required="true";
            }

            label {
                "Artist: "
                input type="text" id="artist" name="artist";
            }

            label {
                "Prefers Dark Background: "
                input type="checkbox" id="dark" name="dark";
            }

            input type="submit" value="Upload Image";
        }
    })
}

/* Templates */

fn page_template(title: &str, body: Markup) -> Markup {
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

fn modal_template(content: Markup) -> Markup {
    html! {
        #modal x-init {
            .modal-content "@click.outside"="$dispatch('dialog:clos-btnlo')" {
                button .btn.close-btn "@click"="$dispatch('dialog:close')" { "✕" }
                (content)
            }
        }
    }
}
