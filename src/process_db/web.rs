use actix_web::{http, server, App, Path, Responder, State, HttpResponse};
use diesel::sqlite::SqliteConnection;
use diesel::prelude::*;
use diesel::Connection;
use itertools::Itertools;

struct AppState {
    connection: SqliteConnection,
}


fn get_channel((state, info): (State<AppState>, Path<(String,)>)) -> impl Responder {
    let con = &state.connection;
    use super::schema::channels::dsl;
    use super::models::Channel;

    let channel = match dsl::channels
        .filter(dsl::name.eq(&info.0))
        .first::<Channel>(con) {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to query for channel {}: {}", info.0, e);
            return HttpResponse::NotFound().finish();
        }
    };

    HttpResponse::Ok().json(channel)
}

fn channel_list(state: State<AppState>) -> impl Responder {
    let con = &state.connection;
    use super::schema::channels::dsl;
    use super::models::Channel;

    let channels = if let Ok(channels) = dsl::channels.load::<Channel>(con) {
        channels
    } else {
        return HttpResponse::NotFound().finish();
    };

    HttpResponse::Ok().json(channels)
}

fn get_channel_reports((state, info): (State<AppState>, Path<(String,)>)) -> impl Responder {
    let con = &state.connection;
    use super::schema::channels::dsl as chan_dsl;
    use super::schema::reports::dsl as report_dsl;
    use super::models::{Channel, Report};

    // retrieve channel first, this could be done via JOIN but diesel is weird..
    let channel = match chan_dsl::channels
        .filter(chan_dsl::name.eq(&info.0))
        .first::<Channel>(con) {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to retrieve channel {}: {}", info.0, e);
            return HttpResponse::NotFound().finish();
        }
    };

    let reports = match report_dsl::reports
        .filter(report_dsl::channel_id.eq(channel.id))
        .load::<Report>(con) {
        Ok(rs) => rs,
        Err(e) => {
            error!("Failed to retrieve reports for channel {}: {}", info.0, e);
            return HttpResponse::NotFound().finish();
        }
    };

    HttpResponse::Ok().json(reports)
}

fn get_latest_channel_report((state, info): (State<AppState>, Path<(String,)>)) -> impl Responder {
    let con = &state.connection;
    use super::schema::channels::dsl as chan_dsl;
    use super::schema::reports::dsl as report_dsl;
    use super::models::{Channel, Report};

    // retrieve channel first, this could be done via JOIN but diesel is weird..
    let channel = match chan_dsl::channels
        .filter(chan_dsl::name.eq(&info.0))
        .first::<Channel>(con) {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to retrieve channel {}: {}", info.0, e);
            return HttpResponse::NotFound().finish();
        }
    };

    let reports = match report_dsl::reports
        .filter(report_dsl::channel_id.eq(channel.id))
        .order_by(report_dsl::advance_time.desc())
        .first::<Report>(con) {
        Ok(rs) => rs,
        Err(e) => {
            error!("Failed to retrieve reports for channel {}: {}", info.0, e);
            return HttpResponse::NotFound().finish();
        }
    };

    HttpResponse::Ok().json(reports)
}

fn get_report((state, info): (State<AppState>, Path<(i32,)>)) -> impl Responder {
    let con = &state.connection;
    use super::schema::reports::dsl as report_dsl;
    use super::models::Report;

    let report = match report_dsl::reports
        .filter(report_dsl::id.eq(info.0))
        .first::<Report>(con) {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to query for report {}: {}", info.0, e);
            return HttpResponse::NotFound().finish();
        }
    };

    HttpResponse::Ok().json(report)
}

fn get_issues(state: State<AppState>) -> impl Responder {
    let con = &state.connection;
    use super::schema::issues::dsl as issue_dsl;
    use super::models::Issue;

    let issues = match issue_dsl::issues.load::<Issue>(con) {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to query for issues: {}", e);
            return HttpResponse::NotFound().finish();
        }
    };

    HttpResponse::Ok().json(issues)
}

