use clap::crate_version;
use poem_openapi::{Object, OpenApi, payload::Json};

#[derive(Object)]
struct WelcomeMsg {
    msg: String,
    version: String,
}

pub struct Router;

#[OpenApi]
impl Router {
    #[oai(path = "/", method = "get")]
    async fn index(&self) -> Json<WelcomeMsg> {
        Json(WelcomeMsg {
            msg: String::from("Welcome to the CiteSync API!"),
            version: String::from(crate_version!()),
        })
    }
}
