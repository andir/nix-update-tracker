use actix_web::{HttpResponse, Path, Responder, State};
use diesel::prelude::*;

use super::AppState;

use super::super::process_db::{models, schema};
use super::queries;

pub fn channel_list(state: State<AppState>) -> impl Responder {
    match models::Channel::all(&state.connection) {
        Ok(channels) => HttpResponse::Ok().json(channels),
        Err(e) => {
            error!("Failed to retrieve channels: {}", e);
            HttpResponse::NotFound().finish()
        }
    }
}

pub fn issue_list(state: State<AppState>) -> impl Responder {
    match models::Issue::all(&state.connection) {
        Ok(issues) => HttpResponse::Ok().json(issues),
        Err(e) => {
            error!("Failed to retrieve issue list: {}", e);
            HttpResponse::NotFound().finish()
        }
    }
}

pub fn package_list(state: State<AppState>) -> impl Responder {
    match models::Package::all(&state.connection) {
        Ok(issues) => HttpResponse::Ok().json(issues),
        Err(e) => {
            error!("Failed to retrieve package list: {}", e);
            HttpResponse::NotFound().finish()
        }
    }
}

pub fn package_detail((state, info): (State<AppState>, Path<(String,)>)) -> impl Responder {
    let pkgname = &info.0;

    let pkg = match schema::packages::table
        .filter(schema::packages::dsl::attribute_name.eq(&pkgname))
        .first::<models::Package>(&state.connection)
    {
        Ok(pkg) => pkg,
        // Err(diesel::result::Error::NotFound) => {
        Err(e) => {
            error!("Failed to find pkg with attribute name {}: {}", pkgname, e);
            return HttpResponse::NotFound().finish();
        }
    };

    #[derive(Serialize)]
    struct PackageDetail {
        package: models::Package,
        versions: Vec<models::PackageVersion>,
    }

    match models::PackageVersion::belonging_to(&pkg).load(&state.connection) {
        Ok(pkg_versions) => HttpResponse::Ok().json(PackageDetail {
            package: pkg,
            versions: pkg_versions,
        }),
        Err(e) => {
            error!(
                "Failed to retrieve list of package versions for pkg attribute {}: {}",
                pkgname, e
            );
            HttpResponse::NotFound().finish()
        }
    }
}

pub fn commit_list(state: State<AppState>) -> impl Responder {
    match models::Commit::all(&state.connection) {
        Ok(commits) => HttpResponse::Ok().json(commits),
        Err(e) => {
            error!("Failed to retrieve list of commits: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub fn channel_detail_latest((state, info): (State<AppState>, Path<(String,)>)) -> impl Responder {
    let channel_name = &info.0;
    let channel = match models::Channel::get(&channel_name, &state.connection) {
        Ok(c) => c,
        Err(diesel::result::Error::NotFound) => {
            error!("Failed to find channel {}", channel_name);
            return HttpResponse::NotFound().finish();
        }
        Err(e) => {
            error!("Failed to find channel {}: {}", channel_name, e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    // retrieve bump on channel
    let bump: models::ChannelBump = match schema::channel_bumps::table
        .filter(schema::channel_bumps::dsl::channel_id.eq(channel.id))
        .order(schema::channel_bumps::dsl::channel_bump_date.desc())
        .first(&state.connection)
    {
        Ok(bump) => bump,
        Err(e) => {
            error!("Failed to retrieve latest channel bump for version: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    // retrieve latest commit on channel
    let commit: models::Commit = match schema::commits::table
        .filter(schema::commits::dsl::id.eq(bump.id))
        .first(&state.connection)
    {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to retrieve commit {}: {}", bump.commit_id, e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    info!("Retrieving pkg_versions for: {}", commit.revision);

    let pkg_versions = match queries::issues_in_commit(&commit, &state.connection) {
        Ok(o) => o,
        Err(e) => {
            error!(
                "Failed to retrieve issues for commit {}: {}",
                commit.revision, e
            );
            return HttpResponse::InternalServerError().finish();
        }
    };

    HttpResponse::Ok().json(&pkg_versions)
}
