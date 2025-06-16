use clap::crate_version;
use poem_openapi::{Object, OpenApi, payload::Json};

use super::tags::CategoryTags;

/// Represents a welcome message response.
#[derive(Object)]
struct WelcomeMsg {
    /// The welcome message.
    msg: String,
    /// The version of the CiteSync server.
    version: String,
}

/// Root router for handling API requests.
pub struct Router;

#[OpenApi(tag = "CategoryTags::CiteSync")]
impl Router {
    /// Endpoint to retrieve the welcome message and server version.
    ///
    /// # Returns
    ///
    /// A JSON response containing the welcome message and server version.
    #[oai(path = "/", method = "get")]
    async fn index(&self) -> Json<WelcomeMsg> {
        Json(WelcomeMsg {
            msg: String::from("Welcome to the CiteSync API!"),
            version: String::from(crate_version!()),
        })
    }
}
