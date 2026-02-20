use std::{fs, path::Path};

use poem::web::Data;
use poem_openapi::{ApiResponse, OpenApi, param, payload::Attachment};

use crate::{state::AppState, tags::CategoryTags};

#[derive(Debug, ApiResponse)]
enum DownloadFileResponse {
    #[oai(status = 200)]
    Ok(
        Attachment<Vec<u8>>,
        #[oai(header = "Content-Disposition")] String,
    ),

    /// File not found
    #[oai(status = 404)]
    NotFound,

    /// IO Error
    #[oai(status = 500)]
    FileReadFailed,
}

pub struct Router;

#[OpenApi(prefix_path = "content", tag = "CategoryTags::Content")]
impl Router {
    /// Endpoint to download content
    ///
    /// # Returns
    ///
    /// The downloadable file
    #[oai(path = "/:collection/:dir/:name", method = "get")]
    async fn download(
        &self,
        param::Path(collection): param::Path<String>,
        param::Path(dir): param::Path<String>,
        param::Path(name): param::Path<String>,
        Data(state): Data<&AppState>,
    ) -> DownloadFileResponse {
        let bib_guard = match state.get_bibliography().await {
            Ok(bib) => bib,
            Err(_) => return DownloadFileResponse::FileReadFailed,
        };

        let bib = bib_guard.read().await;

        let collection_metadata = match bib.collections_metadata.get(&collection) {
            Some(metadata) => metadata,
            None => return DownloadFileResponse::NotFound,
        };

        let file_path = Path::new(&collection_metadata.storage_path)
            .join(&dir)
            .join(&name);

        if !file_path.exists() {
            return DownloadFileResponse::NotFound;
        };

        match fs::read(file_path) {
            Ok(data) => DownloadFileResponse::Ok(
                Attachment::new(data),
                format!("attachment; filename=\"{}\"", name),
            ),
            Err(_) => DownloadFileResponse::FileReadFailed,
        }
    }
}
