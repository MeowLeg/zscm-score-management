use super::*;

pub struct Score {}

#[derive(Debug, Deserialize)]
pub struct ScoreRequest {
    article_id: u32,
    score_basic: u32,
    score_action: u32,
    reporter_scores: Vec<UserScore>,
}

#[derive(Debug, Deserialize)]
pub struct UserScore {
    reporter_id: Option<u32>,
    reporter_name: Option<String>,
    reporter_category_id: u32,
    ref_code: Option<String>,
    score: u32,
}

impl ExecSql<ScoreRequest> for Score {
    async fn handle_post(
        cfg: Extension<Arc<Config>>,
        prms: Result<Json<ScoreRequest>, JsonRejection>,
    ) -> Result<Json<Value>, WebErr> {
        let Json(prms) = prms?;
        println!("param: {:?}", &prms);
        let mut conn = SqliteConnection::connect(&cfg.db_path).await?;
        let del_sql = format!(
            "DELETE FROM article_reporter_score WHERE article_id = {}",
            prms.article_id
        );
        let _ = sqlx::query(&del_sql).execute(&mut conn).await?;

        let update_sql = r#"
            update article set score_basic = ?, score_action = ? where id = ?
        "#;
        let _ = sqlx::query(&update_sql)
            .bind(prms.score_basic)
            .bind(prms.score_action)
            .bind(prms.article_id)
            .execute(&mut conn)
            .await?;

        let insert_sql = r#"
            insert into article_reporter_score (article_id, reporter_id, reporter_category_id, score)
            values (?, ?, ?, ?)
        "#;
        for reporter_score in prms.reporter_scores.into_iter() {
            println!("{:?}", &reporter_score);
            match reporter_score.reporter_id {
                Some(r_id) => {
                    println!("reporter_id: {:?}", &r_id);
                    let _ = sqlx::query(&insert_sql)
                        .bind(prms.article_id)
                        .bind(r_id)
                        .bind(reporter_score.reporter_category_id)
                        .bind(reporter_score.score)
                        .execute(&mut conn)
                        .await?;
                }
                None => match reporter_score.ref_code {
                    Some(code) => {
                        println!("ref_code: {:?}", &code);
                        let s_sql = "select id from reporter where ref_code = ?";
                        match sqlx::query_as::<Sqlite, (u32,)>(&s_sql)
                            .bind(&code)
                            .fetch_one(&mut conn)
                            .await
                        {
                            Ok(r_id) => {
                                println!("ref_code: {:?}, reporter_id: {:?}", &code, &r_id.0);
                                let _ = sqlx::query(&insert_sql)
                                    .bind(prms.article_id)
                                    .bind(r_id.0)
                                    .bind(reporter_score.reporter_category_id)
                                    .bind(reporter_score.score)
                                    .execute(&mut conn)
                                    .await?;
                            }
                            Err(e) => {
                                println!("can not find reporter_id by ref_code: {:?}", &code);
                                println!("Error fetching reporter ID: {}", e);
                            }
                        };
                    }
                    None => match reporter_score.reporter_name {
                        Some(name) => {
                            let s_sql = if reporter_score.reporter_category_id == 7 {
                                "select id from reporter where name = ? and reporter_category_id = 7"
                            } else {
                                "select id from reporter where name = ? and reporter_category_id != 7"
                            }; 
                            match sqlx::query_as::<Sqlite, (u32,)>(&s_sql)
                                .bind(&name)
                                .fetch_one(&mut conn)
                                .await
                            {
                                Ok(r_id) => {
                                    let _ = sqlx::query(&insert_sql)
                                        .bind(prms.article_id)
                                        .bind(r_id.0)
                                        .bind(reporter_score.reporter_category_id)
                                        .bind(reporter_score.score)
                                        .execute(&mut conn)
                                        .await?;
                                }
                                Err(e) => {
                                    // 通讯员
                                    // 通讯员仅仅靠名字来处理，因此存在同名的情况，但是因为是月结，只要再一个月内不存在同名的人就没有问题
                                    if reporter_score.reporter_category_id == 7 {
                                        let i_sql = "insert into reporter (name, phone, ref_code, department, reporter_category_id, state) values (?, ?, ?, ?, ?, ?)";
                                        let r = sqlx::query(&i_sql)
                                            .bind(&name)
                                            .bind("")
                                            .bind("")
                                            .bind("外部")
                                            .bind(reporter_score.reporter_category_id)
                                            .bind(1)
                                            .execute(&mut conn)
                                            .await?;
                                        let r_id = r.last_insert_rowid();
                                        let _ = sqlx::query(&insert_sql)
                                            .bind(prms.article_id)
                                            .bind(r_id)
                                            .bind(reporter_score.reporter_category_id)
                                            .bind(reporter_score.score)
                                            .execute(&mut conn)
                                            .await?;
                                    } else {
                                        println!("Error fetching reporter ID: {}", e);
                                    }
                                }
                            }
                        }
                        None => {
                            println!("Reporter name, ref_code, id not found");
                        }
                    },
                },
            }
        }

        Ok(Json(json!({
            "success": true,
            "errMsg": "操作成功",
            "data": ""
        })))
    }
}
