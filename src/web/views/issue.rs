use std::collections::HashMap;

use actix_web::{HttpResponse, Path, Responder, State};
use diesel::SqliteConnection;
use horrorshow::prelude::*;

use crate::process_db::models;
use crate::report::nix_patches::get_cves_from_patches;
use crate::web::AppState;

use super::queries;
use super::HttpResult;
use super::Page;
use super::Result;

#[derive(Serialize)]
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
                li { a(href=format!("https://www.vulncode-db.com/{}", self.issue.identifier.clone()))  : "vulncode-db" }
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
            return HttpResult::Err(HttpResponse::InternalServerError());
        }
    };

    return HttpResult::Ok(p);
}
