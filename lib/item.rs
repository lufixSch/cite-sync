use biblatex::{self, ChunksExt, EntryType};
use eyre::{Result, eyre};
use poem_openapi::{Enum, Object};
use serde_json::Value;
use std::{fmt::Display, str::FromStr};

use mime::Mime;

use crate::errors::EnumConversionError;

/// Trait for creating instances from bibliography data.
pub trait Bibliography {
    fn from_bib(bibliography: biblatex::Bibliography) -> Vec<Self>
    where
        Self: std::marker::Sized;

    fn from_json(bibliography: Value) -> Option<Vec<Self>>
    where
        Self: std::marker::Sized;
}

/// Represents the type of a research item.
#[derive(Default, Enum)]
pub enum ItemType {
    /// An article, such as a journal paper.
    Article,

    /// Miscellaneous items that don't fit into other categories.
    #[default]
    Misc,
}

impl Display for ItemType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ItemType::Article => "article",
                ItemType::Misc => "misc",
            }
        )
    }
}

impl FromStr for ItemType {
    type Err = EnumConversionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        dbg!(s);

        if s == "article" {
            Ok(ItemType::Article)
        } else if s == "misc" {
            Ok(ItemType::Misc)
        } else {
            Err(EnumConversionError)
        }
    }
}

/// Represents an author of a research item.
#[derive(Object)]
pub struct Author {
    /// The first name of the author.
    pub first_name: String,
    /// The last name of the author.
    pub last_name: String,
}

/// Represents the type of a file associated with a research item.
#[derive(Enum)]
pub enum FileType {
    /// A document, such as a PDF or Word file.
    Document,
    /// A snapshot of a webpage in a format such as an image or html file.
    Snapshot,
}

impl Display for FileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}",
            match self {
                FileType::Document => "document",
                FileType::Snapshot => "snapshot",
            }
        )
    }
}

impl FromStr for FileType {
    type Err = EnumConversionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "document" {
            Ok(FileType::Document)
        } else if s == "snapshot" {
            Ok(FileType::Snapshot)
        } else {
            Err(EnumConversionError)
        }
    }
}

/// Represents a file associated with a research item.
pub struct File {
    /// The unique identifier for the file.
    pub id: String,
    /// The MIME type of the file.
    pub mime_type: Mime,
    /// The kind of the file (e.g., document, snapshot).
    pub kind: FileType,
}

impl File {
    /// Generates a URL for accessing the file.
    ///
    /// # Arguments
    ///
    /// * `item_id` - A string slice that holds the unique identifier of the research item to which the file belongs.
    ///
    /// # Returns
    ///
    /// A `String` representing the URL to access the file.
    pub fn get_url(&self, item_id: String) -> String {
        format!("/content/{item_id}/{}", self.id)
    }
}

/// Represents a research item, such as an article or paper.
#[derive(Default, Object)]
pub struct ResearchItem {
    /// The unique identifier for the research item (usually the key in the bibtex format)
    pub id: String,

    /// A list of authors of the research item.
    pub authors: Vec<Author>,
    /// The title of the research item.
    pub title: String,
    /// An optional summary or abstract of the research item.
    pub summary: Option<String>,
    /// An optional publisher of the research item.
    pub publisher: Option<String>,
    /// The type of the research item (e.g., article, miscellaneous).
    pub kind: String, // TODO: ItemKind

    /// A list of files associated with the research item.
    #[oai(skip = true)]
    pub files: Vec<File>,
}

impl Bibliography for ResearchItem {
    fn from_bib(bibliography: biblatex::Bibliography) -> Vec<Self>
    where
        Self: std::marker::Sized,
    {
        bibliography
            .iter()
            .flat_map(|b| {
                Ok::<ResearchItem, eyre::Report>(ResearchItem {
                    id: b.key.clone(),
                    title: b
                        .title()
                        .map_err(|_| eyre!("Bibliography Item has no Title"))?
                        .parse()
                        .map_err(|_| eyre!("Unable to parse Title"))?,
                    authors: b
                        .author()
                        .unwrap_or_default()
                        .into_iter()
                        .map(|a| Author {
                            first_name: a.given_name,
                            last_name: a.name,
                        })
                        .collect(),
                    publisher: None,
                    summary: Some(String::from("")), // TODO: b.abstract_()?.parse()?,
                    kind: b.entry_type.to_string(),
                    files: vec![],
                })
            })
            .collect::<Vec<ResearchItem>>()
    }

    fn from_json(bibliography: Value) -> Option<Vec<Self>>
    where
        Self: std::marker::Sized,
    {
        Some(
            bibliography
                .get("items")?
                .as_array()?
                .iter()
                .flat_map(|b| {
                    Some::<ResearchItem>(ResearchItem {
                        id: b["citationKey"].as_str()?.to_string(),
                        title: b["title"].as_str()?.to_string(),
                        authors: b["creators"].as_array()?.iter().flat_map(|c| {
                            if c["creatorType"].as_str()? == "author" {
                                Some(Author {
                                    first_name: c["firstName"].as_str()?.to_string(),
                                    last_name: c["lastName"].as_str()?.to_string()
                                })
                            } else {
                                None
                            }
                        }).collect(),
                        publisher: b["publisher"].as_str().map(|p| p.to_string()),
                        summary: b["abstractNote"].as_str().map(|p| p.to_string()),
                        kind: {
                            let item_type = b["itemType"].as_str()?;

                            // Add space before each uppercase letter
                            let mut result = String::new();
                            for (i, c) in item_type.char_indices() {
                                if i > 0 && c.is_uppercase() {
                                    result.push(' ');
                                }
                                result.push(c);
                            }

                            // Set first letter to uppercase
                            let mut chars = result.chars();
                            match chars.next() {
                                None => String::new(),
                                Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
                            }
                        },
                        files: vec![],
                    })
                })
                .collect::<Vec<ResearchItem>>(),
        )
    }
}
