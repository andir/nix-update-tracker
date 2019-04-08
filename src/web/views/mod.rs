use actix_web::{HttpResponse, Responder};
use diesel::prelude::SqliteConnection;
use horrorshow::helper::doctype;
use horrorshow::prelude::*;

use crate::process_db::models;

use super::queries;

pub use self::channel::{issues_in_channel_revision, latest_issues_in_channel};
pub use self::index::index as show_index;
pub use self::issue::show_issue;
pub use self::response::{HtmlResponse, JsonResponse};
pub use self::result::HttpResult;
pub use self::search::show_search_results;
use actix_web::HttpRequest;
use reqwest::mime;

mod channel;
mod header;
mod index;
mod issue;
mod response;
mod result;
mod search;
mod table;

error_chain! {
    foreign_links {
        DieselError(diesel::result::Error);
    }
}

fn issue_link(issue: models::Issue) -> Box<Render> {
    let link = format!("/issues/{}", issue.identifier.clone());
    box_html! {
        a(href=link.clone()) : issue.identifier.clone()
    }
}

struct Page<C> {
    title: String,
    channels: Vec<models::Channel>,
    content: C,
}

// If a view supports serializing we must also check if the request
// preferres that content type and handle it accordingly
impl<C> Responder for Page<C>
where
    C: JsonResponse,
    Page<C>: HtmlResponse,
{
    type Item = HttpResponse;
    type Error = actix_web::Error;

    fn respond_to<S>(
        self,
        req: &HttpRequest<S>,
    ) -> actix_web::Result<HttpResponse, actix_web::Error> {
        use actix_web::HttpMessage;

        let headers = req.headers();

        if let Some(val) = headers.get("accept") {
            if let Some(accept) = header::Accept::from_header(val) {
                if accept.contains(&mime::APPLICATION_JSON) {
                    return self.content.to_json_response(req);
                }
            }
        }

        return self.to_html_response(req);
    }
}

impl<C> JsonResponse for Page<C>
where
    C: serde::Serialize,
{
    fn to_json_response<S>(
        &self,
        req: &HttpRequest<S>,
    ) -> actix_web::Result<HttpResponse, actix_web::Error> {
        let body = serde_json::to_string(&self.content)?;
        Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(&body))
    }
}

impl<C> HtmlResponse for Page<C>
where
    C: RenderOnce,
{
    fn to_html_response<S>(
        self,
        _req: &HttpRequest<S>,
    ) -> actix_web::Result<HttpResponse, actix_web::Error> {
        let r = self.into_string();
        match r {
            Ok(o) => Ok(HttpResponse::Ok().content_type("text/html").body(o)),
            Err(e) => {
                error!("Failed to render index: {}", e);
                Ok(HttpResponse::InternalServerError().finish())
            }
        }
    }
}

impl<C> Page<C> {
    fn new_with(title: String, content: C, conn: &SqliteConnection) -> Result<Self> {
        let channels = queries::get_channels(&conn)?;

        Ok(Self {
            title,
            channels,
            content,
        })
    }
}

impl<C> RenderOnce for Page<C>
where
    C: RenderOnce,
{
    fn render_once(self, tmpl: &mut TemplateBuffer) {
        tmpl << html! {
            : doctype::HTML;
            html(lang="en") {
                head {
                    meta(charset="utf-8");
                    meta(name="viewport", content="width=device-width, initial-scale=1, shrink-to-fit=no");
                    title : &self.title;
                    link(rel="stylesheet", href="/_static/bootstrap-4.3.1.min.css");
                    link(rel="shortcut icon", type="image/png", href="/_static/logo.png");
                }
                body {
                    nav(class="navbar navbar-expand-sm navbar-dark bg-dark") {
                        a(class="navbar-brand", href="/") {
                            img(src="/_static/logo.png", class="d-inline-block", style="height: 30px;");
                            : "broken.sh";
                        }
                        button(class="navbar-toggler", type="button", data-toggle="collapse",
                                data-target="#navbarSupportedContent",
                                aria-controls="navbarSupportedContent",
                                aria-expanded="false",
                                aria-label="Toggle navigation") {
                            span(class="navbar-toggler-icon") {}
                        }

                        div(class="collapse navbar-collapse", id="navbarSupportedContent") {
                            ul(class="navbar-nav mr-auto") {
                                @ for channel in self.channels {
                                    li(class="nav-item") {
                                        a(class="nav-link",
                                           href=format!("/channels/{}", channel.name)) : channel.name.clone();
                                    }
                                }
                            }
                            form(class="form-inline my-2 my-lg-0", action="/search", method="get") {
                                input(class="form-control mr-sm-2", name="q", type="search", placeholder="Search", aria-label="Search");
                                button(class="btn btn-outline-success my-2 my-sm-0", type="submit") : "Search";
                            }
                        }
                    }
                    }
                div(class="container") : self.content;
                script(src="/_static/jquery-3.3.1.slim.min.js") {}
                script(src="/_static/bootstrap-4.3.1.bundle.min.js") {}
            }
        };
    }
}
