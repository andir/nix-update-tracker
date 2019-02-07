use actix_web::{HttpResponse, Path, Responder, State};
use diesel::prelude::*;
use itertools::Itertools;

use super::AppState;

use super::super::process_db::{models, schema};

pub fn get_channel((state, info): (State<AppState>, Path<(String,)>)) -> impl Responder {
    let con = &state.connection;
    use self::schema::channels::dsl;

    let channel = match dsl::channels
        .filter(dsl::name.eq(&info.0))
        .first::<models::Channel>(con)
    {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to query for channel {}: {}", info.0, e);
            return HttpResponse::NotFound().finish();
        }
    };

    HttpResponse::Ok().json(channel)
}

pub fn channel_list(state: State<AppState>) -> impl Responder {
    let con = &state.connection;
    use self::schema::channels::dsl;

    let channels = if let Ok(channels) = dsl::channels.load::<models::Channel>(con) {
        channels
    } else {
        return HttpResponse::NotFound().finish();
    };

    HttpResponse::Ok().json(channels)
}

pub fn get_channel_channel_versions(
    (state, info): (State<AppState>, Path<(String,)>),
) -> impl Responder {
    let con = &state.connection;
    use self::schema::channel_versions::dsl as report_dsl;
    use self::schema::channels::dsl as chan_dsl;

    // retrieve channel first, this could be done via JOIN but diesel is weird..
    let channel = match chan_dsl::channels
        .filter(chan_dsl::name.eq(&info.0))
        .first::<models::Channel>(con)
    {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to retrieve channel {}: {}", info.0, e);
            return HttpResponse::NotFound().finish();
        }
    };

    let channel_versions = match report_dsl::channel_versions
        .filter(report_dsl::channel_id.eq(channel.id))
        .load::<models::Report>(con)
    {
        Ok(rs) => rs,
        Err(e) => {
            error!(
                "Failed to retrieve channel_versions for channel {}: {}",
                info.0, e
            );
            return HttpResponse::NotFound().finish();
        }
    };

    HttpResponse::Ok().json(channel_versions)
}

pub fn get_latest_channel_report(
    (state, info): (State<AppState>, Path<(String,)>),
) -> impl Responder {
    let con = &state.connection;
    use self::schema::channel_versions::dsl as report_dsl;
    use self::schema::channels::dsl as chan_dsl;

    // retrieve channel first, this could be done via JOIN but diesel is weird..
    let channel = match chan_dsl::channels
        .filter(chan_dsl::name.eq(&info.0))
        .first::<models::Channel>(con)
    {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to retrieve channel {}: {}", info.0, e);
            return HttpResponse::NotFound().finish();
        }
    };

    let channel_versions = match report_dsl::channel_versions
        .filter(report_dsl::channel_id.eq(channel.id))
        .order_by(report_dsl::advance_time.desc())
        .first::<models::Report>(con)
    {
        Ok(rs) => rs,
        Err(e) => {
            error!(
                "Failed to retrieve channel_versions for channel {}: {}",
                info.0, e
            );
            return HttpResponse::NotFound().finish();
        }
    };

    HttpResponse::Ok().json(channel_versions)
}

pub fn get_report((state, info): (State<AppState>, Path<(i32,)>)) -> impl Responder {
    let con = &state.connection;
    use self::schema::channel_versions::dsl as report_dsl;

    let report = match report_dsl::channel_versions
        .filter(report_dsl::id.eq(info.0))
        .first::<models::Report>(con)
    {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to query for report {}: {}", info.0, e);
            return HttpResponse::NotFound().finish();
        }
    };

    HttpResponse::Ok().json(report)
}

pub fn get_issues(state: State<AppState>) -> impl Responder {
    let con = &state.connection;
    use self::schema::issues::dsl as issue_dsl;

    let issues = match issue_dsl::issues.load::<models::Issue>(con) {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to query for issues: {}", e);
            return HttpResponse::NotFound().finish();
        }
    };

    HttpResponse::Ok().json(issues)
}

