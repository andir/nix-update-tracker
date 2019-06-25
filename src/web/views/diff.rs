use actix_web::{HttpResponse, Path, Query, Responder, State};
use diesel::SqliteConnection;
use horrorshow::prelude::*;

use crate::process_db::models;
use crate::web::queries;
use crate::web::views::{issue_link, HttpResult, Page, Result};
use crate::web::AppState;

use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub struct DiffT<T>
where
    T: PartialEq + Ord,
{
    common: Vec<T>,
    old: Vec<T>,
    new: Vec<T>,
}

impl<T> DiffT<T>
where
    T: PartialEq + Ord,
{
    fn sort(mut self) -> Self {
        self.old.sort();
        self.new.sort();
        self.common.sort();
        self
    }
}

impl<T> DiffT<T>
where
    T: PartialEq + Ord + Clone,
    //    T: std::cmp::Eq + std::hash::Hash + Clone,
{
    pub fn new<ValFn, V>(a: &[T], b: &[T], get_value: ValFn) -> DiffT<T>
    where
        ValFn: Fn(&T) -> V,
        V: std::cmp::Eq + std::hash::Hash + Clone,
    {
        use std::collections::{HashMap, HashSet};
        use std::iter::FromIterator;

        let a: HashMap<V, T> = a
            .into_iter()
            .map(|v| (get_value(v), v.clone()))
            .collect::<HashMap<_, _>>();
        let b = b
            .into_iter()
            .map(|v| (get_value(v), v.clone()))
            .collect::<HashMap<_, _>>();

        let a_keys: HashSet<&V> = a.keys().into_iter().collect::<HashSet<_>>();
        let b_keys: HashSet<&V> = b.keys().into_iter().collect::<HashSet<_>>();

        let common = a_keys
            .intersection(&b_keys)
            .map(|key| {
                let t = a.get(key).expect("Element vanished from hashmap");
                t
            })
            .cloned()
            .collect::<Vec<_>>();
        let old = a_keys
            .difference(&b_keys)
            .map(|key| a.get(key).expect("Element vanished from hashmap"))
            .cloned()
            .collect();
        let new = b_keys
            .difference(&a_keys)
            .map(|key| b.get(key).expect("Element vanished from hashmap"))
            .cloned()
            .collect();

        return (Self { common, old, new }).sort();
    }
}

#[derive(PartialEq, Eq, Clone, PartialOrd, Ord, Debug, Serialize, Hash)]
struct IssueEntry {
    attribute_name: String,
    version: String,
    issue: String,
}

#[derive(PartialEq, Eq, Clone, PartialOrd, Ord, Debug, Serialize, Hash)]
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

        let issues_diff = DiffT::new(&a_issues[..], &b_issues[..], |ie: &IssueEntry| {
            (ie.attribute_name.clone(), ie.issue.clone())
        });
        let patches_diff = DiffT::new(&a_patches[..], &b_patches[..], |pe: &PatchEntry| {
            (pe.attribute_name.clone(), pe.patch.clone())
        });

        Ok(DiffView {
            old: a,
            new: b,
            issues: issues_diff,
            patches: patches_diff,
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
        let should_show_attr_name = |s, o: Option<&IssueEntry>| match o {
            None => true,
            Some(x) if &x.attribute_name == s => false,
            _ => true,
        };

        let should_show_version = |attribute_name, version, o: Option<&IssueEntry>| match o {
            None => true,
            Some(x) if &x.version == version && attribute_name == &x.attribute_name => false,
            _ => true,
        };

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
                    @ for (issue, prev) in self.issues.old.iter().zip(std::iter::once(None).chain(self.issues.old.iter().map(|v| Some(v)))) {
                        tr {
                            td {
                                @ if should_show_attr_name(&issue.attribute_name, prev) {
                                   : issue.attribute_name.clone()
                                }
                            }
                            td {
                                @ if should_show_version(&issue.attribute_name, &issue.version, prev) {
                                    : issue.version.clone()
                                }
                            }
                            td : super::issue_link(issue.issue.clone())
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
                    @ for (issue, prev) in self.issues.new.iter().zip(std::iter::once(None).chain(self.issues.new.iter().map(|v| Some(v)))) {
                        tr {
                            td {
                                @ if should_show_attr_name(&issue.attribute_name, prev) {
                                   : issue.attribute_name.clone()
                                }
                            }
                            td {
                                @ if should_show_version(&issue.attribute_name, &issue.version, prev) {
                                    : issue.version.clone()
                                }
                            }
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
