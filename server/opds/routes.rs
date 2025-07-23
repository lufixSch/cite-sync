use std::{path::Path, str::FromStr};

use atom_syndication::Feed;
use eyre::eyre;
use libcitesync::{
    bib,
    item::{Bibliography, ItemType, ResearchItem},
};
use poem::{Error, Result, http::StatusCode, web::Data};
use poem_openapi::{
    OpenApi, param,
    payload::{PlainText, Response},
};
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
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
        Data(paths): Data<&CiteSyncPaths>,
    ) -> Result<Response<PlainText<String>>> {
        let path = Path::new(&paths.data_dir);
        let bibliographies = bib::load_dir(path, &paths.bib_name).map_err(|e| {
            Error::from_string(
                format!("Unable to load bibliography: {e}"),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

        let mut entries: Vec<ResearchItem> = ResearchItem::from_json_collections(&bibliographies)
            .into_values()
            .collect();

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
        Data(paths): Data<&CiteSyncPaths>,
    ) -> Result<Response<PlainText<String>>> {
        let path = Path::new(&paths.data_dir);
        let bibliographies = bib::load_dir(path, &paths.bib_name).map_err(|e| {
            Error::from_string(
                format!("Unable to load bibliography: {e}"),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

        let mut entries: Vec<(String, String)> =
            ResearchItem::from_json_collections(&bibliographies)
                .par_iter()
                .flat_map(|(_, e)| &e.authors)
                .map(|a| (a.get_id(), a.get_name()))
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
        Data(paths): Data<&CiteSyncPaths>,
        param::Path(id): param::Path<String>,
    ) -> Result<Response<PlainText<String>>> {
        let path = Path::new(&paths.data_dir);
        let bibliographies = bib::load_dir(path, &paths.bib_name).map_err(|e| {
            Error::from_string(
                format!("Unable to load bibliography: {e}"),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

        let mut entries: Vec<ResearchItem> = ResearchItem::from_json_collections(&bibliographies)
            .into_par_iter()
            .flat_map(|(_, e)| {
                if e.authors
                    .iter()
                    .map(|a| a.get_id())
                    .collect::<Vec<String>>()
                    .contains(&id)
                {
                    Some(e)
                } else {
                    None
                }
            })
            .collect::<Vec<ResearchItem>>();

        entries.sort_by(|a: &ResearchItem, b| a.title.cmp(&b.title));

        let feed: Feed = AcquisitionCatalog::build(
            format!("authors/{id}"),
            "Author".into(),
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
        Data(paths): Data<&CiteSyncPaths>,
    ) -> Result<Response<PlainText<String>>> {
        let path = Path::new(&paths.data_dir);
        let bibliographies = bib::load_dir(path, &paths.bib_name).map_err(|e| {
            Error::from_string(
                format!("Unable to load bibliography: {e}"),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

        let mut entries = bibliographies
            .into_keys()
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
        Data(paths): Data<&CiteSyncPaths>,
        param::Path(base): param::Path<String>,
    ) -> Result<Response<PlainText<String>>> {
        let path = Path::new(&paths.data_dir);
        let bibliographies = bib::load_dir(path, &paths.bib_name).map_err(|e| {
            Error::from_string(
                format!("Unable to load bibliography: {e}"),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

        let mut entries = bibliographies
            .get(&base)
            .ok_or(Error::from_string(
                "Base collection not found!",
                StatusCode::NOT_FOUND,
            ))?
            .get("collections")
            .and_then(|c| c.as_object())
            .ok_or(Error::from_string(
                "Unable to deserialize collections!",
                StatusCode::INTERNAL_SERVER_ERROR,
            ))?
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
        Data(paths): Data<&CiteSyncPaths>,
        param::Path(base): param::Path<String>,
        param::Path(id): param::Path<String>,
    ) -> Result<Response<PlainText<String>>> {
        let path = Path::new(&paths.data_dir);
        let bibliographies = bib::load_dir(path, &paths.bib_name).map_err(|e| {
            Error::from_string(
                format!("Unable to load bibliography: {e}"),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

        let bibliography = bibliographies.get(&base).ok_or(Error::from_string(
            "Base collection not found!",
            StatusCode::NOT_FOUND,
        ))?;

        let collections = bibliography
            .get("collections")
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
                    .flat_map(|e| e.as_str())
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
                .flat_map(|id| id.as_u64())
                .collect::<Vec<u64>>();

            let entries: Vec<ResearchItem> = ResearchItem::from_json(&bibliography, &base)
                .ok_or(Error::from_string(
                    "Unable to deserialize bibliography!",
                    StatusCode::INTERNAL_SERVER_ERROR,
                ))?
                .into_par_iter()
                .flat_map(|(item_id, e)| {
                    if items.contains(&item_id) {
                        Some(e)
                    } else {
                        None
                    }
                })
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
        Data(paths): Data<&CiteSyncPaths>,
    ) -> Result<Response<PlainText<String>>> {
        let path = Path::new(&paths.data_dir);
        let bibliographies = bib::load_dir(path, &paths.bib_name).map_err(|e| {
            Error::from_string(
                format!("Unable to load bibliography: {e}"),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

        let mut entries: Vec<String> = ResearchItem::from_json_collections(&bibliographies)
            .into_par_iter()
            .flat_map(|(_, e)| e.tags)
            .collect::<Vec<String>>();

        //entries.sort_by(|(_, a)| );
        entries.sort();
        entries.dedup();

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
        Data(paths): Data<&CiteSyncPaths>,
        param::Path(id): param::Path<String>,
    ) -> Result<Response<PlainText<String>>> {
        let path = Path::new(&paths.data_dir);
        let bibliographies = bib::load_dir(path, &paths.bib_name).map_err(|e| {
            Error::from_string(
                format!("Unable to load bibliography: {e}"),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

        let mut entries: Vec<ResearchItem> = ResearchItem::from_json_collections(&bibliographies)
            .into_par_iter()
            .flat_map(|(_, e)| if e.tags.contains(&id) { Some(e) } else { None })
            .collect::<Vec<ResearchItem>>();

        entries.sort_by(|a: &ResearchItem, b| a.title.cmp(&b.title));

        let feed: Feed = AcquisitionCatalog::build(
            format!("authors/{id}"),
            "Author".into(),
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