pub fn get_issue((state, info): (State<AppState>, Path<(String,)>)) -> impl Responder {
    let con = &state.connection;
    use self::schema::channel_versions as report_dsl;
    use self::schema::channels as channel_dsl;
    use self::schema::issues as issue_dsl;
    use self::schema::package_version_channel_versions as pkgvr_dsl;
    use self::schema::package_version_report_issues as pkgvri_dsl;
    use self::schema::package_versions as pkgv_dsl;
    use self::schema::packages as pkg_dsl;

    let issue = match issue_dsl::table
        .filter(issue_dsl::identifier.eq(&info.0))
        .first::<models::Issue>(con)
    {
        Ok(i) => i,
        Err(e) => {
            error!("Failed to query for issue {}: {}", &info.0, e);
            return HttpResponse::NotFound().finish();
        }
    };

    let results = match models::PackageVersionReportIssue::belonging_to(&issue)
        .inner_join(
            pkgvr_dsl::table
                .inner_join(pkgv_dsl::table.inner_join(pkg_dsl::table))
                .inner_join(report_dsl::table.inner_join(channel_dsl::table)),
        )
        .select((
            pkgvri_dsl::id,
            pkgvr_dsl::id,
            pkgv_dsl::all_columns,
            pkg_dsl::all_columns,
            report_dsl::all_columns,
            channel_dsl::all_columns,
        ))
        .load::<(
            i32,
            i32,
            models::PackageVersion,
            models::Package,
            models::Report,
            models::Channel,
        )>(con)
    {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to query for issue {}: {}", info.0, e);
            return HttpResponse::NotFound().finish();
        }
    };

    #[derive(Serialize)]
    struct IssueView {
        issue: models::Issue,
        channel_versions: Vec<ChannelView>,
    }

    #[derive(Serialize)]
    struct ReportView {
        report: models::Report,
        packages: Vec<PackageView>,
    }

    #[derive(Serialize)]
    struct ChannelView {
        channel: models::Channel,
        channel_versions: Vec<ReportView>,
    }

    #[derive(Serialize)]
    struct PackageView {
        version: String,
        name: String,
        attribute_name: String,
    }

    let results: Vec<ChannelView> = results
        .iter()
        .group_by(|(_, _, _pkgv, _pkg, _report, channel)| channel)
        .into_iter()
        .map(|(channel, group)| ChannelView {
            channel: channel.clone(),
            channel_versions: group
                .cloned()
                .group_by(|(_, _, _pkgv, _pkg, report, _)| report.clone())
                .into_iter()
                .map(|(report, inner_group)| ReportView {
                    report: report.clone(),
                    packages: inner_group
                        .map(|(_, _, pkgv, pkg, _, _)| PackageView {
                            version: pkgv.version,
                            name: pkg.name,
                            attribute_name: pkg.attribute_name,
                        })
                        .collect::<Vec<PackageView>>(),
                })
                .collect(),
        })
        .collect();

    HttpResponse::Ok().json(IssueView {
        issue: issue,
        channel_versions: results,
    })
}

pub fn get_report_issues((state, info): (State<AppState>, Path<(i32,)>)) -> impl Responder {
    let con = &state.connection;

    use self::schema::channel_versions as report_dsl;
    use self::schema::channels as channel_dsl;
    use self::schema::issues as issue_dsl;
    use self::schema::package_version_channel_versions as pkvr_dsl;
    use self::schema::package_version_report_issues as pkgvri_dsl;
    use self::schema::package_versions as pkv_dsl;
    use self::schema::packages as pkg_dsl;

    let report = match report_dsl::table
        .filter(report_dsl::id.eq(info.0))
        .first::<models::Report>(con)
    {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to query for report {}: {}", info.0, e);
            return HttpResponse::NotFound().finish();
        }
    };

    let channel = match channel_dsl::table
        .filter(channel_dsl::id.eq(report.channel_id))
        .first::<models::Channel>(con)
    {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to query for channel {}: {}", report.channel_id, e);
            return HttpResponse::NotFound().finish();
        }
    };

    let q = models::PackageVersionReport::belonging_to(&report)
        .inner_join(pkgvri_dsl::table.inner_join(issue_dsl::table))
        .inner_join(pkv_dsl::table.inner_join(pkg_dsl::table))
        .filter(pkvr_dsl::report_id.eq(report.id))
        .select((
            pkvr_dsl::all_columns,
            pkv_dsl::all_columns,
            pkg_dsl::all_columns,
            pkgvri_dsl::all_columns,
            issue_dsl::all_columns,
        ));

    #[derive(Serialize)]
    struct PackageVersionView {
        attribute_name: String,
        name: String,
        version: String,
    }

    #[derive(Serialize)]
    struct IssueView {
        identifier: String,
        packages: Vec<PackageVersionView>,
    }

    #[derive(Serialize)]
    struct ReportIssues {
        report: models::Report,
        channel: models::Channel,
        issues: Vec<IssueView>,
    }

    let f = q.load::<(
        models::PackageVersionReport,
        models::PackageVersion,
        models::Package,
        models::PackageVersionReportIssue,
        models::Issue,
    )>(con);

    let results = match f {
        Err(e) => {
            error!("Failed to query for realted package versions: {}", e);
            return HttpResponse::NotFound().finish();
        }
        Ok(s) => s,
    };

    let results = results
        .iter()
        .group_by(|(_, _, _, _, i)| i.identifier.clone())
        .into_iter()
        .map(|(identifier, group)| IssueView {
            identifier: identifier.clone(),
            packages: group
                .cloned()
                .map(|(_pkvr, pv, p, _rpvri, _)| PackageVersionView {
                    attribute_name: p.attribute_name,
                    name: p.name,
                    version: pv.version,
                })
                .collect(),
        })
        .collect::<Vec<IssueView>>();

    return HttpResponse::Ok().json(ReportIssues {
        report: report.clone(),
        channel: channel,
        issues: results,
    });
}

