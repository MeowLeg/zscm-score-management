use super::*;

pub struct GetReporterEvents;

#[derive(Debug, Deserialize)]
pub struct GetReporterEventsReq {
    pub reporter_id: u32,
    pub year: u32,
    pub month: u32,
    pub day: Option<u32>,
}

#[derive(Debug, Serialize, FromRow)]
struct GetReporterEventsResp {
    id: u32,
    reporter_id: u32,
    content: String,
    score: i32,
    score_from: String,
    publish_year: u32,
    publish_month: u32,
    publish_day: u32,
}

impl ExecSql<GetReporterEventsReq> for GetReporterEvents {
    async fn handle_get(
        cfg: Extension<Arc<Config>>,
        prms: Option<Query<GetReporterEventsReq>>,
    ) -> Result<Json<Value>, WebErr> {
        let prms = prms.ok_or("Missing parameters")?;
        let mut conn = SqliteConnection::connect(&cfg.db_path).await?;
        let sql = format!(
            r#"
            select * from event_score
            where state = 1
                and reporter_id = ?
                and publish_year = ?
                and publish_month = ?
                {}
            "#,
            match prms.day {
                Some(day) => format!("and publish_day = {}", day),
                None => String::new(),
            }
        );
        let events = sqlx::query_as::<Sqlite, GetReporterEventsResp>(&sql)
            .bind(prms.reporter_id)
            .bind(prms.year)
            .bind(prms.month)
            .fetch_all(&mut conn)
            .await?;

        Ok(Json(json!({
            "success": true,
            "errMsg": "",
            "data": events
        })))
    }
}
