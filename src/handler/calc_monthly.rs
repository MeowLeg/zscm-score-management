use super::*;

pub struct CalcMonthly;

#[derive(Debug, Deserialize)]
pub struct CalcMonthlyReq {
    pub year: u32,
    pub month: u32,
    pub reporter_id: Option<u32>, // 添加记者ID参数，可选
}

#[derive(Debug, Serialize, FromRow)]
struct CalcMonthlyResp {
    id: i32,
    name: String,
    phone: String,
    department: String,
    reporter_category_id: i32,
    reporter_category_name: String,
    article_count: i32,
    article_score: i32,
    event_score: i32,
    cooperation_score: i32,
    monthly_add_score: i32,
    monthly_sub_score: i32,
    total_score: i32,
}

impl ExecSql<CalcMonthlyReq> for CalcMonthly {
    async fn handle_get(
        cfg: Extension<Arc<Config>>,
        prms: Option<Query<CalcMonthlyReq>>,
    ) -> Result<Json<Value>, WebErr> {
        let prms = prms.ok_or("Missing parameters")?;
        let mut conn = SqliteConnection::connect(&cfg.db_path).await?;
        
        // 构建SQL查询，修复字段名不匹配、添加必要信息
        let sql = r#"
        SELECT
            r.id,
            r.name,
            r.phone,
            r.department,
            r.reporter_category_id,
            rc.name as reporter_category_name,
            COALESCE(article_stats.article_count, 0) as article_count,
            COALESCE(article_stats.article_score, 0) as article_score,
            COALESCE(event_score.total_event_score, 0) as event_score,
            COALESCE(cooperation_score.total_cooperation_score, 0) as cooperation_score,
            COALESCE(monthly_add_score.total_monthly_add_score, 0) as monthly_add_score,
            COALESCE(monthly_sub_score.total_monthly_sub_score, 0) as monthly_sub_score,
            (
                COALESCE(article_stats.article_score, 0) + 
                COALESCE(event_score.total_event_score, 0) + 
                COALESCE(cooperation_score.total_cooperation_score, 0) + 
                COALESCE(monthly_add_score.total_monthly_add_score, 0) - 
                COALESCE(monthly_sub_score.total_monthly_sub_score, 0)
            ) as total_score
        FROM reporter r
        JOIN reporter_category rc ON r.reporter_category_id = rc.id
        LEFT JOIN (
            SELECT
                ars.reporter_id,
                COUNT(DISTINCT ars.article_id) as article_count,
                SUM(ars.score) as article_score
            FROM article_reporter_score ars
            JOIN article a ON ars.article_id = a.id
            WHERE a.publish_year = ?
                AND a.publish_month = ?
                AND a.state = 1
            GROUP BY ars.reporter_id
        ) AS article_stats ON r.id = article_stats.reporter_id
        LEFT JOIN (
            SELECT
                es.reporter_id,
                SUM(es.score) as total_event_score
            FROM event_score es
            WHERE es.publish_year = ?
                AND es.publish_month = ?
                AND es.state = 1
            GROUP BY es.reporter_id
        ) AS event_score ON r.id = event_score.reporter_id
        LEFT JOIN (
            SELECT
                cs.reporter_id,
                SUM(cs.score) as total_cooperation_score
            FROM cooperation_score cs
            WHERE cs.publish_year = ?
                AND cs.publish_month = ?
                AND cs.state = 1
            GROUP BY cs.reporter_id
        ) AS cooperation_score ON r.id = cooperation_score.reporter_id
        LEFT JOIN (
            SELECT
                mas.reporter_id,
                SUM(mas.score) as total_monthly_add_score
            FROM monthly_add_score mas
            WHERE mas.publish_year = ?
                AND mas.publish_month = ?
                AND mas.state = 1
            GROUP BY mas.reporter_id
        ) AS monthly_add_score ON r.id = monthly_add_score.reporter_id
        LEFT JOIN (
            SELECT
                mss.reporter_id,
                SUM(mss.score) as total_monthly_sub_score
            FROM monthly_sub_score mss
            WHERE mss.publish_year = ?
                AND mss.publish_month = ?
                AND mss.state = 1
            GROUP BY mss.reporter_id
        ) AS monthly_sub_score ON r.id = monthly_sub_score.reporter_id
        WHERE r.state = 1
        "#;
        
        // 添加记者ID筛选条件（如果提供）
        let sql = match prms.reporter_id {
            Some(_) => format!("{} AND r.id = ?", sql),
            None => String::from(sql),
        };
        
        // 执行查询并绑定参数
        let mut query = sqlx::query_as::<Sqlite, CalcMonthlyResp>(&sql);
        
        // 绑定年份和月份参数（每个子查询需要一对）
        for _ in 0..5 {
            query = query.bind(prms.year).bind(prms.month);
        }
        
        // 如果提供了记者ID，添加到参数列表
        let query = match prms.reporter_id {
            Some(reporter_id) => query.bind(reporter_id),
            None => query,
        };
        
        let rs = query.fetch_all(&mut conn).await?;

        Ok(Json(json!({
            "success": true,
            "errMsg": "",
            "data": rs
        })))
    }
}