use std::str::FromStr;

use eyre::{Result, eyre};
use libcitesync::item::{Author, File, FileType, ItemType, ResearchItem};
use sqlx::SqlitePool;

pub async fn create(db: &SqlitePool, item: ResearchItem) -> Result<(), eyre::Error> {
    let item_kind = match item.kind {
        ItemType::Article => "Article",
        ItemType::Misc => "Misc",
    };

    // Insert into items table and get the last inserted row id
    sqlx::query!(
        "INSERT INTO items (id, title, summary, publisher, kind) VALUES (?, ?, ?, ?, ?)",
        item.id,
        item.title,
        item.summary,
        item.publisher,
        item_kind
    )
    .execute(db)
    .await?;

    // Insert authors into authors table and establish relation with items
    for author in &item.authors {
        // Check if publisher already exists
        let existing_author = sqlx::query!(
            "SELECT id FROM authors WHERE first_name = ? AND last_name = ?",
            author.first_name,
            author.last_name
        )
        .fetch_optional(db)
        .await?;

        let author_id = match existing_author {
            Some(row) => row.id,
            None => sqlx::query!(
                "INSERT INTO authors (first_name, last_name) VALUES (?, ?)",
                author.first_name,
                author.last_name
            )
            .execute(db)
            .await?
            .last_insert_rowid(),
        };

        // Insert into the join table
        sqlx::query!(
            "INSERT INTO author_item (author_id, item_id) VALUES (?, ?)",
            author_id,
            item.id
        )
        .execute(db)
        .await?;
    }

    // Insert files into files table and establish relation with items
    for file in &item.files {
        let mime_type = file.mime_type.to_string();
        let file_kind = file.kind.to_string();
        sqlx::query!(
            "INSERT INTO files (id, mime_type, kind, item_id) VALUES (?, ?, ?, ?)",
            file.id,
            mime_type,
            file_kind,
            item.id
        )
        .execute(db)
        .await?;
    }

    Ok(())
}

pub async fn read(db: &SqlitePool, id: String) -> Result<ResearchItem, eyre::Error> {
    let item_row = sqlx::query!(
        "SELECT id, title, summary, publisher, kind FROM items WHERE id = ?",
        id
    )
    .fetch_one(db)
    .await?;

    let authors: Vec<_> = sqlx::query_as!(
        Author,
        "SELECT first_name, last_name FROM author_item JOIN authors ON author_item.author_id = authors.id WHERE item_id = ?",
        item_row.id
    )
    .fetch_all(db)
    .await?;

    let files: Vec<_> = sqlx::query!(
        "SELECT id, mime_type, kind FROM files WHERE item_id = ?",
        item_row.id
    )
    .fetch_all(db)
    .await?;

    let item_kind = ItemType::from_str(item_row.kind.as_str())?;

    let files: Vec<File> = files
        .into_iter()
        .map(|file| {
            Ok::<File, eyre::Error>(File {
                id: file.id.ok_or(eyre!("Encountered File without ID!"))?, // Should never be NONE
                mime_type: file.mime_type.parse()?,
                kind: FileType::from_str(file.kind.as_str())?,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let item = ResearchItem {
        id: item_row.id.ok_or(eyre!("Encountered Item without ID!"))?, // Should never be NONE
        title: item_row.title,
        summary: item_row.summary,
        publisher: item_row.publisher,
        kind: item_kind,
        authors,
        files,
    };

    Ok(item)
}
