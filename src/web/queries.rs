use crate::report::nix_patches::get_cves_from_patches;
use diesel::prelude::*;
use itertools::Itertools;

use super::super::process_db::{models, schema};

pub fn get_channels(conn: &SqliteConnection) -> diesel::QueryResult<Vec<models::Channel>> {
    let channels = schema::channels::table
        .order(schema::channels::dsl::name.desc())
        .load::<models::Channel>(conn)?;
    Ok(channels)
}

pub type PackagesWithIssues = Vec<PackageWithIssues>;

#[derive(Serialize)]
pub struct PackageWithIssues {
    pub name: String,
    pub attribute_name: String,
    pub versions: Vec<PackageVersionWithIssues>,
}

#[derive(Serialize)]
pub struct PackageVersionWithIssues {
    pub version: String,
    pub issues: Vec<models::Issue>,
    pub patches: Vec<models::Patch>,
}

pub fn latest_issues_in_channel<S: AsRef<str>>(
    channel: S,
    only_unpatched: bool,
    conn: &SqliteConnection,
) -> diesel::QueryResult<(models::Commit, PackagesWithIssues)> {
    let channel = models::Channel::get(channel, conn)?;
    let channel_bump = schema::channel_bumps::table
        .filter(schema::channel_bumps::dsl::channel_id.eq(channel.id))
        .order(schema::channel_bumps::dsl::channel_bump_date.desc())
        .first::<models::ChannelBump>(conn)?;

    let commit = schema::commits::table
        .filter(schema::commits::dsl::id.eq(channel_bump.commit_id))
        .first::<models::Commit>(conn)?;

    Ok((
        commit.clone(),
        issues_in_commit(&commit, only_unpatched, conn)?,
    ))
}

pub fn issues_in_commit(
    commit: &models::Commit,
    only_unpatched: bool,
    conn: &SqliteConnection,
) -> diesel::QueryResult<PackagesWithIssues> {
    let pkg_versions = schema::package_versions_in_commits::table
        .filter(schema::package_versions_in_commits::dsl::commit_id.eq(commit.id))
        .inner_join(schema::patches_in_commits::table.inner_join(schema::patches::table))
        .inner_join(
            schema::package_versions::table
                .inner_join(schema::packages::table)
                .inner_join(schema::issues_in_versions::table.inner_join(schema::issues::table)),
        )
        .order((
            schema::packages::dsl::attribute_name.asc(),
            schema::issues::dsl::identifier.asc(),
        ))
        .select((
            schema::package_versions::all_columns,
            schema::packages::all_columns,
            schema::issues::all_columns,
            schema::patches::all_columns,
        ))
        .load::<(
            models::PackageVersion,
            models::Package,
            models::Issue,
            models::Patch,
        )>(conn)?;

    let grouped_by_pkg = pkg_versions
        .iter()
        .group_by(|(_, pkg, _, _)| pkg)
        .into_iter()
        .map(|(pkg, group)| PackageWithIssues {
            name: pkg.name.clone(),
            attribute_name: pkg.attribute_name.clone(),
            versions: group
                .group_by(|(pkg_version, _pkg, _issue, _patch)| pkg_version)
                .into_iter()
                .map(|(pkg_version, pkg_version_group)| {
                    let pkg_version_group: Vec<_> = pkg_version_group.collect();
                    let patches = pkg_version_group
                        .iter()
                        .map(|(_pkg_version, _pkg, _issue, patch)| patch.clone())
                        .collect::<Vec<models::Patch>>();
                    let patch_names: Vec<String> =
                        patches.iter().map(|p| p.name.to_string()).collect();
                    let patched_cves = get_cves_from_patches(&patch_names);

                    PackageVersionWithIssues {
                        version: pkg_version.version.clone(),
                        patches,
                        issues: pkg_version_group
                            .iter()
                            .group_by(|(_pkg_version, _pkg, issue, _patch)| issue)
                            .into_iter()
                            .filter(|(issue, _issue_group)| {
                                !(only_unpatched && patched_cves.contains(&issue.identifier))
                            })
                            .map(|(issue, _issue_group)| issue.clone())
                            .collect(),
                    }
                })
                .filter(|pkvwi| !pkvwi.issues.is_empty())
                .collect::<Vec<_>>(),
        })
        .collect::<Vec<PackageWithIssues>>();

    Ok(grouped_by_pkg)
}

