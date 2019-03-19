use std::collections::HashMap;

use actix_web::Path;
use actix_web::{HttpResponse, Responder, State};
use diesel::prelude::SqliteConnection;
use horrorshow::helper::doctype;
use horrorshow::prelude::*;

use crate::process_db::models;
use crate::report::nix_patches::get_cves_from_patches;

use super::{queries, AppState};

pub use self::search::show_search_results;

mod search;
mod table;

error_chain! {
    foreign_links {
        DieselError(diesel::result::Error);
    }
}

struct Page<C> {
    title: String,
    channels: Vec<models::Channel>,
    content: C,
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

struct Index {
    statistics: Vec<(String, queries::ChannelIssueStatistics)>,
}

impl RenderOnce for Index {
    fn render_once(self, tmpl: &mut TemplateBuffer) {
        use crate::web::views::table::{AsTable, Column};

        tmpl << html! {
            |t| {
                type Row = (String, queries::ChannelIssueStatistics);
                self.statistics.as_table(t, &[
                    Column::<Row>::new("channel", &|(k, v)| { Box::new(k.to_string()) }),
                    Column::new("patched", &|(k, v)| Box::new(v.patched)),
                    Column::new("open", &|(k, v)| Box::new(v.unpatched)),
                ])
            }
        };
    }
}

impl Page<Index> {
    fn new(conn: &SqliteConnection) -> Result<Self> {
        let channels = queries::get_channels(&conn)?;
        let statistics = channels
            .iter()
            .map(|channel| {
                let stats = queries::issue_statistics_for_channel(&channel, &conn)?;
                Ok((channel.name.clone(), stats))
            })
            .collect::<diesel::QueryResult<Vec<(_, _)>>>()?;

        Page::new_with(
            "Nix Vulnerability Scanner".to_string(),
            Index { statistics },
            conn,
        )
    }
}

struct ChannelOverview {
    channel: models::Channel,
    commit: models::Commit,
    packages_with_issues: queries::PackagesWithIssues,
}

fn issue_link(issue: models::Issue) -> Box<Render> {
    let link = format!("/issues/{}", issue.identifier.clone());
    box_html! {
        a(href=link.clone()) : issue.identifier.clone()
    }
}

impl RenderOnce for ChannelOverview {
    fn render_once(self, tmpl: &mut TemplateBuffer) {
        let addr = format!(
            "https://github.com/nixos/nixpkgs-channels/commit/{}",
            self.commit.revision
        );
        let short_ref = self.commit.revision[..7].to_owned();

        let is_patched = |patches: &[models::Patch], issue: &models::Issue| {
            let names: Vec<_> = patches.iter().map(|v| v.name.clone()).collect();
            get_cves_from_patches(&names).contains(&issue.identifier)
        };

        tmpl << html! {
            h1 {
                : "Issues in channel ";
                : self.channel.name.clone();
                : " - ";
                a(href=addr) {
                    : short_ref;
                }
            }

            table(class="table") {
                tbody {
                    tr {
                        td : "Revision";
                        td : self.commit.revision.clone();
                    }
                    tr {
                        td : "Commit date";
                        td : self.commit.commit_time;
                    }
                }
            }

            h2 {
                : "Packages";
            }

            table(class="table") {
                thead {
                    tr {
                        td : "attribute name";
                        td : "version";
                        td : "issues";
                    }
                }
                tbody {
                    @ for entry in self.packages_with_issues.iter() {
                        tr(rowspan=entry.versions.len() + entry.versions.iter().fold(0, |a, b| a + b.issues.len())) {
                            td : entry.attribute_name.clone();
                        }
                        @ for pver in entry.versions.iter() {
                            tr {
                                td : "";
                                td : pver.version.clone();
                                td : "";
                                td : "";
                                //td : pver.patches.iter().fold(" ".to_string(), |a,b| a+ &b.name);
                            }
                            @ for issue in pver.issues.iter() {
                                tr {
                                    td : "";
                                    td : "";
                                    td : issue_link((*issue).clone());
                                    td {
                                        @ if is_patched(&pver.patches[..], &issue) {
                                            span(style="color: green;"): "patched";
                                        } else {
                                            span(style="color: red;"): "vulnerable";
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        };
    }
}

impl Page<ChannelOverview> {
    fn new_for_channel<T: AsRef<str>>(channel: T, conn: &SqliteConnection) -> Result<Self> {
        let channel = models::Channel::get(channel, conn)?;
        let (commit, pkgs_with_issues) =
            queries::latest_issues_in_channel(&channel.name, false, conn)?;
        Page::new_with(
            format!(
                "Issues in channel {} ({})",
                channel.name.clone(),
                commit.revision
            ),
            ChannelOverview {
                channel,
                commit,
                packages_with_issues: pkgs_with_issues,
            },
            conn,
        )
    }
}

pub fn index(state: State<AppState>) -> impl Responder {
    let p: Page<Index> = match Page::new(&state.connection) {
        Ok(p) => p,
        Err(e) => {
            error!("failed to generate index page instance: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    match p.into_string() {
        Ok(o) => HttpResponse::Ok().body(o),
        Err(e) => {
            error!("Failed to render index: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub fn latest_issues_in_channel(
    (state, info): (State<AppState>, Path<(String,)>),
) -> impl Responder {
    let p = match Page::<ChannelOverview>::new_for_channel(&info.0, &state.connection) {
        Ok(p) => p,
        Err(e) => {
            error!(
                "failed to generate page instance for latest issues in channel: {}",
                e
            );
            return HttpResponse::InternalServerError().finish();
        }
    };

    match p.into_string() {
        Ok(s) => HttpResponse::Ok().body(s),
        Err(e) => {
            error!(
                "Failed to render latest issues for channel template to string: {}",
                e
            );
            HttpResponse::InternalServerError().finish()
        }
    }
}

struct IssueView {
    issue: models::Issue,
    packages: Vec<queries::PackageWithChannelAndVersions>,
    description: Option<String>,
}

impl Page<IssueView> {
    fn new_for_issue<T: AsRef<str>>(
        identifier: T,
        conn: &SqliteConnection,
        nvds: &HashMap<String, String>,
    ) -> Result<Self> {
        let description = nvds.get(identifier.as_ref()).cloned();

        let issue = models::Issue::get(&identifier, conn)?;

        Page::new_with(
            format!("Issue {}", identifier.as_ref()),
            IssueView {
                issue: issue.clone(),
                packages: queries::package_versions_and_commits_for_issue(&issue, conn)?,
                description,
            },
            conn,
        )
    }
}

impl RenderOnce for IssueView {
    fn render_once(self, tmpl: &mut TemplateBuffer) {
        let issue = self.issue.clone();

        let is_patched = |patches: &[models::Patch]| {
            let names: Vec<_> = patches.iter().map(|v| v.name.clone()).collect();
            get_cves_from_patches(&names).contains(&issue.identifier)
        };

        tmpl << html! {
            h1 : issue.identifier.clone();

            ul {
                li { a(href=format!("https://github.com/nixos/nixpkgs/issues?utf8=%E2%9C%93&q={}", self.issue.identifier.clone()))  : "nixpkgs issues" }
                li { a(href=format!("https://github.com/search?q={}", self.issue.identifier.clone()))  : "GitHub" }
                li { a(href=format!("https://cve.mitre.org/cgi-bin/cvename.cgi?name={}", self.issue.identifier.clone())) : "MITRE" }
                li { a(href=format!("https://nvd.nist.gov/vuln/detail/{}", self.issue.identifier.clone())) : "NVD" }
                li { a(href=format!("https://marc.info/?s={}&l=oss-security", self.issue.identifier.clone())) : "oss-sec" }
                li { a(href=format!("https://security-tracker.debian.org/tracker/{}", self.issue.identifier.clone())) : "Debian" }
            }

            @ if let Some(d) = self.description {
                p : d
            }

            table(class="table") {
                thead {
                    tr {
                        td : "package";
                        td : "channel";
                        td : "channel version";
                        td : "pkg version";
                        td : "patches";
                        td : "status";
                    }
                }
                tbody {
                    @ for package in self.packages.iter() {
                        tr {
                            td : package.package.attribute_name.clone();
                        }

                        @ for channel in package.channels.iter() {
                            tr {
                                td : "";
                                td : channel.channel.name.clone();
                            }

                            @ for bump in channel.bumps_with_versions.iter() {
                                tr {
                                    td : "";
                                    td : "";
                                    td {
                                        : bump.channel_bump.channel_bump_date.clone();
                                        : " (";
                                        a(href=format!("https://github.com/nixos/nixpkgs-channels/commit/{}", bump.commit.revision.clone())) {
                                            : bump.commit.revision[..6].to_string();
                                        }
                                        : ")";
                                    }
                                    // there is always just one version:
                                    td: bump.versions[0].version.clone();
                                    td  {
                                        @ if is_patched(&bump.versions[0].patches[..]) {
                                            span(style="color: green;") : "patched";
                                        } else {
                                            span(Style="color: red;") : "vulnerable";
                                        }
                                    }

                                }
                            }
                        }
                    }
                }
            }

        };
    }
}

pub fn show_issue((state, info): (State<AppState>, Path<(String,)>)) -> impl Responder {
    let p = match Page::<IssueView>::new_for_issue(&info.0, &state.connection, &state.nvd) {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to load details for issue {}: {}", info.0, e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    match p.into_string() {
        Ok(s) => HttpResponse::Ok().body(s),
        Err(e) => {
            error!("Failed to render template for issue {}: {}", info.0, e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
