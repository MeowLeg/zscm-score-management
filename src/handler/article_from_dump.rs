use super::*;

pub struct Article;

#[derive(Debug, Deserialize)]
pub struct ArticleRequest {
    id: Option<u32>,
    title: String,
    tv_or_paper: u8,
    publish_year: u32,
    publish_month: u32,
    publish_day: u32,
    tv_url: String,
    page_meta_id: u8,
    page_name: String,
    state: u8,
    content: String,
    html_content: String,
    ref_id: u32,
    duration: u32,
    character_count: u32,
    is_collaboration: Option<u8>,
}

impl ExecSql<ArticleRequest> for Article {
    async fn handle_post(
        cfg: Extension<Arc<Config>>,
        prms: Result<Json<ArticleRequest>, JsonRejection>,
    ) -> Result<Json<Value>, WebErr> {
        let Json(prms) = prms?;
        // println!("ArticleRequest: {:?}", &prms);
        let mut conn = SqliteConnection::connect(&cfg.db_path).await?;
        let is_collaboration = prms.is_collaboration.unwrap_or(0);
        match prms.id {
            Some(id) => {
                if prms.state == 0 {
                    let sql = "update article set state=? where id=?";
                    let r = sqlx::query(sql)
                        .bind(prms.state)
                        .bind(id)
                        .execute(&mut conn)
                        .await?;
                    Ok(Json(json!({
                        "success": true,
                        "errMsg": "文章删除成功",
                        "data": r.rows_affected(),
                    })))
                } else {
                    let sql = r#"
                        update article set title=?, tv_or_paper=?, publish_year=?,
                            publish_month=?, publish_day=?, tv_url=?, page_meta_id=?,
                            content=?, html_content=?, page_name=?,
                            ref_id=?, duration=?, character_count=?,
                            state=?, is_collaboration=?
                        where id=?
                    "#;
                    let r = sqlx::query(sql)
                        .bind(&prms.title)
                        .bind(prms.tv_or_paper)
                        .bind(&prms.publish_year)
                        .bind(&prms.publish_month)
                        .bind(&prms.publish_day)
                        .bind(&prms.tv_url)
                        .bind(prms.page_meta_id)
                        .bind(&prms.page_name)
                        .bind(&prms.content)
                        .bind(&prms.html_content)
                        .bind(&prms.ref_id)
                        .bind(&prms.duration)
                        .bind(&prms.character_count)
                        .bind(prms.state)
                        .bind(is_collaboration)
                        .bind(id)
                        .execute(&mut conn)
                        .await?;

                    Ok(Json(json!({
                        "success": true,
                        "errMsg": "文章更新成功",
                        "data": r.rows_affected(),
                    })))
                }
            }
            None => {
                let sql = r#"
                    insert into article(
                        title, tv_or_paper, publish_year, publish_month,
                        publish_day, tv_url, page_meta_id, page_name,
                        content, html_content, state, ref_id, duration,
                        character_count, is_collaboration)
                    values(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)
                "#;
                match sqlx::query(sql)
                    .bind(&prms.title)
                    .bind(prms.tv_or_paper)
                    .bind(&prms.publish_year)
                    .bind(&prms.publish_month)
                    .bind(&prms.publish_day)
                    .bind(&prms.tv_url)
                    .bind(prms.page_meta_id)
                    .bind(&prms.page_name)
                    .bind(&prms.content)
                    .bind(&prms.html_content)
                    .bind(prms.state)
                    .bind(prms.ref_id)
                    .bind(prms.duration)
                    .bind(prms.character_count)
                    .bind(is_collaboration)
                    .execute(&mut conn)
                    .await
                {
                    Ok(r) => Ok(Json(json!({
                        "success": true,
                        "errMsg": "文章新建成功",
                        "data": r.last_insert_rowid(),
                    }))),
                    Err(e) => {
                        println!("Error: {}", &e);
                        Err(e.into())
                    }
                }
            }
        }
    }
}
