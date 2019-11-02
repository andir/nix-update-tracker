use actix_web::{HttpRequest, HttpResponse};
use serde::Serialize;

// Trait that signifies that a type is able to produce an HTML response
pub trait HtmlResponse {
    fn to_html_response<S>(
        self,
        req: &HttpRequest<S>,
    ) -> actix_web::Result<HttpResponse, actix_web::Error>;
}

// Trait that signifies that a type is able to produce a JSON response
pub trait JsonResponse {
    fn to_json_response<S>(
        &self,
        req: &HttpRequest<S>,
    ) -> actix_web::Result<HttpResponse, actix_web::Error>;
}

impl<C> JsonResponse for C
where
    C: Serialize,
{
    fn to_json_response<S>(
        &self,
        _req: &HttpRequest<S>,
    ) -> actix_web::Result<HttpResponse, actix_web::Error> {
        let body = serde_json::to_string(&self)?;
        Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(&body))
    }
}