#[derive(Serialize)]
pub struct PackageVersionWithPatches {
    pub version: String,
    pub patches: Vec<models::Patch>,
}

#[derive(Serialize)]
pub struct ChannelBumpWithPackageVersions {
    pub channel_bump: models::ChannelBump,
    pub commit: models::Commit,
    pub versions: Vec<PackageVersionWithPatches>,
}

#[derive(Serialize)]
pub struct ChannelBumpsWithPackagesAndPatches {
    pub channel: models::Channel,
    pub bumps_with_versions: Vec<ChannelBumpWithPackageVersions>,
}

#[derive(Serialize)]
pub struct PackageWithChannelAndVersions {
    pub package: models::Package,
    pub channels: Vec<ChannelBumpsWithPackagesAndPatches>,
}

pub fn package_versions_and_commits_for_issue(
    issue: &models::Issue,
    conn: &SqliteConnection,
) -> diesel::QueryResult<Vec<PackageWithChannelAndVersions>> {
    let pkg_versions = schema::issues_in_versions::table
        .filter(schema::issues_in_versions::dsl::issue_id.eq(issue.id))
        .inner_join(
            schema::package_versions::table
                .inner_join(schema::packages::table)
                .inner_join(
                    schema::package_versions_in_commits::table
                        .inner_join(schema::commits::table.inner_join(
                            schema::channel_bumps::table.inner_join(schema::channels::table),
                        ))
                        .inner_join(
                            schema::patches_in_commits::table.inner_join(schema::patches::table),
                        ),
                ),
        )
        .order_by((
            schema::packages::dsl::attribute_name.asc(),
            schema::channels::dsl::name.asc(),
            schema::channel_bumps::dsl::channel_bump_date.desc(),
        ))
        .select((
            schema::package_versions::all_columns,
            schema::packages::all_columns,
            schema::patches_in_commits::all_columns,
            schema::commits::all_columns,
            schema::channel_bumps::all_columns,
            schema::channels::all_columns,
            schema::patches::all_columns,
        ))
        .load::<(
            models::PackageVersion,
            models::Package,
            models::Patch2Commit,
            models::Commit,
            models::ChannelBump,
            models::Channel,
            models::Patch,
        )>(conn)?;

    // Due to the lack of `group_by` in diesel we are "forced" to do something like what is written below.
    // Ideally we would be able to make it a bit more modular and have less nesting but until I figured all
    // the required features/fields/… it is easier to keep it in this format.
    let grouped_by_pkg: Vec<PackageWithChannelAndVersions> = pkg_versions
        .iter()
        .group_by(
            |(_pkg_version, pkg, _patch2commit, _commit, _channel_bump, _channel, _patch)| pkg,
        )
        .into_iter()
        .map(|(pkg, group)| PackageWithChannelAndVersions {
            package: pkg.clone(),
            channels: group
                .group_by(
                    |(
                        _pkg_version,
                        _pkg,
                        _patch2commit,
                        _commit,
                        _channel_bump,
                        channel,
                        _patch,
                    )| channel,
                )
                .into_iter()
                .map(
                    |(channel, channel_bumps_group)| ChannelBumpsWithPackagesAndPatches {
                        channel: channel.clone(),
                        bumps_with_versions: channel_bumps_group
                            .group_by(
                                |(
                                    _pkg_version,
                                    _pkg,
                                    _patch2commit,
                                    commit,
                                    channel_bump,
                                    _channel,
                                    _patch,
                                )| (channel_bump, commit),
                            )
                            .into_iter()
                            .map(|((channel_bump, commit), versions_group)| {
                                ChannelBumpWithPackageVersions {
                                    channel_bump: channel_bump.clone(),
                                    commit: commit.clone(),
                                    versions: versions_group
                                        .group_by(
                                            |(
                                                pkg_version,
                                                _pkg,
                                                _patch2commit,
                                                _commit,
                                                _channel_bump,
                                                _channel,
                                                _patch,
                                            )| {
                                                pkg_version
                                            },
                                        )
                                        .into_iter()
                                        .map(|(version, patches_group)| {
                                            let patches = patches_group.cloned().map(
                                                |(
                                                     _pkg_version,
                                                     _pkg,
                                                     _patch2commit,
                                                     _commit,
                                                     _channel_bump,
                                                     _channel,
                                                     patch
                                                 )|
                                                    patch
                                            ).collect::<Vec<models::Patch>>();

                                            PackageVersionWithPatches {
                                                version: version.version.clone(),
                                                patches,
                                            }
                                        })
                                        .collect(),
                                }
                            })
                            .collect(),
                    },
                )
                .collect::<Vec<ChannelBumpsWithPackagesAndPatches>>(),
        })
        .collect();

    Ok(grouped_by_pkg)
}

