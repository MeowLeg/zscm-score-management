use super::*;
use uuid::Uuid;

pub struct Login;

#[derive(Debug, Deserialize)]
pub struct LoginReq {
    pub name: String,
    pub password: String,
}

fn md5_encrypt(pswd: &str) -> String {
    let digest = md5::compute(pswd.as_bytes());
    format!("{digest:x}")
}

fn get_uuid() -> String {
    let u = Uuid::new_v4();
    format!("{u}").split("-").collect::<String>()
}


#[derive(Debug, Serialize, FromRow)]
pub struct LoginResponse {
    pub id: u32,
    pub name: String,
    pub department: String,
    // pub password: String,
    #[sqlx(default)]
    pub session_id: String,
}


impl ExecSql<LoginReq> for Login {
    async fn handle_post_with_redis_cli(
        cfg: Extension<Arc<Config>>,
        rds: Extension<Arc<Client>>,
        prms: Result<Json<LoginReq>, JsonRejection>
    ) -> Result<Json<Value>, WebErr> {
        let mut conn = SqliteConnection::connect(&cfg.db_path).await?;
        let Json(params) = prms?;
        let sql_sel = r#"
            select id, name, department from admin where name = ? and password = ?
        "#;
        let password_md5 = md5_encrypt(&params.password);
        println!("password_md5: {}", &password_md5);
        let mut r = sqlx::query_as::<Sqlite, LoginResponse>(sql_sel)
            .bind(&params.name)
            .bind(&password_md5)
            .fetch_one(&mut conn)
            .await?;
        let u4_str = get_uuid();
        r.session_id = u4_str.clone();

        let mut rds_con = rds.get_multiplexed_async_connection().await?;
        let _: () = rds_con.set(format!("zscm_{}", &params.name), &u4_str).await?; // surprise
        r.session_id = u4_str;


        Ok(Json(json!({
            "success": true,
            "errMsg": "account data got",
            "data": r
        })))

    }
}
