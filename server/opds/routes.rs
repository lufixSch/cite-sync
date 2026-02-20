use atom_syndication::Feed;
use libcitesync::item::ResearchItem;
use poem::{Error, Result, http::StatusCode, web::Data};
use poem_openapi::{
    OpenApi, param,
    payload::{PlainText, Response},
};
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use serde_json::Value;

use super::{
    catalog::{self, AcquisitionCatalog, CatalogLocations, NavigationCatalog, OpdsCatalog},
    entry::NavigationEntry,
};

use crate::{state::AppState, tags::CategoryTags};

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
            "base".into(),
            "CiteSync".into(),
            CatalogLocations {
                current: "/opds".into(),
                parent: None,
            },
            vec![
                NavigationEntry {
                    id: "alphabetical".into(),
                    title: "Alphabetical".into(),
                    description: Some("Sorted alphabetically".into()),
                    location: "/opds/sort/name".into(),
                },
                NavigationEntry {
                    id: "authors".into(),
                    title: "Authors".into(),
                    description: Some("Grouped by authors".into()),
                    location: "/opds/filter/authors".into(),
                },
                NavigationEntry {
                    id: "collections".into(),
                    title: "Collections".into(),
                    description: Some("Grouped by collections".into()),
                    location: "/opds/filter/collections".into(),
                },
                NavigationEntry {
                    id: "tags".into(),
                    title: "Tags".into(),
                    description: Some("Grouped by Tags".into()),
                    location: "/opds/filter/tags".into(),
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
    async fn catalog_by_name(
        &self,
        Data(state): Data<&AppState>,
    ) -> Result<Response<PlainText<String>>> {
        let bib_guard = state.get_bibliography().await.map_err(|e| {
            Error::from_string(
                format!("Unable to load bibliography: {e}"),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

        let bib = bib_guard.read().await;

        let mut entries: Vec<ResearchItem> = bib.research_items.values().cloned().collect();

        entries.sort_by(|a: &ResearchItem, b| a.title.cmp(&b.title));

        let feed: Feed = AcquisitionCatalog::build(
            "alphabetical".into(),
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

    /// Endpoint to retrieve the catalog of all authors sorted by name.
    ///
    /// # Returns
    ///
    /// A `PlainText` response containing the serialized OPDS feed.
    #[oai(path = "/filter/authors", method = "get")]
    async fn catalog_filter_authors(
        &self,
        Data(state): Data<&AppState>,
    ) -> Result<Response<PlainText<String>>> {
        let bib_guard = state.get_bibliography().await.map_err(|e| {
            Error::from_string(
                format!("Unable to load bibliography: {e}"),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

        let bib = bib_guard.read().await;

        let mut entries: Vec<(String, String)> = bib
            .authors
            .iter()
            .map(|(author_id, (author_name, _))| (author_id.clone(), author_name.clone()))
            .collect::<Vec<(String, String)>>();

        entries.sort_by(|(_, a), (_, b)| a.cmp(b));
        entries.dedup();

        let feed: Feed = NavigationCatalog::build(
            "authors".into(),
            "Authors".into(),
            CatalogLocations {
                current: "/opds/filter/authors".into(),
                parent: Some("/opds".into()),
            },
            entries
                .iter()
                .enumerate()
                .map(|(i, (id, a))| NavigationEntry {
                    id: i.to_string(),
                    title: a.into(),
                    description: None,
                    location: format!("/opds/filter/authors/{}", id),
                })
                .collect(),
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

    /// Endpoint to retrieve the publications of one author sorted by name.
    ///
    /// # Returns
    ///
    /// A `PlainText` response containing the serialized OPDS feed.
    #[oai(path = "/filter/authors/:id", method = "get")]
    async fn catalog_filter_author(
        &self,
        Data(state): Data<&AppState>,
        param::Path(id): param::Path<String>,
    ) -> Result<Response<PlainText<String>>> {
        let bib_guard = state.get_bibliography().await.map_err(|e| {
            Error::from_string(
                format!("Unable to load bibliography: {e}"),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

        let bib = bib_guard.read().await;

        let (author_name, item_ids) = bib.authors.get(&id).ok_or(Error::from_string(
            "Author not found!",
            StatusCode::NOT_FOUND,
        ))?;

        let mut entries: Vec<ResearchItem> = item_ids
            .par_iter()
            .filter_map(|item_id| bib.research_items.get(item_id).cloned())
            .collect();

        entries.sort_by(|a: &ResearchItem, b| a.title.cmp(&b.title));

        let feed: Feed = AcquisitionCatalog::build(
            format!("authors/{id}"),
            author_name.clone(),
            CatalogLocations {
                current: format!("/opds/filter/authors/{id}"),
                parent: Some("/opds/filter/authors".into()),
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

    /// Endpoint to retrieve catalog of all base collections.
    ///
    /// # Returns
    ///
    /// A `PlainText` response containing the serialized OPDS feed.
    #[oai(path = "/filter/collections", method = "get")]
    async fn catalog_filter_collections(
        &self,
        Data(state): Data<&AppState>,
    ) -> Result<Response<PlainText<String>>> {
        let bib_guard = state.get_bibliography().await.map_err(|e| {
            Error::from_string(
                format!("Unable to load bibliography: {e}"),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

        let bib = bib_guard.read().await;

        let mut entries = bib
            .collections_metadata
            .keys()
            .map(|c| NavigationEntry {
                id: c.clone(),
                title: c.clone(),
                description: None,
                location: format!("/opds/filter/collections/{c}"),
            })
            .collect::<Vec<NavigationEntry>>();

        entries.sort_by(|a, b| a.title.cmp(&b.title));

        let feed: Feed = NavigationCatalog::build(
            "collections".into(),
            "Collections".into(),
            CatalogLocations {
                current: "/opds/filter/collections".into(),
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

    /// Endpoint to retrieve catalog of one base collection.
    ///
    /// # Returns
    ///
    /// A `PlainText` response containing the serialized OPDS feed.
    #[oai(path = "/filter/collections/:base", method = "get")]
    async fn catalog_filter_collection_base(
        &self,
        Data(state): Data<&AppState>,
        param::Path(base): param::Path<String>,
    ) -> Result<Response<PlainText<String>>> {
        let bib_guard = state.get_bibliography().await.map_err(|e| {
            Error::from_string(
                format!("Unable to load bibliography: {e}"),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

        let bib = bib_guard.read().await;

        let metadata = bib
            .collections_metadata
            .get(&base)
            .ok_or(Error::from_string(
                "Base collection not found!",
                StatusCode::NOT_FOUND,
            ))?;

        let collections = metadata
            .collections_structure
            .as_ref()
            .and_then(|c| c.as_object())
            .ok_or(Error::from_string(
                "Unable to deserialize collections!",
                StatusCode::INTERNAL_SERVER_ERROR,
            ))?;

        let mut entries = collections
            .into_iter()
            .flat_map(|(id, c)| {
                if c["parent"] == "" {
                    Some(NavigationEntry {
                        id: id.clone(),
                        title: c["name"].as_str()?.into(),
                        description: None,
                        location: format!("/opds/filter/collections/{}/{}", base, id),
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<NavigationEntry>>();

        entries.sort_by(|a, b| a.title.cmp(&b.title));

        let feed: Feed = NavigationCatalog::build(
            "collections".into(),
            "Collections".into(),
            CatalogLocations {
                current: format!("/opds/filter/collections/{base}"),
                parent: Some("/opds/filter/collections".into()),
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

    /// Endpoint to retrieve catalog of all collections (subcollections or content).
    ///
    /// # Returns
    ///
    /// A `PlainText` response containing the serialized OPDS feed.
    #[oai(path = "/filter/collections/:base/:id", method = "get")]
    async fn catalog_filter_collection(
        &self,
        Data(state): Data<&AppState>,
        param::Path(base): param::Path<String>,
        param::Path(id): param::Path<String>,
    ) -> Result<Response<PlainText<String>>> {
        let bib_guard = state.get_bibliography().await.map_err(|e| {
            Error::from_string(
                format!("Unable to load bibliography: {e}"),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

        let bib = bib_guard.read().await;

        let metadata = bib
            .collections_metadata
            .get(&base)
            .ok_or(Error::from_string(
                "Base collection not found!",
                StatusCode::NOT_FOUND,
            ))?;

        let collections = metadata
            .collections_structure
            .as_ref()
            .and_then(|c| c.as_object())
            .ok_or(Error::from_string(
                "Unable to deserialize collections!",
                StatusCode::INTERNAL_SERVER_ERROR,
            ))?;

        let collection = collections.get(&id).ok_or(Error::from_string(
            "Collection not found!",
            StatusCode::NOT_FOUND,
        ))?;

        let empty_subcollections = vec![];
        let subcollections = collection
            .get("collections")
            .and_then(|c| c.as_array())
            .unwrap_or(&empty_subcollections);

        let name: String = collection["name"]
            .as_str()
            .ok_or(Error::from_string(
                "Unable to deserialize collections!",
                StatusCode::INTERNAL_SERVER_ERROR,
            ))?
            .into();

        let feed = if !subcollections.is_empty() {
            NavigationCatalog::build(
                format!("collections/{id}"),
                name,
                CatalogLocations {
                    current: format!("/opds/filter/collections/{base}/{id}"),
                    parent: Some(format!("/opds/filter/collections/{base}")),
                },
                subcollections
                    .par_iter()
                    .flat_map(|e: &Value| e.as_str())
                    .flat_map(|e| {
                        let subcollection = collections.get(e)?;

                        Some(NavigationEntry {
                            id: e.into(),
                            title: subcollection["name"].as_str()?.into(),
                            description: None,
                            location: format!("/opds/filter/collections/{}/{}", base, e),
                        })
                    })
                    .collect(),
            )
        } else {
            let items = collection
                .get("items")
                .and_then(|c| c.as_array())
                .unwrap_or(&empty_subcollections)
                .par_iter()
                .filter_map(|id: &Value| id.as_u64())
                .collect::<Vec<u64>>();

            let entries: Vec<ResearchItem> = items
                .par_iter()
                .filter_map(|item_id| bib.research_items.get(item_id).cloned())
                .collect();

            AcquisitionCatalog::build(
                format!("collections/{id}"),
                name,
                CatalogLocations {
                    current: format!("/opds/filter/collections/{id}"),
                    parent: Some("/opds/filter/collections".into()),
                },
                entries,
            )
        };

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

    /// Endpoint to retrieve catalog of all tags.
    ///
    /// # Returns
    ///
    /// A `PlainText` response containing the serialized OPDS feed.
    #[oai(path = "/filter/tags", method = "get")]
    async fn catalog_filter_tags(
        &self,
        Data(state): Data<&AppState>,
    ) -> Result<Response<PlainText<String>>> {
        let bib_guard = state.get_bibliography().await.map_err(|e| {
            Error::from_string(
                format!("Unable to load bibliography: {e}"),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

        let bib = bib_guard.read().await;

        let mut entries: Vec<String> = bib.tags.keys().cloned().collect();

        entries.sort();

        let feed: Feed = NavigationCatalog::build(
            "tags".into(),
            "Tags".into(),
            CatalogLocations {
                current: "/opds/filter/tags".into(),
                parent: Some("/opds".into()),
            },
            entries
                .into_par_iter()
                .map(|t| NavigationEntry {
                    id: t.clone(),
                    title: t.clone(),
                    description: None,
                    location: format!("/opds/filter/tags/{}", t),
                })
                .collect(),
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

    /// Endpoint to retrieve catalog of a specific tag
    ///
    /// # Returns
    ///
    /// A `PlainText` response containing the serialized OPDS feed.
    #[oai(path = "/filter/tags/:id", method = "get")]
    async fn catalog_filter_tag(
        &self,
        Data(state): Data<&AppState>,
        param::Path(id): param::Path<String>,
    ) -> Result<Response<PlainText<String>>> {
        let bib_guard = state.get_bibliography().await.map_err(|e| {
            Error::from_string(
                format!("Unable to load bibliography: {e}"),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

        let bib = bib_guard.read().await;

        let item_ids = bib.tags.get(&id).ok_or(Error::from_string(
            "Tag not found!",
            StatusCode::NOT_FOUND,
        ))?;

        let mut entries: Vec<ResearchItem> = item_ids
            .par_iter()
            .filter_map(|item_id| bib.research_items.get(item_id).cloned())
            .collect();

        entries.sort_by(|a: &ResearchItem, b| a.title.cmp(&b.title));

        let feed: Feed = AcquisitionCatalog::build(
            format!("tags/{id}"),
            id.clone(),
            CatalogLocations {
                current: format!("/opds/filter/tags/{id}"),
                parent: Some("/opds/filter/tags".into()),
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