fn get_issue((state, info): (State<AppState>, Path<(String,)>)) -> impl Responder {
    let con = &state.connection;
    use super::schema::issues as issue_dsl;
    use super::schema::package_version_report_issues as pkgvri_dsl;
    use super::schema::package_version_reports as pkgvr_dsl;
    use super::schema::package_versions as pkgv_dsl;
    use super::schema::packages as pkg_dsl;
    use super::schema::reports as report_dsl;
    use super::schema::channels as channel_dsl;

    use super::models::{Issue, PackageVersionReportIssue, PackageVersion, Package, Report, Channel};

    let issue = match issue_dsl::table
        .filter(issue_dsl::identifier.eq(&info.0))
        .first::<Issue>(con) {
        Ok(i) => i,
        Err(e) => {
            error!("Failed to query for issue {}: {}", &info.0, e);
            return HttpResponse::NotFound().finish();
        }
    };

    let results = match PackageVersionReportIssue::belonging_to(&issue)
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
        .load::<(i32, i32, PackageVersion, Package, Report, Channel)>(con) {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to query for issue {}: {}", info.0, e);
            return HttpResponse::NotFound().finish();
        }
    };

    #[derive(Serialize)]
    struct IssueView {
        issue: Issue,
        reports: Vec<ChannelView>,
    }

    #[derive(Serialize)]
    struct ReportView {
        report: Report,
        packages: Vec<PackageView>,
    }

    #[derive(Serialize)]
    struct ChannelView {
        channel: Channel,
        reports: Vec<ReportView>,
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
        .map(|(channel, group)| {
            ChannelView {
                channel: channel.clone(),
                reports: group
                    .cloned()
                    .group_by(|(_, _, _pkgv, _pkg, report, _)| report.clone())
                    .into_iter()
                    .map(|(report, inner_group)| {
                        ReportView {
                            report: report.clone(),
                            packages: inner_group
                                .map(|(_, _, pkgv, pkg, _, _)| {
                                    PackageView {
                                        version: pkgv.version,
                                        name: pkg.name,
                                        attribute_name: pkg.attribute_name,
                                    }
                                })
                                .collect::<Vec<PackageView>>(),
                        }
                    })
                    .collect(),
            }
        })
        .collect();

    HttpResponse::Ok().json(IssueView {
        issue: issue,
        reports: results,
    })
}

fn get_report_issues((state, info): (State<AppState>, Path<(i32,)>)) -> impl Responder {
    let con = &state.connection;
    use super::schema::reports as report_dsl;
    use super::schema::package_version_reports as pkvr_dsl;
    use super::schema::package_versions as pkv_dsl;
    use super::schema::packages as pkg_dsl;
    use super::schema::issues as issue_dsl;
    use super::schema::channels as channel_dsl;
    use super::schema::package_version_report_issues as pkgvri_dsl;
    use super::models::{Report, PackageVersionReport, PackageVersion, Package,
                        PackageVersionReportIssue, Issue, Channel};

    let report = match report_dsl::table
        .filter(report_dsl::id.eq(info.0))
        .first::<Report>(con) {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to query for report {}: {}", info.0, e);
            return HttpResponse::NotFound().finish();
        }
    };

    let channel = match channel_dsl::table
        .filter(channel_dsl::id.eq(report.channel_id))
        .first::<Channel>(con) {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to query for channel {}: {}", report.channel_id, e);
            return HttpResponse::NotFound().finish();
        }
    };

    let q = PackageVersionReport::belonging_to(&report)
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
        report: Report,
        channel: Channel,
        issues: Vec<IssueView>,
    }

    let f = q.load::<(PackageVersionReport, PackageVersion, Package, PackageVersionReportIssue, Issue)>(con);

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
        .map(|(identifier, group)| {
            IssueView {
                identifier: identifier.clone(),
                packages: group
                    .cloned()
                    .map(|(_pkvr, pv, p, _rpvri, _)| {
                        PackageVersionView {
                            attribute_name: p.attribute_name,
                            name: p.name,
                            version: pv.version,
                        }
                    })
                    .collect(),
            }
        })
        .collect::<Vec<IssueView>>();

    return HttpResponse::Ok().json(ReportIssues {
        report: report.clone(),
        channel: channel,
        issues: results,
    });
}

