use super::*;

pub struct Event;

#[derive(Debug, Deserialize)]
pub struct EventReq {
    pub id: Option<u32>,
    pub reporter_id: u32,
    pub content: String,
    pub score: u32,
    pub score_from: String,
    pub publish_year: u32,
    pub publish_month: u32,
    pub state: u32,
}

impl ExecSql<EventReq> for Event {
    async fn handle_post(
        cfg: Extension<Arc<Config>>,
        prms: Result<Json<EventReq>, JsonRejection>,
    ) -> Result<Json<Value>, WebErr> {
        let Json(prms) = prms?;
        let mut conn = SqliteConnection::connect(&cfg.db_path).await?;
        match prms.id {
            Some(id) => {
                if prms.state == 0 {
                    let sql = "DELETE FROM event_score WHERE id = ?";
                    let r= sqlx::query(sql)
                        .bind(id)
                        .execute(&mut conn)
                        .await?;
                    let rows_affected = r.rows_affected();
                    if rows_affected == 0 {
                        return Err("未找到指定的记录".into());
                    }
                    Ok(Json(json!({
                        "success": true,
                        "errMsg": "删除成功",
                        "data": rows_affected
                    })))
                } else {
                    let sql = "UPDATE event_score SET reporter_id = ?, content = ?, score = ?, score_from = ?, publish_year = ?, publish_month = ?, state = ? WHERE id = ?";
                    let r = sqlx::query(sql)
                        .bind(prms.reporter_id)
                        .bind(prms.content)
                        .bind(prms.score)
                        .bind(prms.score_from)
                        .bind(prms.publish_year)
                        .bind(prms.publish_month)
                        .bind(prms.state)
                        .bind(id)
                        .execute(&mut conn)
                        .await?;
                    let rows_affected = r.rows_affected();
                    if rows_affected == 0 {
                        return Err("未找到指定的记录".into());
                    }
                    Ok(Json(json!({
                        "success": true,
                        "errMsg": "更新成功",
                        "data": rows_affected
                    })))
                }
            }
            None => {
                let sql = "INSERT INTO event_score (reporter_id, content, score, score_from, publish_year, publish_month, state) VALUES (?, ?, ?, ?, ?, ?, ?)";
                let r = sqlx::query(sql)
                    .bind(prms.reporter_id)
                    .bind(prms.content)
                    .bind(prms.score)
                    .bind(prms.score_from)
                    .bind(prms.publish_year)
                    .bind(prms.publish_month)
                    .bind(prms.state)
                    .execute(&mut conn)
                    .await?;
                Ok(Json(json!({
                    "success": true,
                    "errMsg": "新建成功",
                    "data": r.last_insert_rowid()
                })))
            }
        }
    }
}
