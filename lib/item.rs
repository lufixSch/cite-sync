use mime::Mime;

pub trait Bibliography {
    fn from_bib() -> Self;
}

#[derive(Default)]
pub enum ItemType {
    Article,

    #[default]
    Misc,
}


pub struct Author {
    pub first_name: String,
    pub last_name: String,
}

pub enum FileType {
    Document,
    Snapshot
}


pub struct File {
    pub id: String,
    pub mime_type: Mime,
    pub kind: FileType,
}

impl File {
    pub fn get_url(&self, item_id: String) -> String {
        format!("/content/{item_id}/{}", self.id)
    }
}

#[derive(Default)]
pub struct ResearchItem {
    pub authors: Vec<Author>,
    pub id: String,
    pub title: String,
    pub summary: Option<String>,
    pub publisher: Option<String>,
    pub kind: ItemType,

    pub files: Vec<File>
}
