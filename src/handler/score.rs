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
                        let s_sql = "select id from reporter where ref_code = ?";
                        match sqlx::query_as::<Sqlite, (u32,)>(&s_sql)
                            .bind(code)
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
                                println!("Error fetching reporter ID: {}", e);
                            }
                        };
                    }
                    None => match reporter_score.reporter_name {
                        Some(name) => {
                            let s_sql = "select id from reporter where name = ?";
                            match sqlx::query_as::<Sqlite, (u32,)>(&s_sql)
                                .bind(name)
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
                                    println!("Error fetching reporter ID: {}", e);
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
