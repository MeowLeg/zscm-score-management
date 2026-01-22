use super::*;

pub struct BatchUpdateTvUrls;

#[derive(Debug, Deserialize)]
pub struct BatchUpdateTvUrlsReq {
    pub tv_urls: Vec<(i32, String)>,
}

impl ExecSql<BatchUpdateTvUrlsReq> for BatchUpdateTvUrls {
    async fn handle_post(
        cfg: Extension<Arc<Config>>,
        prms: Result<Json<BatchUpdateTvUrlsReq>, JsonRejection>,
    ) -> Result<Json<Value>, WebErr> {
        let Json(prms) = prms?;
        let mut conn = SqliteConnection::connect(&cfg.db_path).await?;
        for (id, tv_url) in prms.tv_urls {
            sqlx::query(
                "update article set tv_url = ? where id = ?",
            )
            .bind(tv_url)
            .bind(id)
            .execute(&mut conn)
            .await?;
        }
        Ok(Json(json!({
            "success": true,
            "errMsg": "success",
            "data": Value::Null
        })))
    }
}