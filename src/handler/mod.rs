use super::*;
// use reqwest::Client;
use serde_json::{Value, json};
use sqlx::{Connection, FromRow, Sqlite, SqliteConnection};
use std::error::Error;

pub mod article_from_dump;
pub mod calc_monthly;
pub mod event;
pub mod get_event_scores;
pub mod cooperation;
pub mod get_cooperation_scores;
pub mod get_articles;
pub mod get_page_meta;
pub mod get_reporter_events;
pub mod get_reporter_categories;
pub mod get_reporters;
pub mod reporter;
pub mod score;
pub mod monthly_add_score;
pub mod get_monthly_add_score;
pub mod monthly_sub_score;
pub mod get_monthly_sub_score;
pub mod login;
pub mod get_programs;
pub mod article;
pub mod batch_update_tv_urls;
pub mod get_reporter_info_by_name;
pub mod get_paper_abnormal_articles;

#[allow(dead_code)]
pub trait ExecSql<T> {
    async fn handle_post(
        _cfg: Extension<Arc<Config>>,
        _prms: Result<Json<T>, JsonRejection>,
    ) -> Result<Json<Value>, WebErr> {
        Ok(Json(json!({})))
    }

    async fn handle_post_with_redis_cli(
        _cfg_info: Extension<Arc<Config>>,
        _redis_cli: Extension<Arc<Client>>,
        _params: Result<Json<T>, JsonRejection>
    ) -> Result<Json<Value>, WebErr> {
        Ok(Json(json!({})))
    }


    async fn handle_get(
        _cfg: Extension<Arc<Config>>,
        _prms: Option<Query<T>>,
    ) -> Result<Json<Value>, WebErr> {
        Ok(Json(json!({})))
    }

    async fn handle_get_with_headers(
        _headers: http::HeaderMap,
        _cfg_info: Extension<Arc<Config>>,
        _params: Option<Query<T>>
    ) -> Result<Json<Value>, WebErr> {
        Ok(Json(json!({})))
    }

    async fn handle_get_with_headers_and_redis_cli(
        _headers: http::HeaderMap,
        _cfg_info: Extension<Arc<Config>>,
        _redis_cli: Extension<Arc<Client>>,
        _params: Option<Query<T>>
    ) -> Result<Json<Value>, WebErr> {
        Ok(Json(json!({})))
    }

}

#[derive(Debug)]
pub struct WebErr(Box<dyn Error + Send + Sync>);

impl IntoResponse for WebErr {
    fn into_response(self) -> Response {
        let j = json!({
            "status": 0,
            "message": format!("{}", self.0),
            "data": "",
        })
        .to_string();
        Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(j.into())
            .unwrap()
    }
}

impl<E> From<E> for WebErr
where
    E: Into<Box<dyn Error + Send + Sync>>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
