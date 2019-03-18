use actix_web::HttpResponse;
use actix_web::Query;
use actix_web::Responder;
use actix_web::State;
use diesel::prelude::*;
use horrorshow::{Render, RenderOnce, Template, TemplateBuffer};

use process_db::models;
use web::queries;
use web::views::Result;
use web::views::{issue_link, Page};
use web::AppState;

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

fn value_fn(row: &models::Issue) -> String {
    row.identifier.clone()
}

impl RenderOnce for IssueSearchResults {
    fn render_once(self, tmpl: &mut TemplateBuffer) {
        use web::views::table::{AsTable, Column};

        let fun = |tmpl| {
            self.0.as_table(
                tmpl,
                &[
                    Column::new("Identifier", issue_fn),
                    Column::new("Description", value_fn),
                ],
            )
        };

        tmpl << html! {
                    div {
                        h2 : "Issues";
                        |tmpl| fun(tmpl)
        //                table(class="table") {
        //                    thead {
        //                        tr {
        //                           td {
        //                            : "Identifier";
        //                           }
        //                           td {
        //                            : "Description";
        //                           }
        //                        }
        //                    }
        //                    tbody {
        //                        @ for issue in self.0.iter() {
        //                            tr {
        //                                td {
        //                                  : issue_link(issue.clone());
        //                                }
        //                                td {
        //                                  : "FIXME";
        //                                }
        //                            }
        //                        }
        //                    }
        //                }
                    }
                };
    }
}

impl RenderOnce for PackageSearchResults {
    fn render_once(self, tmpl: &mut TemplateBuffer) {
        tmpl << html! {
            div {
                h2 : "Packages";
                table(class="table") {
                    thead {
                        tr {
                            td {
                                : "Attribute";
                            }
                            td {
                                : "Name";
                            }
                        }
                    }
                }
                tbody {
                    @for package in self.0.iter() {
                        tr {
                            td {
                               : package.attribute_name.clone();
                            }
                            td {
                               : package.name.clone();
                            }
                        }
                    }
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
