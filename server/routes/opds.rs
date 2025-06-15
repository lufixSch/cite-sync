use atom_syndication::{Content, Entry, Feed, Link, Person};
use poem_openapi::{OpenApi, payload::PlainText};

use libcitesync::item;

trait OpdsEntry {
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

fn build_mime_type(kind: &str) -> Option<String> {
    Some(format!(
        "application/atom+xml;profile=opds-catalog;kind={}",
        kind
    ))
}

fn build_feed_links(current: &str, kind: &str) -> Vec<Link> {
    vec![
        Link {
            rel: "self".into(),
            href: current.into(),
            mime_type: build_mime_type("navigation"),
            ..Default::default()
        },
        Link {
            rel: "start".into(),
            href: "/opds".into(),
            mime_type: build_mime_type(kind),
            ..Default::default()
        },
    ]
}

pub struct Router;

#[OpenApi(prefix_path = "opds")]
impl Router {
    #[oai(path = "/", method = "get")]
    async fn index(&self) -> PlainText<String> {
        let mut feed = Feed {
            title: "CiteSync Catalog".into(),
            id: "0".into(),
            links: build_feed_links("/opds", "navigation"),
            ..Default::default()
        };

        feed.set_entries(vec![Entry {
            title: "New".into(),
            id: "0".into(),
            content: Some(Content {
                content_type: Some("text".into()),
                value: Some("New Publications".into()),
                ..Default::default()
            }),
            links: vec![Link {
                href: "/opds/sort/new".into(),
                mime_type: build_mime_type("acquisition"),
                ..Default::default()
            }],
            ..Default::default()
        }]);

        PlainText(
            String::from_utf8(feed.write_to(Vec::new()).unwrap_or_default()).unwrap_or_default(),
        )
    }

    #[oai(path = "/sort/new", method = "get")]
    async fn catalog(&self) -> PlainText<String> {
        let mut feed = Feed {
            title: "New Publications".into(),
            id: "1".into(),
            links: build_feed_links("/opds/sort/new", "acquisition"),
            ..Default::default()
        };

        let content: Vec<item::ResearchItem> = vec![
            item::ResearchItem {
                id: "0".into(),
                title: "Some Paper".into(),
                kind: item::ItemType::Article,
                summary: Some("This is a super interesting Paper".into()),
                authors: vec![item::Author {
                    first_name: "A".into(),
                    last_name: "BC".into(),
                }],
                files: vec![
                    item::File {
                        id: "0".into(),
                        mime_type: mime::APPLICATION_PDF,
                        kind: item::FileType::Document,
                    },
                    item::File {
                        id: "1".into(),
                        mime_type: mime::TEXT_HTML,
                        kind: item::FileType::Snapshot,
                    },
                ],
                ..Default::default()
            },
            item::ResearchItem {
                id: "1".into(),
                title: "Another Paper".into(),
                kind: item::ItemType::Article,
                summary: Some("This is a super duper interesting Paper".into()),
                authors: vec![item::Author {
                    first_name: "D".into(),
                    last_name: "EF".into(),
                }],
                files: vec![
                    item::File {
                        id: "0".into(),
                        mime_type: mime::APPLICATION_PDF,
                        kind: item::FileType::Document,
                    },
                    item::File {
                        id: "1".into(),
                        mime_type: mime::IMAGE_PNG,
                        kind: item::FileType::Snapshot,
                    },
                ],
                ..Default::default()
            },
        ];

        feed.set_entries::<Vec<Entry>>(content.iter().map(|c| c.to_opds()).collect());

        PlainText(
            String::from_utf8(feed.write_to(Vec::new()).unwrap_or_default()).unwrap_or_default(),
        )
    }
}