pub fn get_report_packages((state, info): (State<AppState>, Path<(i32,)>)) -> impl Responder {
    let con = &state.connection;

    use self::schema::channel_versions as report_dsl;
    use self::schema::channels as channel_dsl;
    use self::schema::issues as issue_dsl;
    use self::schema::package_version_channel_versions as pkvr_dsl;
    use self::schema::package_version_report_issues as pkgvri_dsl;
    use self::schema::package_versions as pkv_dsl;
    use self::schema::packages as pkg_dsl;
    use self::schema::patches as patch_dsl;

    let report = match report_dsl::table
        .filter(report_dsl::id.eq(info.0))
        .first::<models::Report>(con)
    {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to query for report {}: {}", info.0, e);
            return HttpResponse::NotFound().finish();
        }
    };

    let channel = match channel_dsl::table
        .filter(channel_dsl::id.eq(report.channel_id))
        .first::<models::Channel>(con)
    {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to query for channel {}: {}", report.channel_id, e);
            return HttpResponse::NotFound().finish();
        }
    };

    let q = models::PackageVersionReport::belonging_to(&report)
        .inner_join(pkgvri_dsl::table.inner_join(issue_dsl::table))
        .inner_join(pkv_dsl::table.inner_join(pkg_dsl::table))
        .filter(pkvr_dsl::report_id.eq(report.id))
        .select((
            pkvr_dsl::all_columns,
            pkv_dsl::all_columns,
            pkg_dsl::all_columns,
            pkgvri_dsl::all_columns,
            issue_dsl::all_columns,
        ));

    #[derive(Serialize)]
    struct IssueView {
        identifier: String,
    }

    #[derive(Serialize)]
    struct PackageVersionView {
        attribute_name: String,
        name: String,
        version: String,
        issues: Vec<IssueView>,
        patches: Vec<String>,
    }

    #[derive(Serialize)]
    struct ReportPackages {
        report: models::Report,
        channel: models::Channel,
        packages: Vec<PackageVersionView>,
    }

    let f = q.load::<(
        models::PackageVersionReport,
        models::PackageVersion,
        models::Package,
        models::PackageVersionReportIssue,
        models::Issue,
    )>(con);

    let results = match f {
        Err(e) => {
            error!("Failed to query for realted package versions: {}", e);
            return HttpResponse::NotFound().finish();
        }
        Ok(s) => s,
    };

    let query_patches = |pkgvrp: &models::PackageVersionReport| {
        models::PackageVersionReportPatch::belonging_to(pkgvrp)
            .inner_join(patch_dsl::table)
            .select((patch_dsl::all_columns,))
            .load::<(models::Patch,)>(con)
    };

    let results = results
        .iter()
        .group_by(|(pkgvr, pv, p, _, _)| (pkgvr, pv, p))
        .into_iter()
        .map(|((pkgvr, pv, p), group)| {
            let issues: Vec<IssueView> = group
                .map(|(_, _, _, _, i)| IssueView {
                    identifier: i.identifier.clone(),
                })
                .collect();

            let patches = match query_patches(pkgvr) {
                //                Err(diesel::NotFound) => vec![],
                Err(e) => {
                    error!("Failed to query related patches: {}", e);
                    //return HttpResponse::NotFound().finish();
                    vec![]
                }
                Ok(d) => d.into_iter().map(|(p,)| p.name).collect(),
            };

            PackageVersionView {
                attribute_name: p.attribute_name.clone(),
                name: p.name.clone(),
                version: pv.version.clone(),
                issues: issues,
                patches: patches,
            }
        })
        .collect::<Vec<PackageVersionView>>();

    return HttpResponse::Ok().json(ReportPackages {
        report: report.clone(),
        channel: channel,
        packages: results,
    });
}
