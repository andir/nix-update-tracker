use super::mime::Mime;

// Dirty way to parse the accept header
#[derive(Debug)]
pub struct Accept {
    media: Vec<Mime>,
}

impl Accept {
    pub fn from_header(value: &actix_web::http::header::HeaderValue) -> Option<Self> {
        match value.to_str() {
            Ok(s) => s
                .split(",")
                .map(|s| s.trim().parse())
                .collect::<Result<Vec<_>, _>>()
                .map(|m| Some(Self { media: m }))
                .unwrap_or(None),
            Err(e) => {
                error!("failed to parse header value: {}", e);
                None
            }
        }
    }

    pub fn contains(&self, m: &Mime) -> bool {
        self.media.iter().any(|entry| {
            entry == m || (m.type_() == entry.type_() && m.subtype() == entry.subtype())
        })
    }
}
