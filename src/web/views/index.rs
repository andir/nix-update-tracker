use actix_web::{HttpResponse, Responder, State};
use diesel::SqliteConnection;
use horrorshow::prelude::*;

use super::queries;
use super::Result;
use super::{HttpResult, Page};
use crate::web::AppState;

#[derive(Serialize)]
struct Index {
    statistics: Vec<(String, queries::ChannelIssueStatistics)>,
}

impl RenderOnce for Index {
    fn render_once(self, tmpl: &mut TemplateBuffer) {
        use super::table::{AsTable, Column};

        tmpl << html! {
            |t| {
                type Row = (String, queries::ChannelIssueStatistics);
                self.statistics.as_table(t, &[
                    Column::<Row>::new("channel", &|(k, _v)| { Box::new(k.to_string()) }),
                    Column::new("patched", &|(_k, v)| Box::new(v.patched)),
                    Column::new("open", &|(_k, v)| Box::new(v.unpatched)),
                ])
            }
        };
    }
}

impl Page<Index> {
    fn new(conn: &SqliteConnection) -> Result<Self> {
        let channels = queries::get_channels(&conn)?;
        let statistics = channels
            .iter()
            .map(|channel| {
                let stats = queries::issue_statistics_for_channel(&channel, &conn)?;
                Ok((channel.name.clone(), stats))
            })
            .collect::<diesel::QueryResult<Vec<(_, _)>>>()?;

        Page::new_with(
            "Nix Vulnerability Scanner".to_string(),
            Index { statistics },
            conn,
        )
    }
}

pub fn index(state: State<AppState>) -> impl Responder {
    let p: Page<Index> = match Page::new(&state.connection) {
        Ok(p) => p,
        Err(e) => {
            error!("failed to generate index page instance: {}", e);
            let r = HttpResponse::InternalServerError();
            return HttpResult::Err(r);
        }
    };

    return HttpResult::Ok(p);
}
