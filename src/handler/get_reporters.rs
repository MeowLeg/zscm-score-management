use super::*;

pub struct GetReporters;

#[derive(Debug, Deserialize)]
pub struct GetReportersReq {
    department: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct Reporter {
    id: i64,
    name: String,
    phone: String,
    department: String,
    reporter_category_id: u32,
    reporter_category_name: String,
    state: u32,
}

impl ExecSql<GetReportersReq> for GetReporters {
    async fn handle_get(
        cfg: Extension<Arc<Config>>,
        prms: Option<Query<GetReportersReq>>,
    ) -> Result<Json<Value>, WebErr> {
        let Query(prms) = prms.ok_or("Missing parameters")?;
        let mut conn = SqliteConnection::connect(&cfg.db_path).await?;
        let sql = format!(r#"
                select
                    r.id, r.name, r.phone, r.reporter_category_id, r.department,
                    rc.name as reporter_category_name, 
                    r.state
                from reporter r
                join reporter_category rc on r.reporter_category_id = rc.id
                where r.state = 1 {}
            "#,
            match prms.department {
                Some(department) => format!("and r.department = '{}'", department),
                None => "".to_string(),
            }
        );
        let rows = sqlx::query_as::<Sqlite, Reporter>(&sql)
            .fetch_all(&mut conn)
            .await?;
        Ok(Json(json!({
            "success": true,
            "errMsg": "记者列表数据获取成功",
            "data": rows,
        })))
    }
}