fn get_report_packages((state, info): (State<AppState>, Path<(i32,)>)) -> impl Responder {
    let con = &state.connection;
    use super::schema::reports as report_dsl;
    use super::schema::package_version_reports as pkvr_dsl;
    use super::schema::package_versions as pkv_dsl;
    use super::schema::packages as pkg_dsl;
    use super::schema::issues as issue_dsl;
    use super::schema::channels as channel_dsl;
    use super::schema::package_version_report_issues as pkgvri_dsl;
    use super::models::{Report, PackageVersionReport, PackageVersion, Package,
                        PackageVersionReportIssue, Issue, Channel};

    let report = match report_dsl::table
        .filter(report_dsl::id.eq(info.0))
        .first::<Report>(con) {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to query for report {}: {}", info.0, e);
            return HttpResponse::NotFound().finish();
        }
    };

    let channel = match channel_dsl::table
        .filter(channel_dsl::id.eq(report.channel_id))
        .first::<Channel>(con) {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to query for channel {}: {}", report.channel_id, e);
            return HttpResponse::NotFound().finish();
        }
    };

    let q = PackageVersionReport::belonging_to(&report)
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
    }

    #[derive(Serialize)]
    struct ReportPackages {
        report: Report,
        channel: Channel,
        packages: Vec<PackageVersionView>,
    }


    let f = q.load::<(PackageVersionReport, PackageVersion, Package, PackageVersionReportIssue, Issue)>(con);

    let results = match f {
        Err(e) => {
            error!("Failed to query for realted package versions: {}", e);
            return HttpResponse::NotFound().finish();
        }
        Ok(s) => s,
    };

    let results = results
        .iter()
        .group_by(|(_, pv, p, _, _)| (pv, p))
        .into_iter()
        .map(|((pv, p), group)| {
            PackageVersionView {
                attribute_name: p.attribute_name.clone(),
                name: p.name.clone(),
                version: pv.version.clone(),
                issues: group
                    .map(|(_, _, _, _, i)| {
                        IssueView { identifier: i.identifier.clone() }
                    })
                    .collect(),
            }
        })
        .collect::<Vec<PackageVersionView>>();

    return HttpResponse::Ok().json(ReportPackages {
        report: report.clone(),
        channel: channel,
        packages: results,
    });
}

fn index(_: State<AppState>) -> impl Responder {
    HttpResponse::Ok().content_type("text/html").body(
        include_str!(
            "../../static/index.html"
        ),
    )
}

pub fn serve(listen_address: &str, dbfile: &str) {
    let dbfile = String::from(dbfile);
    server::new(move || {
        let context = AppState { connection: SqliteConnection::establish(&dbfile).unwrap() };
        let app = App::with_state(context)
            .resource("/", |r| r.get().with(index))
            .resource("/channels", |r| {
                r.method(http::Method::GET).with(channel_list)
            })
            .resource("/channels/{id}", |r| {
                r.method(http::Method::GET).with(get_channel)
            })
            .resource("/channels/{id}/reports", |r| {
                r.method(http::Method::GET).with(get_channel_reports)
            })
            .resource("/channels/{id}/reports/latest", |r| {
                r.method(http::Method::GET).with(get_latest_channel_report)
            })
            .resource("/reports/{report_id}", |r| {
                r.method(http::Method::GET).with(get_report)
            })
            .resource("/reports/{report_id}/issues", |r| {
                r.method(http::Method::GET).with(get_report_issues)
            })
            .resource("/reports/{report_id}/packages", |r| {
                r.method(http::Method::GET).with(get_report_packages)
            })
            .resource("/issues", |r| r.method(http::Method::GET).with(get_issues))
            .resource("/issues/{issue_identifier}", |r| {
                r.method(http::Method::GET).with(get_issue)
            });

        return app;
    }).bind(listen_address)
        .unwrap()
        .run();
}
