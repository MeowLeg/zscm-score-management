use super::*;

pub struct GetPaperAbnormalArticles;

#[derive(Debug, Deserialize)]
pub struct GetPaperAbnormalArticlesReq {
    pub year: u16,
    pub month: u8,
    pub keyword: Option<String>,
    pub reporter_id: Option<u32>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct GetPaperAbnormalArticlesResp {
    pub articles: Vec<PaperAbnormalArticle>,
    pub total: i64,
}

#[derive(Debug, Serialize, FromRow, Clone)]
pub struct PaperAbnormalArticleReporterScore {
    pub reporter_id: i32,
    pub reporter_name: String,
    pub department: String,
    pub reporter_category_id: i32,
    pub reporter_category_name: String,
    pub score: u32,
}

#[derive(Debug, Serialize)]
pub struct PaperAbnormalArticle {
    pub id: i32,
    pub title: String,
    pub tv_or_paper: u8,
    pub media_type: u8,
    pub program_name: String,
    pub publish_year: u32,
    pub publish_month: u32,
    pub publish_day: u32,
    pub tv_url: String,
    pub paper_url: String,
    pub page_meta_id: u32,
    pub page_name: String,
    pub score_basic: u32,
    pub score_action: u32,
    pub reporter_scores: Vec<PaperAbnormalArticleReporterScore>,
    // new
    pub content: String,
    pub html_content: String,
    pub ref_id: u32,
    pub duration: u32,
    pub character_count: u32,
}

#[derive(Debug, FromRow)]
pub struct PaperAbnormalArticleRow {
    pub id: i32,
    pub title: String,
    pub tv_or_paper: u8,
    pub media_type: u8,
    pub program_name: String,
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

impl ExecSql<GetPaperAbnormalArticlesReq> for GetPaperAbnormalArticles {
    async fn handle_get_with_headers(
        headers: http::HeaderMap,
        cfg: Extension<Arc<Config>>,
        prms: Option<Query<GetPaperAbnormalArticlesReq>>,
    ) -> Result<Json<Value>, WebErr> {
        let account = match headers.get("account") {
            Some(ant) => ant.to_str()?,
            None => ""
        };
        if account.is_empty() {
            return Err("no account".into());
        }
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
                p.media_type,
                p.name as program_name,
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
            from program p, article a {}
                left join page_meta pm on a.page_meta_id = pm.id
                where
                {}
                a.state = 1
                and a.tv_or_paper not in (0,2,3,4,5)
                and a.publish_year = ?
                and a.publish_month = ?
                and a.tv_or_paper = p.site_id
                and p.state = 1
                {}
            group by a.id
            order by a.publish_year desc, a.publish_month desc, a.publish_day desc
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
            match &prms.keyword {
                Some(keyword) => format!("and a.title like '%{}%'", keyword),
                None => String::new(),
            }
        );
        // println!("sql is {}", &sql);
        let article_rows = sqlx::query_as::<Sqlite, PaperAbnormalArticleRow>(&sql)
            .bind(prms.year)
            .bind(prms.month)
            .fetch_all(&mut conn)
            .await?;

        // 获取文章ID列表
        let article_ids: Vec<i32> = article_rows.iter().map(|row| row.id).collect();

        // 获取所有文章的记者得分信息
        let mut article_reporter_scores_map: std::collections::HashMap<
            i32,
            Vec<PaperAbnormalArticleReporterScore>,
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
                    r.department,
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
                sqlx::query_as::<Sqlite, (i32, i32, String, String, i32, String, u32)>(&sql)
                    .fetch_all(&mut conn)
                    .await?;

            // 构建文章ID到记者得分的映射
            for (
                article_id,
                reporter_id,
                reporter_name,
                department,
                reporter_category_id,
                reporter_category_name,
                score,
            ) in reporter_scores
            {
                let score_info = PaperAbnormalArticleReporterScore {
                    reporter_id,
                    reporter_name,
                    department,
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

        let normal_departments: Vec<String> = vec!["时政要闻部".into(), "民生专题部".into(), "经济专题部".into()];
        // 构建包含记者得分的文章列表
        let articles: Vec<PaperAbnormalArticle> = article_rows
            .into_iter()
            .filter(|row| {
                let reporter_scores = article_reporter_scores_map
                    .get(&row.id)
                    .cloned()
                    .unwrap_or_default();
                if reporter_scores.is_empty() {
                    return false;
                }
                let mut is_out_department = false;
                for r in reporter_scores.iter() {
                    if normal_departments.contains(&r.department) {
                        return false;
                    }
                    if &r.department == "外部" || r.department.len() == 0 {
                        is_out_department = true;
                    }
                }
                !is_out_department
            })
            .map(|row| {
                let reporter_scores = article_reporter_scores_map
                    .get(&row.id)
                    .cloned()
                    .unwrap_or_default();
                PaperAbnormalArticle {
                    id: row.id,
                    title: row.title,
                    tv_or_paper: row.tv_or_paper,
                    publish_year: row.publish_year,
                    publish_month: row.publish_month,
                    publish_day: row.publish_day,
                    tv_url: row.tv_url,
                    media_type: row.media_type,
                    program_name: row.program_name,
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

        let articles_len = articles.len();
        Ok(Json(json!({
            "success": true,
            "errMsg": "",
            "data": GetPaperAbnormalArticlesResp {
                articles,
                total: articles_len as i64,
            }
        })))
    }
}
