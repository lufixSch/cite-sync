use std::path::Path;

use poem::web::Data;
use poem_openapi::{ApiResponse, OpenApi, param, payload::Attachment};

use crate::{state::CiteSyncPaths, tags::CategoryTags};

#[derive(Debug, ApiResponse)]
enum DownloadFileResponse {
    #[oai(status = 200)]
    Ok(
        Attachment<Vec<u8>>,
        #[oai(header = "Content-Disposition")] String
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
    #[oai(path = "/:dir/:name", method = "get")]
    async fn download(
        &self,
        param::Path(dir): param::Path<String>,
        param::Path(name): param::Path<String>,
        Data(paths): Data<&CiteSyncPaths>,
    ) -> DownloadFileResponse {
        let file_path = Path::new(&paths.data_dir)
            .join("storage")
            .join(&dir)
            .join(&name);

        dbg!(file_path.clone());

        if !file_path.exists() {
            return DownloadFileResponse::NotFound;
        };

        match std::fs::read(file_path) {
            Ok(data) => DownloadFileResponse::Ok(Attachment::new(data), format!("attachment; filename=\"{}\"", name)),
            Err(_) => DownloadFileResponse::FileReadFailed,
        }
    }
}
