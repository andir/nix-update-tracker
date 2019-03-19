use crate::report::nvd::NVDSearchable;
use actix_web::middleware::DefaultHeaders;
use actix_web::{http, server, App, HttpResponse, Responder, State};
use diesel::sqlite::SqliteConnection;
use diesel::Connection;

mod queries;
mod v2;
mod views;

pub struct AppState {
    connection: SqliteConnection,
    nvd: std::collections::HashMap<String, String>,
}

//fn index(_: State<AppState>) -> impl Responder {
//    HttpResponse::Ok()
//        .content_type("text/html")
//        .body(include_str!("../../static/index.html"))
//}

fn static_bootstrap_css(_: State<AppState>) -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/css")
        .body(include_str!(
            "../../static/bootstrap-4.3.1/css/bootstrap.min.css"
        ))
}

fn static_bootstrap(_: State<AppState>) -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/javascript")
        .body(include_str!(
            "../../static/bootstrap-4.3.1/js/bootstrap.bundle.min.js"
        ))
}

fn static_jquery(_: State<AppState>) -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/javscript")
        .body(include_str!("../../static/jquery-3.3.1.slim.min.js"))
}

fn static_logo(_: State<AppState>) -> impl Responder {
    let body = include_bytes!("../../logo.png");
    let body: Vec<u8> = body.to_vec();
    HttpResponse::Ok().content_type("image/png").body(body)
}

pub fn serve(listen_address: &str, dbfile: &str) {
    let dbfile = String::from(dbfile);
    let nvds = super::report::nvd::get_nvd_databases(2012, false).unwrap();
    let nvds = nvds.get_descriptions();

    server::new(move || {
        let context = AppState {
            connection: SqliteConnection::establish(&dbfile).unwrap(),
            nvd: nvds.clone(),
        };
        App::with_state(context)
            .middleware(DefaultHeaders::new().header("Content-Type", "text/html"))
            .resource("/", |r| r.get().with(views::index))
            //            .resource("/api/v1/channels", |r| {
            //                r.method(http::Method::GET).with(v1::channel_list)
            //            })
            //            .resource("/api/v1/channels/{id}", |r| {
            //                r.method(http::Method::GET).with(v1::get_channel)
            //            })
            //            .resource("/api/v1/channels/{id}/channel_versions", |r| {
            //                r.method(http::Method::GET)
            //                    .with(v1::get_channel_channel_versions)
            //            })
            //            .resource("/api/v1/channels/{id}/channel_versions/latest", |r| {
            //                r.method(http::Method::GET)
            //                    .with(v1::get_latest_channel_report)
            //            })
            //            .resource("/api/v1/channel_versions/{report_id}", |r| {
            //                r.method(http::Method::GET).with(v1::get_report)
            //            })
            //            .resource("/api/v1/channel_versions/{report_id}/issues", |r| {
            //                r.method(http::Method::GET).with(v1::get_report_issues)
            //            })
            //            .resource("/api/v1/channel_versions/{report_id}/packages", |r| {
            //                r.method(http::Method::GET).with(v1::get_report_packages)
            //            })
            //            .resource("/api/v1/issues", |r| {
            //                r.method(http::Method::GET).with(v1::get_issues)
            //            })
            //            .resource("/api/v1/issues/{issue_identifier}", |r| {
            //                r.method(http::Method::GET).with(v1::get_issue)
            //            })
            // API version 2 from here on down
            .resource("/api/v2/channels", |r| {
                r.method(http::Method::GET).with(v2::channel_list)
            })
            .resource("/api/v2/channels/{channel}/latest", |r| {
                r.method(http::Method::GET).with(v2::channel_detail_latest)
            })
            .resource("/api/v2/issues", |r| {
                r.method(http::Method::GET).with(v2::issue_list)
            })
            .resource("/api/v2/packages", |r| {
                r.method(http::Method::GET).with(v2::package_list)
            })
            .resource("/api/v2/packages/{package_name}", |r| {
                r.method(http::Method::GET).with(v2::package_detail)
            })
            .resource("/api/v2/commits", |r| {
                r.method(http::Method::GET).with(v2::commit_list)
            })
            .resource("/channels/{channel}", |r| {
                r.method(http::Method::GET)
                    .with(views::latest_issues_in_channel)
            })
            .resource("/issues/{issue}", |r| {
                r.method(http::Method::GET).with(views::show_issue)
            })
            .resource("/_static/jquery-3.3.1.slim.min.js", |r| {
                r.method(http::Method::GET).with(static_jquery)
            })
            .resource("/_static/bootstrap-4.3.1.bundle.min.js", |r| {
                r.method(http::Method::GET).with(static_bootstrap)
            })
            .resource("/_static/bootstrap-4.3.1.min.css", |r| {
                r.method(http::Method::GET).with(static_bootstrap_css)
            })
            .resource("/_static/logo.png", |r| r.get().with(static_logo))
            .resource("/search", |r| r.get().with(views::show_search_results))
    })
    .bind(listen_address)
    .unwrap()
    .run();
}
