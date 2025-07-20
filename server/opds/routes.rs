use std::{path::Path, str::FromStr};

use atom_syndication::Feed;
use eyre::eyre;
use libcitesync::{
    bib,
    item::{Bibliography, ItemType, ResearchItem},
};
use poem::{Error, Result, http::StatusCode, web::Data};
use poem_openapi::{
    OpenApi,
    payload::{PlainText, Response},
};
use sqlx::SqlitePool;

use super::{
    catalog::{self, AcquisitionCatalog, CatalogLocations, NavigationCatalog, OpdsCatalog},
    entry::NavigationEntry,
};

use crate::{state::CiteSyncPaths, tags::CategoryTags};

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
    async fn index(&self) -> Result<Response<PlainText<String>>> {
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
                    title: "Alphabetical".into(),
                    description: "Sorted alphabetically".into(),
                    location: "/opds/sort/name".into(),
                },
                NavigationEntry {
                    id: "2".into(),
                    title: "Authors".into(),
                    description: "Grouped by authors".into(),
                    location: "/opds/filter/authors".into(),
                },
                NavigationEntry {
                    id: "3".into(),
                    title: "Publisher".into(),
                    description: "Grouped by publishers".into(),
                    location: "/opds/sort/publisher".into(),
                },
            ],
        );

        match catalog::serialize(feed) {
            Ok(feed_str) => {
                Ok(Response::new(PlainText(feed_str)).header("Content-type", "text/xml"))
            }
            Err(_) => Err(Error::from_string(
                "Unable to serialize feed!",
                StatusCode::INTERNAL_SERVER_ERROR,
            )),
        }
    }

    /// Endpoint to retrieve the catalog of publications sorted by name.
    ///
    /// # Returns
    ///
    /// A `PlainText` response containing the serialized OPDS feed.
    #[oai(path = "/sort/name", method = "get")]
    // async fn catalog(&self, db: Data<&SqlitePool>) -> Result<Response<PlainText<String>>> {
    //     let result = match sqlx::query!("SELECT id, title, summary, publisher, kind FROM items",)
    //         .fetch_all(db.0)
    //         .await
    //     {
    //         Ok(results) => Ok(results),
    //         Err(_) => Err(Error::from_string(
    //             "Unable to fetch entries!",
    //             StatusCode::INTERNAL_SERVER_ERROR,
    //         )),
    //     }?;
    //
    //     let entries = result
    //         .into_iter()
    //         .map(|e| {
    //             Ok::<ResearchItem, Error>(ResearchItem {
    //                 id: e.id,
    //                 authors: vec![],
    //                 title: e.title,
    //                 summary: e.summary,
    //                 publisher: e.publisher,
    //                 kind: ItemType::from_str(e.kind.as_str()).map_err(|_| {
    //                     Error::from_string(
    //                         "Failed parsing data from database!",
    //                         StatusCode::INTERNAL_SERVER_ERROR,
    //                     )
    //                 })?,
    //                 files: vec![],
    //             })
    //         })
    //         .collect::<Result<Vec<_>>>()?;
    //
    //     let feed: Feed = AcquisitionCatalog::build(
    //         "1".into(),
    //         "Alphabetically".into(),
    //         CatalogLocations {
    //             current: "/opds/sort/name".into(),
    //             parent: Some("/opds".into()),
    //         },
    //         entries,
    //     );
    //
    //     match catalog::serialize(feed) {
    //         Ok(feed_str) => {
    //             Ok(Response::new(PlainText(feed_str)).header("Content-type", "text/xml"))
    //         }
    //         Err(_) => Err(Error::from_string(
    //             "Unable to serialize feed!",
    //             StatusCode::INTERNAL_SERVER_ERROR,
    //         )),
    //     }
    // }
    async fn catalog_by_name(&self, paths: Data<&CiteSyncPaths>) -> Result<Response<PlainText<String>>> {
        let path = Path::new(&paths.0.bib_path);
        let bibliography = bib::load_json(path).map_err(|e| {
            Error::from_string(
                format!("Unable to load bibliography: {e}"),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

        let entries = Bibliography::from_json(bibliography).ok_or(Error::from_string("Unable to deserialize bibliography!", StatusCode::INTERNAL_SERVER_ERROR))?;

        let feed: Feed = AcquisitionCatalog::build(
            "1".into(),
            "Alphabetically".into(),
            CatalogLocations {
                current: "/opds/sort/name".into(),
                parent: Some("/opds".into()),
            },
            entries,
        );

        match catalog::serialize(feed) {
            Ok(feed_str) => {
                Ok(Response::new(PlainText(feed_str)).header("Content-type", "text/xml"))
            }
            Err(_) => Err(Error::from_string(
                "Unable to serialize feed!",
                StatusCode::INTERNAL_SERVER_ERROR,
            )),
        }
    }
}
