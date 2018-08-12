use actix_web::{http, server, App, Path, Responder, State, HttpResponse, HttpRequest};
use diesel::sqlite::SqliteConnection;
use diesel::prelude::*;
use diesel::Connection;
use itertools::Itertools;
use rayon::prelude::*;

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
            });

        return app;
    }).bind(listen_address)
        .unwrap()
        .run();
}
