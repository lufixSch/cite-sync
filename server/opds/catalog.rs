use atom_syndication::{Entry, Feed, Link};
use eyre::{Result, eyre};
use libcitesync::item::ResearchItem;

use super::entry::{NavigationEntry, OpdsEntry};

pub fn build_mime_type(kind: &str) -> Option<String> {
    Some(format!(
        "application/atom+xml;profile=opds-catalog;kind={}",
        kind
    ))
}

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

pub fn serialize(feed: Feed) -> Result<String> {
    let bytes = feed.write_to(Vec::new())?;
    String::from_utf8(bytes).map_err(|err| eyre!("Failed to serialize Opds Feed: {}", err))
}

pub struct CatalogLocations {
    pub current: String,
    pub parent: Option<String>,
}

pub trait OpdsCatalog<E: OpdsEntry> {
    fn build(id: String, title: String, location: CatalogLocations, entries: Vec<E>) -> Feed;

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

pub struct NavigationCatalog;

impl OpdsCatalog<NavigationEntry> for NavigationCatalog {
    fn build(
        id: String,
        title: String,
        location: CatalogLocations,
        entries: Vec<NavigationEntry>,
    ) -> Feed {
        NavigationCatalog::_build(id, title, location, entries, "navigation")
    }
}

pub struct AcquisitionCatalog;

impl OpdsCatalog<ResearchItem> for AcquisitionCatalog {
    fn build(
        id: String,
        title: String,
        location: CatalogLocations,
        entries: Vec<ResearchItem>,
    ) -> Feed {
        AcquisitionCatalog::_build(id, title, location, entries, "navigation")
    }
}
