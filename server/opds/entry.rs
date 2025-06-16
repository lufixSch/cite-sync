use atom_syndication::{Content, Entry, Link, Person};
use libcitesync::item;

use super::catalog::build_mime_type;

pub trait OpdsEntry {
    fn to_opds(&self) -> Entry;
}

impl OpdsEntry for item::ResearchItem {
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


pub struct NavigationEntry {
    pub id: String,
    pub title: String,
    pub description: String,
    pub location: String
}

impl OpdsEntry for NavigationEntry {
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

