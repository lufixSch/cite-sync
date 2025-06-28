use atom_syndication::Feed;
use poem::{Error, Result, http::StatusCode};
use poem_openapi::{OpenApi, payload::PlainText};

use super::{
    catalog::{self, AcquisitionCatalog, CatalogLocations, NavigationCatalog, OpdsCatalog},
    entry::NavigationEntry,
};

use crate::tags::CategoryTags;

/// Root router for handling OPDS API requests.
pub struct Router;

#[OpenApi(prefix_path = "opds", tag = "CategoryTags::Opds")]
impl Router {
    /// Endpoint to retrieve the root navigation catalog in OPDS format.
    ///
    /// # Returns
    ///
    /// A `PlainText` response containing the serialized OPDS feed.
    #[oai(path = "/", method = "get")]
    async fn index(&self) -> Result<PlainText<String>> {
        let feed: Feed = NavigationCatalog::build(
            "0".into(),
            "CiteSync".into(),
            CatalogLocations {
                current: "/opds".into(),
                parent: None,
            },
            vec![
                NavigationEntry {
                    id: "1".into(),
                    title: "Last Added".into(),
                    description: "Newly added publications".into(),
                    location: "/opds/sort/new".into(),
                },
                NavigationEntry {
                    id: "2".into(),
                    title: "Alphabetical".into(),
                    description: "Sorted alphabetically".into(),
                    location: "/opds/sort/alphabetical".into(),
                },
                NavigationEntry {
                    id: "3".into(),
                    title: "Authors".into(),
                    description: "Grouped by Authors".into(),
                    location: "/opds/filter/authors".into(),
                },
            ],
        );

        match catalog::serialize(feed) {
            Ok(feed_str) => Ok(PlainText(feed_str)),
            Err(_) => Err(Error::from_string(
                "Unable to serialize feed!",
                StatusCode::INTERNAL_SERVER_ERROR,
            )),
        }
    }

    /// Endpoint to retrieve the catalog of newly added publications in OPDS format.
    ///
    /// # Returns
    ///
    /// A `PlainText` response containing the serialized OPDS feed.
    #[oai(path = "/sort/new", method = "get")]
    async fn catalog(&self) -> Result<PlainText<String>> {
        let feed: Feed = AcquisitionCatalog::build(
            "1".into(),
            "Last Added".into(),
            CatalogLocations {
                current: "/opds/sort/new".into(),
                parent: Some("/opds".into()),
            },
            vec![],
        );

        match catalog::serialize(feed) {
            Ok(feed_str) => Ok(PlainText(feed_str)),
            Err(_) => Err(Error::from_string(
                "Unable to serialize feed!",
                StatusCode::INTERNAL_SERVER_ERROR,
            )),
        }
    }
}
