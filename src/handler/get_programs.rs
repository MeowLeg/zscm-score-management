use super::*;

pub struct GetPrograms;

#[derive(Debug, Deserialize)]
pub struct GetProgramsReq;


#[derive(Debug, Serialize, FromRow)]
pub struct GetProgramsResp {
    id: u32,
    name: String,
    media_type: u8,
    site_id: u32,
    code: String,
    state: u8,
}

impl ExecSql<GetProgramsReq> for GetPrograms {
    async fn handle_get(
        cfg: Extension<Arc<Config>>,
        _prms: Option<Query<GetProgramsReq>>,
    ) -> Result<Json<Value>, WebErr> {
        let mut conn = SqliteConnection::connect(&cfg.db_path).await?;
        let sql = "select * from program";
        let rs = sqlx::query_as::<Sqlite, GetProgramsResp>(sql)
            .fetch_all(&mut conn)
            .await?;
        Ok(Json(json!({
            "success": true,
            "errMsg": "",
            "data": rs,
        })))
    }
}
