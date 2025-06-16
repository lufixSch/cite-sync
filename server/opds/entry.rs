use atom_syndication::{Content, Entry, Link, Person};
use libcitesync::item;

use super::catalog::build_mime_type;

/// A trait for converting entries to OPDS format.
///
/// # Associated Types
///
/// * `E` - The type of entry that can be converted to OPDS format.
pub trait OpdsEntry {
    /// Converts an entry to an OPDS `Entry`.
    ///
    /// # Returns
    ///
    /// An `Entry` object representing the OPDS entry.
    fn to_opds(&self) -> Entry;
}

impl OpdsEntry for item::ResearchItem {
    /// Converts a `ResearchItem` to an OPDS `Entry`.
    ///
    /// # Returns
    ///
    /// An `Entry` object representing the OPDS entry.
    fn to_opds(&self) -> Entry {
        Entry {
            title: self.title.clone().into(),
            id: self.id.clone(),
            summary: self.summary.clone().map(|c| c.into()),
            authors: self
                .authors
                .iter()
                .map(|author| Person {
                    name: format!("{} {}", author.first_name, author.last_name),
                    ..Default::default()
                })
                .collect(),
            links: self
                .files
                .iter()
                .map(|file| Link {
                    rel: "http://opds-spec.org/acquisition".into(),
                    href: file.get_url(self.id.clone()),
                    mime_type: Some(file.mime_type.essence_str().into()),
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        }
    }
}

/// Represents a navigation entry in an OPDS catalog.
pub struct NavigationEntry {
    /// A unique identifier for the entry.
    pub id: String,
    /// The title of the entry.
    pub title: String,
    /// A description of the entry.
    pub description: String,
    /// The location URL of the entry.
    pub location: String,
}

impl OpdsEntry for NavigationEntry {
    /// Converts a `NavigationEntry` to an OPDS `Entry`.
    ///
    /// # Returns
    ///
    /// An `Entry` object representing the OPDS entry.
    fn to_opds(&self) -> Entry {
        Entry {
            title: self.title.clone().into(),
            id: self.id.clone(),
            content: Some(Content {
                content_type: Some("text".into()),
                value: Some(self.description.clone()),
                ..Default::default()
            }),
            links: vec![Link {
                href: self.location.clone(),
                mime_type: build_mime_type("acquisition"),
                ..Default::default()
            }],
            ..Default::default()
        }
    }
}
