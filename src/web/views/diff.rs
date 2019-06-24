use actix_web::{HttpResponse, Path, Query, Responder, State};
use diesel::SqliteConnection;
use horrorshow::prelude::*;

use crate::process_db::models;
use crate::web::queries;
use crate::web::views::{issue_link, HttpResult, Page, Result};
use crate::web::AppState;

use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub struct DiffT<T> {
    common: Vec<T>,
    old: Vec<T>,
    new: Vec<T>,
}

fn diff<T>(a: &[T], b: &[T]) -> DiffT<T>
where
    T: std::cmp::Eq + std::hash::Hash + Clone,
{
    use std::collections::HashSet;
    use std::iter::FromIterator;
    let a: HashSet<T> = HashSet::from_iter(a.into_iter().cloned());
    let b: HashSet<T> = HashSet::from_iter(b.into_iter().cloned());

    let common = a.intersection(&b).cloned().collect();
    let old = a.difference(&b).cloned().collect();
    let new = b.difference(&a).cloned().collect();

    return DiffT::<T> { common, old, new };
}

#[derive(PartialEq, Eq, Clone, Debug, Serialize, Hash)]
struct IssueEntry {
    attribute_name: String,
    version: String,
    issue: String,
}

#[derive(PartialEq, Eq, Clone, Debug, Serialize, Hash)]
struct PatchEntry {
    attribute_name: String,
    version: String,
    patch: String,
}

#[derive(Debug, Serialize)]
struct DiffView {
    old: models::Commit,
    new: models::Commit,
    issues: DiffT<IssueEntry>,
    patches: DiffT<PatchEntry>,
}

impl DiffView {
    pub fn new<T: AsRef<str>>(a: T, b: T, conn: &SqliteConnection) -> Result<Self> {
        let get_issues_in_commit = |revision, conn: &SqliteConnection| -> Result<_> {
            let commit = models::Commit::get(&revision, conn)?;
            let issues = queries::issues_in_commit(&commit, true, conn)?;
            Ok((commit, issues))
        };

        let (a, old) = get_issues_in_commit(a, conn)?;
        let (b, new) = get_issues_in_commit(b, conn)?;

        let map = |pswi: queries::PackagesWithIssues| -> (Vec<IssueEntry>, Vec<PatchEntry>) {
            use std::iter::Flatten;

            let (issuesl, patchesl): (Vec<Vec<_>>, Vec<Vec<_>>) = pswi
                .iter()
                .map(|pwi| {
                    let (issues, patches): (Vec<_>, Vec<_>) = pwi
                        .versions
                        .iter()
                        .map(|pkv| {
                            (
                                pkv.issues
                                    .iter()
                                    .map(|i| IssueEntry {
                                        attribute_name: pwi.attribute_name.clone(),
                                        version: pkv.version.clone(),
                                        issue: i.identifier.clone(),
                                    })
                                    .collect::<Vec<_>>(),
                                pkv.patches
                                    .iter()
                                    .map(|p| PatchEntry {
                                        attribute_name: pwi.attribute_name.clone(),
                                        version: pkv.version.clone(),
                                        patch: p.name.clone(),
                                    })
                                    .collect::<Vec<_>>(),
                            )
                        })
                        .unzip();

                    (
                        issues.into_iter().flatten().collect(),
                        patches.into_iter().flatten().collect(),
                    )
                })
                .unzip();

            (
                issuesl.into_iter().flatten().collect(),
                patchesl.into_iter().flatten().collect(),
            )
        };

        let (a_issues, a_patches) = map(old);
        let (b_issues, b_patches) = map(new);

        Ok(DiffView {
            old: a,
            new: b,
            issues: diff(&a_issues[..], &b_issues[..]),
            patches: diff(&a_patches[..], &b_patches[..]),
        })
    }
}

impl Page<DiffView> {
    fn new_diff<T: AsRef<str>>(a: T, b: T, conn: &SqliteConnection) -> Result<Self> {
        let dv = DiffView::new(a, b, conn)?;

        Page::new_with(
            format!("Diff between {} and {}", dv.old.revision, dv.new.revision),
            dv,
            conn,
        )
    }
}

impl RenderOnce for DiffView {
    fn render_once(self, tmpl: &mut TemplateBuffer) {
        tmpl << html! {
            h1 {
                : "Diff between ";
                : self.old.revision.clone();
                : " & ";
                : self.new.revision.clone();
            }

            table(class="table") {
                tbody {
                    tr {
                        td : "Old Revision";
                        td : self.old.revision.clone();
                    }
                    tr {
                        td : "New Revision";
                        td : self.new.revision.clone();
                    }
                }
            }

            h2 {
                : "Solved Issues";
            }

            table(class="table") {
                thead {
                    tr {
                        td : "Attribute Name";
                        td : "Version";
                        td : "Issue";
                    }
                }
                tbody {
                    @ for issue in self.issues.old.iter() {
                        tr {
                            td : issue.attribute_name.clone();
                            td : issue.version.clone();
                            td : issue.issue.clone();
                        }
                    }
                }
            }
            h2 {
                : "New Issues";
            }

            table(class="table") {
                thead {
                    tr {
                        td : "Attribute Name";
                        td : "Version";
                        td : "Patches";
                    }
                }
                tbody {
                    @ for issue in self.issues.new.iter() {
                        tr {
                            td : issue.attribute_name.clone();
                            td : issue.version.clone();
                            td : issue.issue.clone();
                        }
                    }
                }
            }


        }
    }
}

pub fn diff_revisions((state, info): (State<AppState>, Path<(String, String)>)) -> impl Responder {
    let (a, b) = (&info.0, &info.1);

    let p = match Page::<DiffView>::new_diff(a, b, &state.connection) {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to laod diff: {}", e);
            return HttpResult::Err(HttpResponse::InternalServerError());
        }
    };

    return HttpResult::Ok(p);
}
