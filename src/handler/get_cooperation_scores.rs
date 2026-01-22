use super::*;

pub struct GetCooperationScores;

#[derive(Debug, Deserialize)]
pub struct GetCooperationScoresReq {
    pub reporter_id: u32,
    pub year: u32,
    pub month: u32,
}

#[derive(Debug, Serialize, FromRow)]
pub struct CooperationScore {
    pub id: u32,
    pub reporter_id: u32,
    pub content: String,
    pub score: i32,
    pub score_from: String,
    pub publish_year: u32,
    pub publish_month: u32,
    pub state: i32,
}

impl ExecSql<GetCooperationScoresReq> for GetCooperationScores {
    async fn handle_get(
        cfg: Extension<Arc<Config>>,
        prms: Option<Query<GetCooperationScoresReq>>,
    ) -> Result<Json<Value>, WebErr> {
        let Query(prms) = prms.ok_or("Missing parameters")?;
        let mut conn = SqliteConnection::connect(&cfg.db_path).await?;
        let sql = "SELECT * FROM cooperation_score WHERE reporter_id = ? AND publish_year = ? AND publish_month = ? and state = 1";
        let rows = sqlx::query_as::<Sqlite, CooperationScore>(sql)
            .bind(prms.reporter_id)
            .bind(prms.year)
            .bind(prms.month)
            .fetch_all(&mut conn)
            .await
            .unwrap_or(vec![]);
        Ok(Json(json!({
            "success": true,
            "errMsg": "查询成功",
            "data": rows
        })))
    }
}