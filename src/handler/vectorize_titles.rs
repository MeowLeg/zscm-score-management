use super::*;
use std::collections::HashMap;

pub struct VectorizeTitles;

#[derive(Debug, Deserialize)]
pub struct VectorizeTitlesReq {
    pub force: Option<bool>,
    pub rebuild_vocab: Option<bool>,
}

#[derive(Debug, FromRow)]
struct ArticleRow {
    id: i64,
    title: String,
}

#[derive(Debug, FromRow)]
struct VocabRow {
    word: String,
    idx: i64,
}

const STOP_WORDS: &[&str] = &[
    "的", "是", "在", "有", "和", "与", "及", "或", "但", "而", "也", "都", "就", "被", "把", "让", "给", "到", "从", "向", "对", "于", "等", "这", "那", "个", "些", "之", "为", "以", "因", "所", "可以", "能够", "已经", "正在", "将要", "不会", "没有", "不是", "就是", "还是", "或者", "并且", "但是", "然而", "因此", "所以", "因为", "由于", "虽然", "尽管", "即使", "就算", "只要", "只有", "除非", "无论", "不管", "凡是", "所有", "全部", "一切", "任何", "每个", "各个", "各种", "各类", "各项", "各位", "许多", "很多", "不少", "大量", "众多", "诸多", "若干", "一些", "一点", "少许", "稍微", "比较", "相当", "非常", "极其", "特别", "尤其", "更加", "越发", "最", "更", "较", "比", "像", "似", "如", "同", "跟", "和", "与", "及", "以及", "暨", "及于", "至于", "关于", "对于", "针对", "按照", "根据", "依据", "凭借", "通过", "经过", "经由", "由于", "因为", "鉴于", "出于", "多亏", "幸亏", "幸好", "好在", "可惜", "遗憾", "不幸", "幸运", "可喜", "可贺", "可悲", "可叹", "可怜", "可恨", "可恶", "可气", "可笑",
];

fn tokenize(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    let n = chars.len();

    while i < n {
        if chars[i].is_whitespace() || !chars[i].is_alphanumeric() && !('\u{4e00}'..='\u{9fff}').contains(&chars[i]) {
            i += 1;
            continue;
        }

        let mut found = false;
        if i + 1 < n {
            let bigram: String = chars[i..i+2].iter().collect();
            if STOP_WORDS.contains(&bigram.as_str()) {
                tokens.push(bigram);
                i += 2;
                found = true;
            }
        }

        if !found {
            tokens.push(chars[i].to_string());
            i += 1;
        }
    }

    tokens
}

async fn load_or_build_vocab(
    conn: &mut SqliteConnection,
    rebuild: bool,
) -> Result<HashMap<String, i64>, WebErr> {
    let mut vocab = HashMap::new();

    if rebuild {
        sqlx::query("DELETE FROM vector_vocab")
            .execute(&mut *conn)
            .await?;
        build_vocab(conn, &mut vocab).await?;
    } else {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) as cnt FROM vector_vocab")
            .fetch_one(&mut *conn)
            .await?;

        if count.0 > 0 {
            let rows: Vec<VocabRow> = sqlx::query_as("SELECT word, idx FROM vector_vocab ORDER BY idx")
                .fetch_all(&mut *conn)
                .await?;
            for row in rows {
                vocab.insert(row.word, row.idx);
            }
        } else {
            build_vocab(conn, &mut vocab).await?;
        }
    }

    Ok(vocab)
}

async fn build_vocab(
    conn: &mut SqliteConnection,
    vocab: &mut HashMap<String, i64>,
) -> Result<(), WebErr> {
    let articles: Vec<ArticleRow> = sqlx::query_as(
        "SELECT id, title FROM article WHERE tv_or_paper != 0 AND tv_or_paper != 2"
    )
    .fetch_all(&mut *conn)
    .await?;

    let mut word_count = HashMap::new();
    for article in articles {
        let tokens = tokenize(&article.title);
        for token in tokens {
            *word_count.entry(token).or_insert(0) += 1;
        }
    }

    let mut sorted_words: Vec<_> = word_count.into_iter().collect();
    sorted_words.sort_by(|a, b| b.1.cmp(&a.1));
    sorted_words.truncate(10000);

    for (idx, (word, _)) in sorted_words.into_iter().enumerate() {
        let idx_i64 = idx as i64;
        vocab.insert(word.clone(), idx_i64);
        sqlx::query("INSERT INTO vector_vocab (word, idx) VALUES (?, ?)")
            .bind(word)
            .bind(idx_i64)
            .execute(&mut *conn)
            .await?;
    }

    Ok(())
}

fn get_embedding(text: &str, vocab: &HashMap<String, i64>) -> Option<Vec<f64>> {
    let tokens = tokenize(text);
    let dim = vocab.len();
    let mut vector = vec![0.0; dim];

    let mut token_count = HashMap::new();
    for token in tokens {
        *token_count.entry(token).or_insert(0) += 1;
    }

    let max_count = token_count.values().copied().max().unwrap_or(1) as f64;

    for (token, count) in token_count {
        if let Some(&idx) = vocab.get(&token) {
            if let Some(vec_elem) = vector.get_mut(idx as usize) {
                *vec_elem = count as f64 / max_count;
            }
        }
    }

    Some(vector)
}

async fn vectorize_all(
    conn: &mut SqliteConnection,
    force: bool,
    rebuild_vocab: bool,
) -> Result<(usize, String), WebErr> {
    let vocab = load_or_build_vocab(conn, rebuild_vocab).await?;

    let mut query = String::from(
        "SELECT a.id, a.title FROM article a WHERE a.tv_or_paper != 0 AND a.tv_or_paper != 2"
    );

    if !force {
        query.push_str(" AND a.id NOT IN (SELECT article_id FROM title_vector)");
    }

    query.push_str(" ORDER BY a.id");

    let articles: Vec<ArticleRow> = sqlx::query_as(&query)
        .fetch_all(&mut *conn)
        .await?;

    let count = articles.len();
    let mut processed = 0;

    for (_i, article) in articles.into_iter().enumerate() {
        if let Some(vector) = get_embedding(&article.title, &vocab) {
            let vector_json = serde_json::to_string(&vector)?;
            let vector_blob = vector_json.as_bytes();

            sqlx::query(
                "INSERT OR REPLACE INTO title_vector (article_id, title, vector, vector_dim, updated_at) VALUES (?, ?, ?, ?, CURRENT_TIMESTAMP)"
            )
            .bind(article.id)
            .bind(&article.title)
            .bind(vector_blob)
            .bind(vector.len() as i64)
            .execute(&mut *conn)
            .await?;

            processed += 1;
        }
    }

    Ok((processed, format!("向量化完成，共处理 {} 条", count)))
}

impl ExecSql<VectorizeTitlesReq> for VectorizeTitles {
    async fn handle_get(
        cfg: Extension<Arc<Config>>,
        prms: Option<Query<VectorizeTitlesReq>>,
    ) -> Result<Json<Value>, WebErr> {
        let prms = prms.unwrap_or_else(|| Query(VectorizeTitlesReq {
            force: None,
            rebuild_vocab: None,
        }));

        let mut conn = SqliteConnection::connect(&cfg.db_path).await?;
        let (processed, message) = vectorize_all(
            &mut conn,
            prms.force.unwrap_or(false),
            prms.rebuild_vocab.unwrap_or(false),
        ).await?;

        Ok(Json(json!({
            "success": true,
            "processed": processed,
            "message": message
        })))
    }
}
