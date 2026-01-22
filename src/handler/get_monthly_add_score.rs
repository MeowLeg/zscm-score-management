use super::*;

pub struct GetMonthlyAddScore;

#[derive(Debug, Deserialize)]
pub struct GetMonthlyAddScoreReq {
    pub reporter_id: u32,
    pub year: u32,
    pub month: u32,
}

#[derive(Debug, Serialize, FromRow, Default)]
pub struct MonthlyAddScore {
    pub id: Option<u32>,
    pub reporter_id: u32,
    pub reason: String,
    pub score: i32,
    pub publish_year: u32,
    pub publish_month: u32,
    pub state: i32,
}

impl ExecSql<GetMonthlyAddScoreReq> for GetMonthlyAddScore {
    async fn handle_get(
        cfg: Extension<Arc<Config>>,
        prms: Option<Query<GetMonthlyAddScoreReq>>,
    ) -> Result<Json<Value>, WebErr> {
        let Query(prms) = prms.ok_or("Missing query parameters")?;
        let mut conn = SqliteConnection::connect(&cfg.db_path).await?;
        let sql = "SELECT * FROM monthly_add_score WHERE reporter_id = ? AND publish_year = ? AND publish_month = ? and satte = 1";
        let r = sqlx::query_as::<Sqlite, MonthlyAddScore>(sql)
            .bind(prms.reporter_id)
            .bind(prms.year)
            .bind(prms.month)
            .fetch_one(&mut conn)
            .await
            .unwrap_or_default();
        Ok(Json(json!({
            "success": true,
            "errMsg": "查询成功",
            "data": r 
        })))
    }
}