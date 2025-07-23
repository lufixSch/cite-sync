use biblatex::{self, ChunksExt};
use eyre::{Result, eyre};
use poem_openapi::{Enum, Object};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde_json::Value;
use std::{collections::HashMap, fmt::Display, str::FromStr};

use mime::Mime;

use crate::errors::EnumConversionError;

/// Trait for creating instances from bibliography data.
pub trait Bibliography {
    fn from_bib(bibliography: biblatex::Bibliography, collection: &str) -> Vec<Self>
    where
        Self: std::marker::Sized;

    fn from_json(bibliography: &Value, collection: &str) -> Option<HashMap<u64, Self>>
    where
        Self: std::marker::Sized;

    fn from_json_collections(bibliographies: &HashMap<String, Value>) -> HashMap<u64, Self>
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
#[derive(Object, Clone, Debug)]
pub struct Author {
    /// The first name of the author.
    pub first_name: String,
    /// The last name of the author.
    pub last_name: String,
}

impl Author {
    /// Returns id of the author generated from the author name
    pub fn get_id(&self) -> String {
        format!(
            "{}_{}",
            self.last_name.to_lowercase(),
            self.first_name.to_lowercase()
        )
    }

    /// Returns author name as string
    pub fn get_name(&self) -> String {
        format!("{}, {}", self.last_name, self.first_name)
    }
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
#[derive(Clone, Debug)]
pub struct File {
    /// The unique identifier/path for the file.
    pub id: String,
    /// The MIME type of the file.
    pub mime_type: Mime,
    // The kind of the file (e.g., document, snapshot).
    // TODO: pub kind: FileType,
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
    pub fn get_url(&self, collection: &str) -> String {
        format!("/content/{}/{}", collection, self.id)
    }
}

/// Represents a research item, such as an article or paper.
#[derive(Default, Object, Clone, Debug)]
pub struct ResearchItem {
    /// The unique identifier for the research item (usually the key in the bibtex format)
    pub id: String,
    /// Name of the base collection
    pub collection: String,

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
    /// List of tags for this item
    pub tags: Vec<String>,

    /// A list of files associated with the research item.
    #[oai(skip = true)]
    pub files: Vec<File>,
}

impl Bibliography for ResearchItem {
    fn from_bib(bibliography: biblatex::Bibliography, collection: &str) -> Vec<Self>
    where
        Self: std::marker::Sized,
    {
        bibliography
            .iter()
            .flat_map(|b| {
                Ok::<ResearchItem, eyre::Report>(ResearchItem {
                    id: b.key.clone(),
                    collection: collection.into(),
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
                    tags: vec![],
                })
            })
            .collect::<Vec<ResearchItem>>()
    }

    fn from_json(bibliography: &Value, collection: &str) -> Option<HashMap<u64, Self>>
    where
        Self: std::marker::Sized,
    {
        Some(
            bibliography
                .get("items")?
                .as_array()?
                .par_iter()
                .flat_map(|b| {
                    Some::<(u64, ResearchItem)>((
                        b.get("itemID")?.as_u64()?,
                        ResearchItem {
                            id: b["citationKey"].as_str()?.to_string(),
                            collection: collection.into(),
                            title: b["title"].as_str()?.to_string(),
                            authors: b["creators"]
                                .as_array()?
                                .iter()
                                .flat_map(|c| {
                                    if c["creatorType"].as_str()? == "author" {
                                        Some(Author {
                                            first_name: c["firstName"].as_str()?.to_string(),
                                            last_name: c["lastName"].as_str()?.to_string(),
                                        })
                                    } else {
                                        None
                                    }
                                })
                                .collect(),
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
                                    Some(f) => {
                                        f.to_uppercase().collect::<String>() + chars.as_str()
                                    }
                                }
                            },
                            files: b["attachments"]
                                .as_array()?
                                .iter()
                                .flat_map(|f| {
                                    let abs_path = f["path"].as_str()?.to_string();
                                    let idx = abs_path.find("storage/")?;

                                    let path =
                                        abs_path.split_at(idx + "storage/".len()).1.to_string();
                                    let mime_guess = mime_guess::from_path(&path);

                                    Some(File {
                                        id: path,
                                        mime_type: mime_guess.first_or_octet_stream(),
                                    })
                                })
                                .collect::<Vec<File>>(),
                            tags: b["tags"]
                                .as_array()?
                                .iter()
                                .flat_map(|t| Some(t["tag"].as_str()?.to_string()))
                                .collect(),
                        },
                    ))
                })
                .collect::<HashMap<u64, ResearchItem>>(),
        )
    }


    /// NOTE: unparsable collections will be skipped
    fn from_json_collections(bibliographies: &HashMap<String, Value>) -> HashMap<u64, Self>
    where
        Self: std::marker::Sized,
    {
        bibliographies
            .par_iter()
            .flat_map(|(collection, bibliography)| Self::from_json(bibliography, collection))
            .flatten()
            .collect::<HashMap<u64, ResearchItem>>()
    }
}