pub struct ChannelIssueStatistics {
    pub unpatched: usize,
    pub patched: usize,
}

pub fn issue_statistics_for_channel(
    channel: &models::Channel,
    conn: &SqliteConnection,
) -> diesel::QueryResult<ChannelIssueStatistics> {
    let (_commit, issues) = latest_issues_in_channel(&channel.name, false, &conn)?;
    let (patched, unpatched) = issues
        .iter()
        .map(|pwi: &PackageWithIssues| {
            let (patched, unpatched) = pwi
                .versions
                .iter()
                .map(|pv: &PackageVersionWithIssues| {
                    let patch_names = pv
                        .patches
                        .iter()
                        .map(|p| p.name.clone())
                        .collect::<Vec<_>>();
                    let cve_patches = get_cves_from_patches(&patch_names);

                    let patched_cves = pv
                        .issues
                        .iter()
                        .filter(|i| cve_patches.contains(&i.identifier))
                        .count();
                    let unpatched_cves = pv
                        .issues
                        .iter()
                        .filter(|i| !cve_patches.contains(&i.identifier))
                        .count();

                    (patched_cves, unpatched_cves)
                })
                .fold((0, 0), |(a0, b0), (a1, b1): (usize, usize)| {
                    (a0 + a1, b0 + b1)
                });
            (patched, unpatched)
        })
        .fold((0, 0), |(a0, b0), (a1, b1)| (a0 + a1, b0 + b1));

    Ok(ChannelIssueStatistics { unpatched, patched })
}

pub fn search_packages<T: AsRef<str>>(
    term: T,
    conn: &SqliteConnection,
) -> diesel::QueryResult<Vec<models::Package>> {
    let term = term.as_ref();
    let term = term.trim();
    if term.is_empty() {
        return Ok(vec![]);
    }

    let filter = schema::packages::dsl::attribute_name
        .like(format!("%{}%", term))
        .or(schema::packages::dsl::name.like(format!("%{}%", term)));

    schema::packages::table
        .filter(filter)
        .order(schema::packages::dsl::attribute_name.asc())
        .load::<models::Package>(conn)
}

pub fn search_issues<T: AsRef<str>>(
    term: T,
    conn: &SqliteConnection,
) -> diesel::QueryResult<Vec<models::Issue>> {
    let term = term.as_ref();
    let term = term.trim();
    if term.len() == 0 {
        return Ok(vec![]);
    }

    schema::issues::table
        .filter(schema::issues::dsl::identifier.like(format!("%{}%", term)))
        .order(schema::issues::dsl::identifier.asc())
        .load::<models::Issue>(conn)
}
