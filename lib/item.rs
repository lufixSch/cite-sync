use mime::Mime;

/// Trait for creating instances from bibliography data.
pub trait Bibliography {
    fn from_bib() -> Self;
}

/// Represents the type of a research item.
#[derive(Default)]
pub enum ItemType {
    /// An article, such as a journal paper.
    Article,

    /// Miscellaneous items that don't fit into other categories.
    #[default]
    Misc,
}

/// Represents an author of a research item.
pub struct Author {
    /// The first name of the author.
    pub first_name: String,
    /// The last name of the author.
    pub last_name: String,
}

/// Represents the type of a file associated with a research item.
pub enum FileType {
    /// A document, such as a PDF or Word file.
    Document,
    /// A snapshot of a webpage in a format such as an image or html file.
    Snapshot
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
#[derive(Default)]
pub struct ResearchItem {
    /// A list of authors of the research item.
    pub authors: Vec<Author>,
    /// The unique identifier for the research item.
    pub id: String,
    /// The title of the research item.
    pub title: String,
    /// An optional summary or abstract of the research item.
    pub summary: Option<String>,
    /// An optional publisher of the research item.
    pub publisher: Option<String>,
    /// The type of the research item (e.g., article, miscellaneous).
    pub kind: ItemType,

    /// A list of files associated with the research item.
    pub files: Vec<File>
}
