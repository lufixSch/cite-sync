use atom_syndication::{Entry, Feed, Link};
use eyre::{Result, eyre};
use libcitesync::item::ResearchItem;

use super::entry::{NavigationEntry, OpdsEntry};

/// Builds the MIME type for an OPDS catalog.
///
/// # Arguments
///
/// * `kind` - A string slice that specifies the kind of the catalog (e.g., "navigation").
///
/// # Returns
///
/// An `Option<String>` containing the MIME type if successful.
pub fn build_mime_type(kind: &str) -> Option<String> {
    Some(format!(
        "application/atom+xml;profile=opds-catalog;kind={}",
        kind
    ))
}

/// Builds a vector of links for an OPDS catalog.
///
/// # Arguments
///
/// * `current` - A string slice that holds the current URL of the catalog.
/// * `parent` - An optional string slice that holds the parent URL of the catalog.
/// * `kind` - A string slice that specifies the kind of the catalog (e.g., "navigation").
///
/// # Returns
///
/// A `Vec<Link>` containing the links for the OPDS catalog.
pub fn build_links(current: &str, parent: Option<&str>, kind: &str) -> Vec<Link> {
    vec![
        Some(Link {
            rel: "self".into(),
            href: current.into(),
            mime_type: build_mime_type("navigation"),
            ..Default::default()
        }),
        Some(Link {
            rel: "start".into(),
            href: "/opds".into(),
            mime_type: build_mime_type(kind),
            ..Default::default()
        }),
        parent.map(|p| Link {
            rel: "up".into(),
            href: p.into(),
            mime_type: build_mime_type(kind),
            ..Default::default()
        }),
    ]
    .into_iter()
    .flatten()
    .collect()
}

/// Serializes an OPDS feed to a string.
///
/// # Arguments
///
/// * `feed` - A reference to the `Feed` object to be serialized.
///
/// # Returns
///
/// A `Result<String>` containing the serialized feed if successful.
pub fn serialize(feed: Feed) -> Result<String> {
    let bytes = feed.write_to(Vec::new())?;
    String::from_utf8(bytes).map_err(|err| eyre!("Failed to serialize Opds Feed: {}", err))
}

/// Represents the locations of an OPDS catalog.
#[derive(Clone)]
pub struct CatalogLocations {
    /// The current URL of the catalog.
    pub current: String,
    /// The optional parent URL of the catalog.
    pub parent: Option<String>,
}

/// A trait for building OPDS catalogs from entries.
///
/// # Associated Types
///
/// * `E` - The type of entry used to build the catalog.
pub trait OpdsCatalog<E: OpdsEntry> {
    /// Builds an OPDS catalog feed from a list of entries.
    ///
    /// # Arguments
    ///
    /// * `id` - A string that uniquely identifies the catalog.
    /// * `title` - A string slice that holds the title of the catalog.
    /// * `location` - An instance of `CatalogLocations` representing the locations of the catalog.
    /// * `entries` - A vector of entries to be included in the catalog.
    ///
    /// # Returns
    ///
    /// A `Feed` object representing the OPDS catalog.
    fn build(id: String, title: String, location: CatalogLocations, entries: Vec<E>) -> Feed;

    /// A helper function for building an OPDS catalog feed from a list of entries.
    ///
    /// This function sets common fields for the feed and adds entries to it.
    ///
    /// # Arguments
    ///
    /// * `id` - A string that uniquely identifies the catalog.
    /// * `title` - A string slice that holds the title of the catalog.
    /// * `location` - An instance of `CatalogLocations` representing the locations of the catalog.
    /// * `entries` - A vector of entries to be included in the catalog.
    /// * `kind` - A string slice that specifies the kind of the catalog (e.g., "navigation").
    ///
    /// # Returns
    ///
    /// A `Feed` object representing the OPDS catalog.
    fn _build(
        id: String,
        title: String,
        location: CatalogLocations,
        entries: Vec<E>,
        kind: &str,
    ) -> Feed {
        let mut feed = Feed {
            title: title.into(),
            id,
            links: build_links(&location.current, location.parent.as_deref(), kind),
            ..Default::default()
        };

        feed.set_entries::<Vec<Entry>>(entries.iter().map(|e| e.to_opds()).collect());

        feed
    }
}

/// A struct representing a navigation catalog.
pub struct NavigationCatalog;

impl OpdsCatalog<NavigationEntry> for NavigationCatalog {
    /// Builds a navigation catalog feed from a list of entries.
    ///
    /// # Arguments
    ///
    /// * `id` - A string that uniquely identifies the catalog.
    /// * `title` - A string slice that holds the title of the catalog.
    /// * `location` - An instance of `CatalogLocations` representing the locations of the catalog.
    /// * `entries` - A vector of entries to be included in the catalog.
    ///
    /// # Returns
    ///
    /// A `Feed` object representing the navigation catalog.
    fn build(
        id: String,
        title: String,
        location: CatalogLocations,
        entries: Vec<NavigationEntry>,
    ) -> Feed {
        NavigationCatalog::_build(id, title, location, entries, "navigation")
    }
}

/// A struct representing an acquisition catalog.
pub struct AcquisitionCatalog;

impl OpdsCatalog<ResearchItem> for AcquisitionCatalog {
    /// Builds an acquisition catalog feed from a list of entries.
    ///
    /// # Arguments
    ///
    /// * `id` - A string that uniquely identifies the catalog.
    /// * `title` - A string slice that holds the title of the catalog.
    /// * `location` - An instance of `CatalogLocations` representing the locations of the catalog.
    /// * `entries` - A vector of entries to be included in the catalog.
    ///
    /// # Returns
    ///
    /// A `Feed` object representing the acquisition catalog.
    fn build(
        id: String,
        title: String,
        location: CatalogLocations,
        entries: Vec<ResearchItem>,
    ) -> Feed {
        AcquisitionCatalog::_build(id, title, location, entries, "navigation")
    }
}
