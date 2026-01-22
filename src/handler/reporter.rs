use super::*;

pub struct Reporter;

#[derive(Debug, Deserialize)]
pub struct ReporterRequest {
    id: Option<u32>,
    name: String,
    phone: String,
    department: String,
    reporter_category_id: u32,
    state: u32,
}

impl ExecSql<ReporterRequest> for Reporter {
    async fn handle_post(
        cfg: Extension<Arc<Config>>,
        prms: Result<Json<ReporterRequest>, JsonRejection>,
    ) -> Result<Json<Value>, WebErr> {
        let Json(prms) = prms?;
        let mut conn = SqliteConnection::connect(&cfg.db_path).await?;
        match prms.id {
            Some(id) => {
                // 栓除
                if prms.state == 0 {
                    let sql = "UPDATE reporter set state = 0 WHERE id = ?";
                    let rows_affected = sqlx::query(sql)
                        .bind(id)
                        .execute(&mut conn)
                        .await?;
                    let row_affected_num = rows_affected.rows_affected();
                    if row_affected_num == 0 {
                        return Err("用户不存在".into());
                    } else {
                        return Ok(Json(json!({
                            "success": true,
                            "errMsg": "删除成功",
                            "data": row_affected_num
                        })));
                    }
                } else {
                    // 更新用户
                    let sql = r#"
                        UPDATE reporter SET name = ?, phone = ?, department = ?,
                            reporter_category_id = ?, state = ? WHERE id = ?
                        "#;
                    let rows_affected = sqlx::query(sql)
                        .bind(&prms.name)
                        .bind(&prms.phone)
                        .bind(&prms.department)
                        .bind(prms.reporter_category_id)
                        .bind(prms.state)
                        .bind(id)
                        .execute(&mut conn)
                        .await?;
                    let row_affected_num = rows_affected.rows_affected();
                    if row_affected_num == 0 {
                        return Err("用户不存在".into());
                    } else {
                        return Ok(Json(json!({
                            "success": true,
                            "errMsg": "更新成功",
                            "data": row_affected_num
                        })));
                    }
                }
            }
            None => {
                // 插入用户
                let sql = "INSERT INTO reporter (name, phone, department, reporter_category_id, state) VALUES (?, ?, ?, ?, ?)";
                let rows_affected = sqlx::query(sql)
                    .bind(&prms.name)
                    .bind(&prms.phone)
                    .bind(&prms.department)
                    .bind(prms.reporter_category_id)
                    .bind(prms.state)
                    .execute(&mut conn)
                    .await?;
                if rows_affected.rows_affected() == 0 {
                    return Err("插入用户失败".into());
                } else {
                    return Ok(Json(json!({
                        "success": true,
                        "errMsg": "插入成功",
                        "data": rows_affected.last_insert_rowid()
                    })));
                }
            }
        }
    }
}
