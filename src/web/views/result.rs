use actix_web::dev::AsyncResult;
use actix_web::{HttpRequest, HttpResponse, Responder};

// Custom Result type that allows us to implement at Responder that uses `into` for the types
// The default implementation doesn't allow different types that can all be converted into the same. :(
pub enum HttpResult<C, E> {
    Ok(C),
    Err(E),
}

impl<C, E> Responder for HttpResult<C, E>
where
    C: Responder,
    E: Responder,
{
    type Item = AsyncResult<HttpResponse>;
    type Error = actix_web::Error;

    fn respond_to<S: 'static>(
        self,
        req: &HttpRequest<S>,
    ) -> actix_web::Result<AsyncResult<HttpResponse>, actix_web::Error> {
        // unwrap each of the fields and convert the inner values
        // it is a bit sad that there doesn't seem to be a nicer way to write this.
        match self {
            HttpResult::Ok(v) => match v.respond_to(req) {
                Ok(o) => Ok(o.into()),
                Err(e) => Err(e.into()),
            },
            HttpResult::Err(v) => match v.respond_to(req) {
                Ok(o) => Ok(o.into()),
                Err(e) => Err(e.into()),
            },
        }
    }
}
