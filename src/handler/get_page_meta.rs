use super::*;

pub struct GetPageMeta;

#[derive(Debug, Deserialize)]
pub struct GetPageMetaReq;

#[derive(Debug, Serialize, FromRow)]
pub struct PageMeta {
    pub id: i64,
    pub paper_name: String,
    pub page_no: String,
}


impl ExecSql<GetPageMetaReq> for GetPageMeta {
    async fn handle_get(
        cfg: Extension<Arc<Config>>,
        _prms: Option<Query<GetPageMetaReq>>,
    ) -> Result<Json<Value>, WebErr> {
        let mut conn = SqliteConnection::connect(&cfg.db_path).await?;
        let sql = r#"
            select * from page_meta
        "#;
        let rows = sqlx::query_as::<Sqlite, PageMeta>(sql)
            .fetch_all(&mut conn)
            .await?;
        Ok(Json(json!({
            "success": true,
            "errMsg": "页面元数据查询成功",
            "data": rows,
        })))
    }
}