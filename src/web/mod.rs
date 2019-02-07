use actix_web::{http, server, App, HttpResponse, Responder, State};
use diesel::sqlite::SqliteConnection;
use diesel::Connection;

mod v1;

pub struct AppState {
    connection: SqliteConnection,
}

fn index(_: State<AppState>) -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(include_str!("../../static/index.html"))
}

pub fn serve(listen_address: &str, dbfile: &str) {
    let dbfile = String::from(dbfile);
    server::new(move || {
        let context = AppState {
            connection: SqliteConnection::establish(&dbfile).unwrap(),
        };
        App::with_state(context)
            .resource("/", |r| r.get().with(index))
            .resource("/api/v1/channels", |r| {
                r.method(http::Method::GET).with(v1::channel_list)
            })
            .resource("/api/v1/channels/{id}", |r| {
                r.method(http::Method::GET).with(v1::get_channel)
            })
            .resource("/api/v1/channels/{id}/channel_versions", |r| {
                r.method(http::Method::GET)
                    .with(v1::get_channel_channel_versions)
            })
            .resource("/api/v1/channels/{id}/channel_versions/latest", |r| {
                r.method(http::Method::GET)
                    .with(v1::get_latest_channel_report)
            })
            .resource("/api/v1/channel_versions/{report_id}", |r| {
                r.method(http::Method::GET).with(v1::get_report)
            })
            .resource("/api/v1/channel_versions/{report_id}/issues", |r| {
                r.method(http::Method::GET).with(v1::get_report_issues)
            })
            .resource("/api/v1/channel_versions/{report_id}/packages", |r| {
                r.method(http::Method::GET).with(v1::get_report_packages)
            })
            .resource("/api/v1/issues", |r| {
                r.method(http::Method::GET).with(v1::get_issues)
            })
            .resource("/api/v1/issues/{issue_identifier}", |r| {
                r.method(http::Method::GET).with(v1::get_issue)
            })
    })
    .bind(listen_address)
    .unwrap()
    .run();
}
