use actix_web::HttpResponse;
use actix_web::Query;
use actix_web::Responder;
use actix_web::State;
use diesel::prelude::*;
use horrorshow::{Render, RenderOnce, Template, TemplateBuffer};

use crate::process_db::models;
use crate::web::queries;
use crate::web::views::{issue_link, HttpResult, Page, Result};
use crate::web::AppState;

#[derive(Deserialize, Debug, Clone)]
pub struct SearchQuery {
    q: String,
}

#[derive(Serialize)]
struct IssueSearchResults {
    // tuple of issue and issue description
    issues: Vec<(models::Issue, String)>,
}

#[derive(Serialize)]
struct PackageSearchResults(Vec<models::Package>);

impl IssueSearchResults {
    pub fn new(
        issues: &Vec<models::Issue>,
        nvd: &std::collections::HashMap<String, String>,
    ) -> Self {
        let issues = issues
            .iter()
            .map(|i| {
                let description = nvd
                    .get(&i.identifier)
                    .cloned()
                    .unwrap_or_else(|| "N/A".to_string());
                (i.clone(), description.to_string())
            })
            .collect();
        IssueSearchResults { issues }
    }

    pub fn is_empty(&self) -> bool {
        self.issues.is_empty()
    }
}

impl PackageSearchResults {
    pub fn new(d: Vec<models::Package>) -> Self {
        PackageSearchResults(d)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(Serialize)]
struct SearchResultView {
    q: String,
    issues: IssueSearchResults,
    packages: PackageSearchResults,
}

fn issue_fn(row: &models::Issue) -> Box<Render> {
    issue_link(row.clone())
}

impl RenderOnce for IssueSearchResults {
    fn render_once(self, tmpl: &mut TemplateBuffer) {
        use super::table::{AsTable, Column};

        tmpl << html! {
            div {
                h2 : "Issues";
                |tm| {
                    self.issues.as_table(
                        tm,
                        &[
                            Column::<(models::Issue, String)>::new("Identifier", &|(issue, _description)| {
                                issue_link(issue.clone())
                            }),
                            Column::new("Description", &|(_issue, description)| {
                                Box::new(description.clone())
                            })
                        ]
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
                    self.0.as_table(t,
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
    fn new_for_query(
        query: &SearchQuery,
        nvd: &std::collections::HashMap<String, String>,
        conn: &SqliteConnection,
    ) -> Result<Self> {
        let issues = queries::search_issues(&query.q, conn)?;
        let packages = queries::search_packages(&query.q, conn)?;

        Page::new_with(
            format!("Search results: {}", query.q),
            SearchResultView {
                q: query.q.clone(),
                issues: IssueSearchResults::new(&issues, nvd),
                packages: PackageSearchResults::new(packages),
            },
            conn,
        )
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
    let p = match Page::<SearchResultView>::new_for_query(&q, &state.nvd, &state.connection) {
        Ok(p) => p,
        Err(e) => {
            error!(
                "Failed to load search results for query `{:?}`: {}",
                q.clone(),
                e
            );
            return HttpResult::Err(HttpResponse::InternalServerError());
        }
    };

    return HttpResult::Ok(p);
}
