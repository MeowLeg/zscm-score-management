use super::*;

pub struct MonthlyAddScore;

#[derive(Debug, Deserialize)]
pub struct MonthlyAddScoreReq {
    pub id: Option<i32>,
    pub reporter_id: i32,
    pub score: i32,
    pub reason: String,
    pub publish_year: i32,
    pub publish_month: i32,
    pub state: i32,
}

impl ExecSql<MonthlyAddScoreReq> for MonthlyAddScore {
    async fn handle_post(
        cfg: Extension<Arc<Config>>,
        prms: Result<Json<MonthlyAddScoreReq>, JsonRejection>,
    ) -> Result<Json<Value>, WebErr> {
        let prms = prms?;
        let mut conn = SqliteConnection::connect(&cfg.db_path).await?;
        match prms.id {
            Some(id) => {
                if prms.state == 0 {
                    let sql = "DELETE FROM monthly_add_score WHERE id = ?";
                    let r = sqlx::query(sql)
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
                    let sql = r#"
                        update monthly_add_score
                        set
                            score = ?,
                            reason = ?,
                            publish_year = ?,
                            publish_month = ?
                        where id = ?
                    "#;
                    let r= sqlx::query(sql)
                        .bind(prms.score)
                        .bind(&prms.reason)
                        .bind(prms.publish_year)
                        .bind(prms.publish_month)
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
                }
            },
            None => {
                let sql = r#"
                    insert into monthly_add_score
                        (reporter_id, score, reason, publish_year, publish_month)
                    values (?, ?, ?, ?, ?)
                "#;
                let r = sqlx::query(sql)
                    .bind(prms.reporter_id)
                    .bind(prms.score)
                    .bind(&prms.reason)
                    .bind(prms.publish_year)
                    .bind(prms.publish_month)
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