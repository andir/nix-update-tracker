use actix_web::{HttpResponse, Path, Responder, State};
use diesel::prelude::*;
use itertools::Itertools;

use super::AppState;

use super::super::process_db::{models, schema};

pub fn channel_list(state: State<AppState>) -> impl Responder {
    match models::Channel::all(&state.connection) {
        Ok(channels) => HttpResponse::Ok().json(channels),
        Err(e) => {
            error!("Failed to retrieve channels: {}", e);
            HttpResponse::NotFound().finish()
        }
    }
}
