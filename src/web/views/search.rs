use actix_web::HttpResponse;
use actix_web::Query;
use actix_web::Responder;
use actix_web::State;
use diesel::prelude::*;
use horrorshow::{RenderOnce, Template, TemplateBuffer};

use process_db::models;
use web::queries;
use web::views::Result;
use web::views::{issue_link, Page};
use web::AppState;

#[derive(Deserialize, Debug, Clone)]
pub struct SearchQuery {
    q: String,
}

struct SearchResultView {
    q: String,
    issues: Vec<models::Issue>,
    packages: Vec<models::Package>,
}

impl Page<SearchResultView> {
    fn new_for_query(query: &SearchQuery, conn: &SqliteConnection) -> Result<Self> {
        let issues = queries::search_issues(&query.q, conn)?;
        let packages = queries::search_packages(&query.q, conn)?;

        return Page::new_with(
            format!("Search results: {}", query.q),
            SearchResultView {
                q: query.q.clone(),
                issues: issues,
                packages: packages,
            },
            conn,
        );
    }
}

impl RenderOnce for SearchResultView {
    fn render_once(self, tmpl: &mut TemplateBuffer) {
        if self.issues.is_empty() && self.packages.is_empty() {
            tmpl << html! {
                p {
                  : "No results found.";
                }
            };
            return;
        } else {
            tmpl << html! {
                    h2 {
                        : "Results for ";
                        strong {
                            : self.q.clone();
                        }
                    }
                    |t| {
                        if !self.issues.is_empty() {
                            t << html! {
                                div {
                                    h2 : "Issues";
                                    ul {
                                        @for issue in self.issues.iter() {
                                            li {
                                                : issue_link(issue.clone());
                                            }
                                        }
                                    }
                                }
                            };
                        }
                    }

                    |t| {
                        if !self.packages.is_empty() {
                            t << html! {
                                div {
                                    h2 : "Packages";
                                    @for package in self.packages.iter() {
                                        p {
                                            : package.name.clone();
                                        }
                                    }
                                }
                            };
                        }
                    }
            };
        }
    }
}

pub fn show_search_results(
    (state, query): (State<AppState>, Query<SearchQuery>),
) -> impl Responder {
    let q = query.into_inner();
    let p = match Page::<SearchResultView>::new_for_query(&q, &state.connection) {
        Ok(p) => p,
        Err(e) => {
            error!(
                "Failed to load search results for query `{:?}`: {}",
                q.clone(),
                e
            );
            return HttpResponse::InternalServerError().finish();
        }
    };

    match p.into_string() {
        Ok(s) => HttpResponse::Ok().body(s),
        Err(e) => {
            error!(
                "Failed to render search results for query `{:?}`: {}",
                q.clone(),
                e
            );
            return HttpResponse::InternalServerError().finish();
        }
    }
}
