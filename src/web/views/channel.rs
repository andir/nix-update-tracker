use actix_web::{HttpResponse, Path, Responder, State};
use diesel::SqliteConnection;
use horrorshow::prelude::*;
use std::collections::HashMap;

use crate::process_db::models;
use crate::report::nix_patches::get_cves_from_patches;
use crate::web::AppState;

use super::queries;
use super::HttpResult;
use super::Page;
use super::Result;

#[derive(Serialize)]
struct ChannelOverview {
    channel: models::Channel,
    commit: models::Commit,
    packages_with_issues: AnnotatedPackagesWithIssues,
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
                        td : "score";
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
                                td : "";
                                td : "";
                                //td : pver.patches.iter().fold(" ".to_string(), |a,b| a+ &b.name);
                            }
                            @ for issue in pver.issues.iter() {
                                tr {
                                    td : "";
                                    td : "";
                                    td : super::issue_link(&issue.issue);
                                    td {
                                        @ if let Some(s) = issue.score {
                                             : s
                                        }
                                    }
                                    td {
                                        @ if is_patched(&pver.patches[..], &issue.issue) {
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

pub type AnnotatedPackagesWithIssues = Vec<AnnotatedPackageWithIssues>;

#[derive(Serialize)]
pub struct AnnotatedPackageWithIssues {
    pub name: String,
    pub attribute_name: String,
    pub versions: Vec<AnnotatedPackageVersionWithIssues>,
}

#[derive(Serialize)]
pub struct AnnotatedPackageVersionWithIssues {
    pub version: String,
    pub issues: Vec<AnnotatedIssue>,
    pub patches: Vec<models::Patch>,
}

#[derive(Serialize)]
pub struct AnnotatedIssue {
    pub issue: models::Issue,
    pub score: Option<f64>,
}

fn annotate_cvss(
    nvds: &std::collections::HashMap<String, crate::report::nvd::DescriptionAndScore>,
    pkgs_with_issues: &queries::PackagesWithIssues,
) -> AnnotatedPackagesWithIssues {
    pkgs_with_issues
        .into_iter()
        .map(|pkg_with_issues| AnnotatedPackageWithIssues {
            name: pkg_with_issues.name.clone(),
            attribute_name: pkg_with_issues.attribute_name.clone(),
            versions: pkg_with_issues
                .versions
                .iter()
                .map(|version| AnnotatedPackageVersionWithIssues {
                    version: version.version.clone(),
                    patches: version.patches.clone(),
                    issues: version
                        .issues
                        .iter()
                        .cloned()
                        .map(|issue| {
                            let score = nvds.get(&issue.identifier).map_or(None, |d| d.score);

                            AnnotatedIssue { score, issue }
                        })
                        .collect(),
                })
                .collect(),
        })
        .collect()
}

impl Page<ChannelOverview> {
    fn new_for_channel<T: AsRef<str>>(
        channel: T,
        nvds: &std::collections::HashMap<String, crate::report::nvd::DescriptionAndScore>,
        conn: &SqliteConnection,
    ) -> Result<Self> {
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
                packages_with_issues: annotate_cvss(nvds, &pkgs_with_issues),
            },
            conn,
        )
    }

    fn new_for_channel_revision<T: AsRef<str>>(
        channel: T,
        revision: T,
        nvds: &std::collections::HashMap<String, crate::report::nvd::DescriptionAndScore>,
        conn: &SqliteConnection,
    ) -> Result<Self> {
        let channel = models::Channel::get(channel, conn)?;
        let commit = models::Commit::get_from_prefix(&revision, conn)?;

        // not sure why but to do it "the right way" we check if the revision actually corresponds to a
        // channel bump in the given channel...
        if let Err(e) = models::ChannelBump::get(&channel, &commit, conn) {
            error!(
                "{} is not a commit of channel {}",
                revision.as_ref(),
                &channel.name
            );
            // FIXME: cleanup the error types while migrating away from error-chain
            //return Err(e)
        }

        let pkgs_with_issues = queries::issues_in_commit(&commit, false, &conn)?;
        let pkgs_with_issues_annotated = annotate_cvss(nvds, &pkgs_with_issues);

        Page::new_with(
            format!(
                "Issues in channel {} ({})",
                channel.name.clone(),
                commit.revision
            ),
            ChannelOverview {
                channel,
                commit,
                packages_with_issues: pkgs_with_issues_annotated,
            },
            conn,
        )
    }
}

pub fn latest_issues_in_channel(
    (state, info): (State<AppState>, Path<(String,)>),
) -> impl Responder {
    let p = match Page::<ChannelOverview>::new_for_channel(&info.0, &state.nvd, &state.connection) {
        Ok(p) => p,
        Err(e) => {
            error!(
                "failed to generate page instance for latest issues in channel: {}",
                e
            );
            return HttpResult::Err(HttpResponse::InternalServerError());
        }
    };

    return HttpResult::Ok(p);
}

pub fn issues_in_channel_revision(
    (state, info): (State<AppState>, Path<(String, String)>),
) -> impl Responder {
    let p = match Page::<ChannelOverview>::new_for_channel_revision(
        &info.0,
        &info.1,
        &state.nvd,
        &state.connection,
    ) {
        Ok(p) => p,
        Err(e) => {
            error!(
                "failed to generate page instanc for revision {} of channel {}; {}",
                &info.1, &info.0, e
            );
            return HttpResult::Err(HttpResponse::InternalServerError());
        }
    };

    return HttpResult::Ok(p);
}
