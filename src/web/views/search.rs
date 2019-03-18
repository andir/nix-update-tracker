use actix_web::HttpResponse;
use actix_web::Query;
use actix_web::Responder;
use actix_web::State;
use diesel::prelude::*;
use horrorshow::{Render, RenderOnce, Template, TemplateBuffer};

use crate::process_db::models;
use crate::web::queries;
use crate::web::views::{issue_link, Page, Result};
use crate::web::AppState;

#[derive(Deserialize, Debug, Clone)]
pub struct SearchQuery {
    q: String,
}

struct IssueSearchResults(Vec<models::Issue>);
struct PackageSearchResults(Vec<models::Package>);

impl IssueSearchResults {
    pub fn new(d: Vec<models::Issue>) -> Self {
        IssueSearchResults(d)
    }

    pub fn is_empty(&self) -> bool {
        return self.0.is_empty();
    }
}

impl PackageSearchResults {
    pub fn new(d: Vec<models::Package>) -> Self {
        PackageSearchResults(d)
    }

    pub fn is_empty(&self) -> bool {
        return self.0.is_empty();
    }
}

struct SearchResultView {
    q: String,
    issues: IssueSearchResults,
    packages: PackageSearchResults,
}

fn issue_fn(row: &models::Issue) -> Box<Render> {
    issue_link(row.clone())
}

fn value_fn(row: &models::Issue) -> Box<Render> {
    Box::new(row.identifier.clone())
}

impl RenderOnce for IssueSearchResults {
    fn render_once(self, tmpl: &mut TemplateBuffer) {
        use super::table::{AsTable, Column};

        tmpl << html! {
            div {
                h2 : "Issues";
                |t| {
                    self.0.as_table(
                        t,
                        &[
                            Column::<models::Issue>::new("Identifier", &issue_fn),
                            Column::new("Description", &value_fn),
                        ],
                    )

                }
            }
        };
    }
}

impl RenderOnce for PackageSearchResults {
    fn render_once(self, tmpl: &mut TemplateBuffer) {
        use super::table::{AsTable, Column};

        tmpl << html! {
            div {
                h2 : "Packages (not of much use right now)";
                |t| {
                    self.0.as_table(
                        t,
                        &[
                            Column::new("Attribute Name", &|pkg: &models::Package| Box::new(pkg.attribute_name.clone())),
                            Column::new("Name", &|pkg: &models::Package| Box::new(pkg.name.clone()))
                        ]
                    )
                }
            }
        };
    }
}

impl Page<SearchResultView> {
    fn new_for_query(query: &SearchQuery, conn: &SqliteConnection) -> Result<Self> {
        let issues = queries::search_issues(&query.q, conn)?;
        let packages = queries::search_packages(&query.q, conn)?;

        return Page::new_with(
            format!("Search results: {}", query.q),
            SearchResultView {
                q: query.q.clone(),
                issues: IssueSearchResults::new(issues),
                packages: PackageSearchResults::new(packages),
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
        } else {
            tmpl << html! {
                h2 {
                    : "Results for ";
                    strong {
                        : self.q.clone();
                    }
                }
                @ if !self.issues.is_empty() {
                    :self.issues;
                }
                @ if !self.packages.is_empty() {
                    : self.packages;
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
        Ok(s) => HttpResponse::Ok().content_type("text/html").body(s),
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
