use super::*;

pub struct GetReporterInfoByName;

#[derive(Debug, Deserialize)]
pub struct GetReporterInfoByNameReq {
    pub name: String,
}


#[derive(Debug, Serialize, FromRow)]
pub struct ReporterInfo {
    pub id: u32,
    pub name: String,
    pub phone: String,
    pub ref_code: String,
    pub reporter_category_id: u32,
    pub department: String,
    pub state: i32,
}


impl ExecSql<GetReporterInfoByNameReq> for GetReporterInfoByName {
    async fn handle_get(
        cfg: Extension<Arc<Config>>,
        prms: Option<Query<GetReporterInfoByNameReq>>,
    ) -> Result<Json<Value>, WebErr> {
        let Query(prms) = prms.ok_or("Missing query parameters")?;
        let mut conn = SqliteConnection::connect(&cfg.db_path).await?;
        let sql = "SELECT * FROM reporter WHERE name = ? and state = 1";
        let row = sqlx::query_as::<Sqlite, ReporterInfo>(sql)
            .bind(prms.name.as_str())
            .fetch_one(&mut conn)
            .await?;
        Ok(Json(json!({
            "success": true,
            "errMsg": "查询成功",
            "data": row
        })))
    }
}