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
    async fn handle_get_with_headers(
        headers: http::HeaderMap,
        cfg: Extension<Arc<Config>>,
        _prms: Option<Query<GetProgramsReq>>,
    ) -> Result<Json<Value>, WebErr> {
        let account = match headers.get("account") {
            Some(ant) => ant.to_str()?,
            None => ""
        };
        if account.is_empty() {
            return Err("no account".into());
        }
        println!("account: {}", account);
        let mut conn = SqliteConnection::connect(&cfg.db_path).await?;
        let sql = format!(
            "select * from program where state = 1 and site_id in (select site_id from admin_sites where admin_id in (select id from admin where name = '{}'))", account
        );
        println!("sql: {}", &sql);
        let rs = sqlx::query_as::<Sqlite, GetProgramsResp>(&sql)
            .fetch_all(&mut conn)
            .await?;
        Ok(Json(json!({
            "success": true,
            "errMsg": "",
            "data": rs,
        })))
    }
}
