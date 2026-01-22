use super::*;

pub struct GetArticles;

#[derive(Debug, Deserialize)]
pub struct GetArticlesReq {
    pub year: u16,
    pub month: u8,
    pub day: Option<u8>,
    pub page: u32,
    pub limit: u32,
    pub tv_or_paper: Option<String>,
    pub keyword: Option<String>,
    pub reporter_id: Option<u32>,
    pub tv_url: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct GetArticlesResp {
    pub articles: Vec<Article>,
    pub total: i64,
}

#[derive(Debug, Serialize, FromRow, Clone)]
pub struct ArticleReporterScore {
    pub reporter_id: i32,
    pub reporter_name: String,
    pub reporter_category_id: i32,
    pub reporter_category_name: String,
    pub score: u32,
}

#[derive(Debug, Serialize)]
pub struct Article {
    pub id: i32,
    pub title: String,
    pub tv_or_paper: u8,
    pub media_type: u8,
    pub publish_year: u32,
    pub publish_month: u32,
    pub publish_day: u32,
    pub tv_url: String,
    pub paper_url: String,
    pub page_meta_id: u32,
    pub page_name: String,
    pub score_basic: u32,
    pub score_action: u32,
    pub reporter_scores: Vec<ArticleReporterScore>,
    // new
    pub content: String,
    pub html_content: String,
    pub ref_id: u32,
    pub duration: u32,
    pub character_count: u32,
}

#[derive(Debug, FromRow)]
pub struct ArticleRow {
    pub id: i32,
    pub title: String,
    pub tv_or_paper: u8,
    pub media_type: u8,
    pub publish_year: u32,
    pub publish_month: u32,
    pub publish_day: u32,
    pub tv_url: String,
    pub paper_url: String,
    pub page_meta_id: u32,
    pub page_name: String,
    pub score_basic: u32,
    pub score_action: u32,
    pub content: String,
    pub html_content: String,
    pub ref_id: u32,
    pub duration: u32,
    pub character_count: u32,
}

impl ExecSql<GetArticlesReq> for GetArticles {
    async fn handle_get(
        cfg: Extension<Arc<Config>>,
        prms: Option<Query<GetArticlesReq>>,
    ) -> Result<Json<Value>, WebErr> {
        let prms = prms.ok_or("Missing parameters")?;
        let mut conn = SqliteConnection::connect(&cfg.db_path).await?;
        let sql = format!(
            r#"
            select
                a.id,
                a.title,
                a.content,
                a.html_content,
                a.tv_or_paper,
                program.media_type,
                a.publish_year,
                a.publish_month,
                a.publish_day,
                a.tv_url,
                a.page_meta_id,
                'https://epaper.wifizs.cn/zsrb/' || a.publish_year || '-' || substr('00'||a.publish_month, -2) || '/' || substr('00'||a.publish_day, -2) || '/node_' || pm.page_web_no || '.html' as paper_url,
                a.page_name,
                a.score_basic,
                a.score_action,
                a.ref_id,
                a.duration,
                a.character_count
            from article a {}
                left join program on a.tv_or_paper = program.site_id
                left join page_meta pm on a.page_meta_id = pm.id
                where
                {}
                {}
                a.state = 1
                and a.publish_year = ?
                and a.publish_month = ?
                {}
                {}
                {}
            order by a.publish_year desc, a.publish_month desc, a.publish_day desc
            limit {} offset {}
        "#,
            match prms.reporter_id {
                Some(_) => String::from(", article_reporter_score ars"),
                None => String::new(),
            },
            match prms.reporter_id {
                Some(reporter_id) => format!(
                    "a.id = ars.article_id and ars.reporter_id = {} and",
                    reporter_id
                ),
                None => String::new(),
            },
            match prms.tv_or_paper.clone() {
                Some(tv_or_paper) => {
                    match tv_or_paper.parse::<i32>() {
                        Ok(tv_or_paper) => {
                            if tv_or_paper < 0 {
                                String::new()
                            } else {
                                format!("a.tv_or_paper = {} and", tv_or_paper)
                            }
                        },
                        Err(_) => format!("a.tv_or_paper in ({}) and ", tv_or_paper),
                    }
                },
                None => String::new(),
            },
            match prms.day {
                Some(day) => format!("and a.publish_day = {}", day),
                None => String::new(),
            },
            match &prms.keyword {
                Some(keyword) => format!("and a.title like '%{}%'", keyword),
                None => String::new(),
            },
            match &prms.tv_url {
                Some(tv_url) => format!("and a.tv_url = '{}'", tv_url),
                None => String::new(),
            },
            prms.limit,
            (prms.page - 1) * prms.limit
        );
        // println!("sql is {}", &sql);
        let article_rows = sqlx::query_as::<Sqlite, ArticleRow>(&sql)
            .bind(prms.year)
            .bind(prms.month)
            .fetch_all(&mut conn)
            .await?;

        // 获取文章ID列表
        let article_ids: Vec<i32> = article_rows.iter().map(|row| row.id).collect();

        // 获取所有文章的记者得分信息
        let mut article_reporter_scores_map: std::collections::HashMap<
            i32,
            Vec<ArticleReporterScore>,
        > = std::collections::HashMap::new();

        if !article_ids.is_empty() {
            let semicolons_str = article_ids
                .iter()
                .map(|id| format!("{}", id))
                .collect::<Vec<_>>()
                .join(",");
            let sql = format!(
                r#"
                select
                    ars.article_id,
                    r.id as reporter_id,
                    r.name as reporter_name,
                    ars.reporter_category_id,
                    rc.name as reporter_category_name,
                    ars.score
                from article_reporter_score ars
                join reporter r on ars.reporter_id = r.id
                join reporter_category rc on ars.reporter_category_id = rc.id
                where ars.article_id in ({})
            "#,
                semicolons_str
            );

            let reporter_scores =
                sqlx::query_as::<Sqlite, (i32, i32, String, i32, String, u32)>(&sql)
                    .fetch_all(&mut conn)
                    .await?;

            // 构建文章ID到记者得分的映射
            for (
                article_id,
                reporter_id,
                reporter_name,
                reporter_category_id,
                reporter_category_name,
                score,
            ) in reporter_scores
            {
                let score_info = ArticleReporterScore {
                    reporter_id,
                    reporter_name,
                    reporter_category_id,
                    reporter_category_name,
                    score,
                };
                article_reporter_scores_map
                    .entry(article_id)
                    .or_default()
                    .push(score_info);
            }
        }

        // 构建包含记者得分的文章列表
        let articles: Vec<Article> = article_rows
            .into_iter()
            .map(|row| {
                let reporter_scores = article_reporter_scores_map
                    .get(&row.id)
                    .cloned()
                    .unwrap_or_default();
                Article {
                    id: row.id,
                    title: row.title,
                    tv_or_paper: row.tv_or_paper,
                    publish_year: row.publish_year,
                    publish_month: row.publish_month,
                    publish_day: row.publish_day,
                    tv_url: row.tv_url,
                    media_type: row.media_type,
                    paper_url: row.paper_url,
                    page_meta_id: row.page_meta_id,
                    page_name: row.page_name,
                    score_basic: row.score_basic,
                    score_action: row.score_action,
                    reporter_scores,
                    content: row.content,
                    html_content: row.html_content,
                    ref_id: row.ref_id,
                    duration: row.duration,
                    character_count: row.character_count,
                }
            })
            .collect();

        let sql = format!(
            r#"
                select count(a.id)
                from article a
                    {}
                    a.state = 1
                    and a.publish_year = ?
                    and a.publish_month = ?
                    {}
                    {}
            "#,
            match prms.reporter_id {
                Some(reporter_id) => format!(
                    ", article_reporter_score ars where a.id = ars.article_id and ars.reporter_id = {} and",
                    reporter_id
                ),
                None => String::from("where "),
            },
            match &prms.keyword {
                Some(keyword) => format!("and a.title like '%{}%'", keyword),
                None => String::new(),
            },
            match prms.tv_or_paper.clone() {
                Some(tv_or_paper) => format!("and a.tv_or_paper in ({})", tv_or_paper),
                None => String::new(),
            },
        );
        let count = sqlx::query_scalar::<Sqlite, i64>(&sql)
            .bind(prms.year)
            .bind(prms.month)
            .fetch_one(&mut conn)
            .await?;

        Ok(Json(json!({
            "success": true,
            "errMsg": "",
            "data": GetArticlesResp {
                articles,
                total: count,
            }
        })))
    }
}
