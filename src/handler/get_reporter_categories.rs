use super::*;


pub struct GetReporterCategories;

#[derive(Debug, Deserialize)]
pub struct GetReporterCategoriesReq;

#[derive(Debug, Serialize, FromRow)]
struct ReporterCategory {
    pub id: u32,
    pub name: String,
}


impl ExecSql<GetReporterCategoriesReq> for GetReporterCategories {
    async fn handle_get(
        cfg: Extension<Arc<Config>>,
        _prms: Option<Query<GetReporterCategoriesReq>>
    ) -> Result<Json<Value>, WebErr> {
        let mut conn = SqliteConnection::connect(&cfg.db_path).await?;
        let sql_sel = r#"
            select id, name from reporter_category
        "#;
        let rs = sqlx::query_as::<Sqlite, ReporterCategory>(sql_sel)
            .fetch_all(&mut conn)
            .await?;
        Ok(Json(json!({
            "success": true,
            "errMsg": "获取记者分类成功",
            "data": rs,
        })))
    }
}