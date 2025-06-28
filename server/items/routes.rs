use poem::web::Data;
use poem_openapi::{ApiResponse, OpenApi, payload::Json};
use sqlx::SqlitePool;
use uuid::Uuid;

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
        let result = sqlx::query!("SELECT title FROM items",)
            .fetch_all(db.0)
            .await;

        match result {
            Ok(items) => {
                ReadAllResponse::Ok(Json(items.into_iter().map(|item| item.title).collect()))
            }
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

            if let Err(e) = database::create(db.0, item).await {
                dbg!(e);
                return CreateBulkResponse::InternalError;
            }
        }

        CreateBulkResponse::Ok(Json(ids))
    }
}
