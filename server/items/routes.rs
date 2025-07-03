use poem::web::Data;
use poem_openapi::{ApiResponse, OpenApi, param::Path, payload::Json};
use sqlx::SqlitePool;

use super::database;
use crate::tags::CategoryTags;

use libcitesync::item::ResearchItem;

pub struct Router;

#[derive(ApiResponse)]
enum ReadAllResponse {
    #[oai(status = 200)]
    Ok(Json<Vec<String>>),

    #[oai(status = 500)]
    InternalError,
}

#[derive(ApiResponse)]
enum ReadResponse {
    #[oai(status = 200)]
    Ok(Json<ResearchItem>),

    #[oai(status = 404)]
    NotFound,
}

#[derive(ApiResponse)]
enum CreateBulkResponse {
    #[oai(status = 200)]
    Ok(Json<Vec<String>>),

    #[oai(status = 500)]
    InternalError,
}

#[OpenApi(prefix_path = "items", tag = "CategoryTags::CiteSync")]
impl Router {
    #[oai(path = "/", method = "get")]
    async fn read_all(&self, db: Data<&SqlitePool>) -> ReadAllResponse {
        let result = sqlx::query!("SELECT id FROM items",).fetch_all(db.0).await;

        match result {
            Ok(items) => ReadAllResponse::Ok(Json(items.into_iter().map(|item| item.id).collect())),
            Err(_) => ReadAllResponse::InternalError,
        }
    }

    #[oai(path = "/", method = "post")]
    async fn create_bulk(
        &self,
        db: Data<&SqlitePool>,
        items: Json<Vec<ResearchItem>>,
    ) -> CreateBulkResponse {
        let mut ids: Vec<String> = vec![];

        for item in items.0 {
            ids.push(item.id.clone());

            if database::create(db.0, item).await.is_err() {
                return CreateBulkResponse::InternalError;
            }
        }

        CreateBulkResponse::Ok(Json(ids))
    }

    #[oai(path = "/:id", method = "get")]
    async fn read(&self, db: Data<&SqlitePool>, id: Path<String>) -> ReadResponse {
        match database::read(db.0, id.0).await {
            Ok(item) => ReadResponse::Ok(Json(item)),
            Err(_) => ReadResponse::NotFound,
        }
    }
}
