use horrorshow::helper::doctype;
use horrorshow::prelude::*;

use actix_web::Path;
use actix_web::{HttpResponse, Responder, State};
use diesel::prelude::SqliteConnection;
use process_db::models;
use report::nix_patches::get_cves_from_patches;
use std::collections::HashMap;
use web::{queries, AppState};

error_chain! {
    foreign_links {
        DieselError(diesel::result::Error);
    }
}

struct Page<C> {
    title: String,
    content: C,
}

impl<C> RenderOnce for Page<C>
where
    C: RenderOnce,
{
    fn render_once(self, tmpl: &mut TemplateBuffer) {
        tmpl << html! {
            : doctype::HTML;
            html {
                head {
                    title : &self.title;

                    style {
                        : r#"
                            body {
                                margin-left: auto;
                                margin-right: auto;
                                max-width: 60em;
                            }

                            table {
                                border: 1px solid grey;
                            }
                        "#;
                    }
                }
                body {
                    div : self.content;
                }
            }
        };
    }
}

struct Index {}

impl RenderOnce for Index {
    fn render_once(self, tmpl: &mut TemplateBuffer) {
        tmpl << html! {
            p {
                : "blafoo"
            }
            ul {
                li : "foo";
                li : "bar";
                li : "baz";
            }
        };
    }
}

impl Page<Index> {
    fn new() -> Self {
        Page {
            title: "Nix Vulnerability Scanner".to_string(),
            content: Index {},
        }
    }
}

struct IssueOverview {
    channel: String,
    commit: String,
    packages_with_issues: queries::PackagesWithIssues,
}

fn issue_link(issue: models::Issue) -> Box<Render> {
    let link = format!("/issues/{}", issue.identifier.clone());
    box_html! {
        a(href=link.clone()) : issue.identifier.clone()
    }
}

impl RenderOnce for IssueOverview {
    fn render_once(self, tmpl: &mut TemplateBuffer) {
        let addr = format!(
            "https://github.com/nixos/nixpkgs-channels/commit/{}",
            self.commit
        );

        let is_patched = |patches: &[models::Patch], issue: &models::Issue| {
            let names: Vec<_> = patches.iter().map(|v| v.name.clone()).collect();
            get_cves_from_patches(&names).contains(&issue.identifier)
        };

        tmpl << html! {
            h1 {
                : "Issues in channel ";
                a(href=addr) : self.channel;
            }
            table {
                thead {
                    tr {
                        td : "attribute name";
                        td : "version";
                        td : "issues";
                    }
                }
                tbody(style="border: 1px solid black;") {
                    @ for entry in self.packages_with_issues.iter() {
                        tr(rowspawn=entry.versions.len() + entry.versions.iter().fold(0, |a, b| a + b.issues.len())) {
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

impl Page<IssueOverview> {
    fn new_for_channel<T: AsRef<str>>(channel: T, conn: &SqliteConnection) -> Result<Self> {
        let (commit, pkgs_with_issues) = queries::latest_issues_in_channel(channel.as_ref(), conn)?;
        Ok(Page {
            title: format!(
                "Issues in channel {} ({})",
                channel.as_ref(),
                commit.revision
            ),
            content: IssueOverview {
                channel: channel.as_ref().to_string(),
                commit: commit.revision,
                packages_with_issues: pkgs_with_issues,
            },
        })
    }
}

pub fn index(state: State<AppState>) -> impl Responder {
    let p: Page<Index> = Page::new();
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
    let p = match Page::<IssueOverview>::new_for_channel(&info.0, &state.connection) {
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
        use super::super::report::nvd::NVDSearchable;

        let description = nvds.get(identifier.as_ref()).map(|s| s.clone());

        let issue = models::Issue::get(&identifier, conn)?;

        Ok(Page {
            title: format!("Issue {}", identifier.as_ref()),
            content: IssueView {
                issue: issue.clone(),
                packages: queries::package_versions_and_commits_for_issue(&issue, conn)?,
                description,
            },
        })
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
                li { a(href=format!("https://cve.mitre.org/cgi-bin/cvename.cgi?name={}", self.issue.identifier.clone())) : "MITRE" }
                li { a(href=format!("https://nvd.nist.gov/vuln/detail/{}", self.issue.identifier.clone())) : "NVD" }
                li { a(href=format!("https://marc.info/?s={}&l=oss-security", self.issue.identifier.clone())) : "oss-sec" }
                li { a(href=format!("https://security-tracker.debian.org/tracker/{}", self.issue.identifier.clone())) : "Debian" }
            }

            @ if let Some(d) = self.description {
                p : d
            }

            table {
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
