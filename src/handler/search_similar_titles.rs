use super::*;
use std::collections::{HashMap, HashSet};

pub struct SearchSimilarTitles;

#[derive(Debug, Deserialize)]
pub struct SearchSimilarTitlesReq {
    title: String,
    threshold: Option<f64>,
    limit: Option<i32>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct VectorVocab {
    word: String,
    idx: i32,
}

#[derive(Debug, Serialize)]
pub struct SimilarTitle {
    id: i64,
    article_id: i64,
    title: String,
    similarity: f64,
    publish_year: i32,
    publish_month: i32,
    publish_day: i32,
    page_name: String,
    page_meta_id: i64,
}

#[derive(Debug, FromRow)]
struct TitleVectorRow {
    id: i64,
    article_id: i64,
    title: String,
    vector: Vec<u8>,
    publish_year: i32,
    publish_month: i32,
    publish_day: i32,
    page_name: String,
    page_meta_id: i64,
}

impl ExecSql<SearchSimilarTitlesReq> for SearchSimilarTitles {
    async fn handle_get(
        cfg: Extension<Arc<Config>>,
        prms: Option<Query<SearchSimilarTitlesReq>>,
    ) -> Result<Json<Value>, WebErr> {
        let Query(prms) = prms.ok_or("Missing parameters")?;
        let threshold = prms.threshold.unwrap_or(0.5);
        let limit = prms.limit.unwrap_or(10);
        
        let mut conn = SqliteConnection::connect(&cfg.db_path).await?;
        
        let vocab_rows = sqlx::query_as::<Sqlite, VectorVocab>(
            "SELECT word, idx FROM vector_vocab ORDER BY idx"
        )
        .fetch_all(&mut conn)
        .await?;
        
        let vocab: HashMap<&str, usize> = vocab_rows
            .iter()
            .map(|row| (row.word.as_str(), row.idx as usize))
            .collect();
        
        let vocab_size = vocab.len();
        
        let query_vector = get_embedding(&prms.title, &vocab, vocab_size);
        
        let vector_rows = sqlx::query_as::<Sqlite, TitleVectorRow>(
            r#"
            SELECT 
                tv.id, tv.article_id, tv.title, tv.vector,
                a.publish_year, a.publish_month, a.publish_day,
                a.page_name, a.page_meta_id
            FROM title_vector tv
            JOIN article a ON tv.article_id = a.id
            "#
        )
        .fetch_all(&mut conn)
        .await?;
        
        let mut results = Vec::new();
        
        for row in vector_rows {
            let vector_str = String::from_utf8(row.vector)?;
            let stored_vector: Vec<f64> = serde_json::from_str(&vector_str)?;
            let similarity = cosine_similarity(&query_vector, &stored_vector);
            
            if similarity >= threshold {
                results.push(SimilarTitle {
                    id: row.id,
                    article_id: row.article_id,
                    title: row.title,
                    similarity,
                    publish_year: row.publish_year,
                    publish_month: row.publish_month,
                    publish_day: row.publish_day,
                    page_name: row.page_name,
                    page_meta_id: row.page_meta_id,
                });
            }
        }
        
        results.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());
        results.truncate(limit as usize);
        
        Ok(Json(json!({
            "success": true,
            "errMsg": "相似标题查询成功",
            "data": results,
        })))
    }
}

fn tokenize(text: &str) -> Vec<String> {
    let stop_words: HashSet<&str> = [
        "的", "是", "在", "有", "和", "与", "及", "或", "但", "而", "也", "都", "就", "被", "把", "让", "给", "到", "从", "向", "对", "于", "等", "这", "那", "个", "些", "之", "为", "以", "因", "所", "可以", "能够", "已经", "正在", "将要", "不会", "没有", "不是", "就是", "还是", "或者", "并且", "但是", "然而", "因此", "所以", "因为", "由于", "虽然", "尽管", "即使", "就算", "只要", "只有", "除非", "无论", "不管", "凡是", "所有", "全部", "一切", "任何", "每个", "各个", "各种", "各类", "各项", "各位", "许多", "很多", "不少", "大量", "众多", "诸多", "若干", "一些", "一点", "少许", "稍微", "比较", "相当", "非常", "极其", "特别", "尤其", "更加", "越发", "最", "更", "较", "比", "像", "似", "如", "同", "跟", "以及", "暨", "及于", "至于", "关于", "对于", "针对", "按照", "根据", "依据", "凭借", "通过", "经过", "经由", "鉴于", "出于", "多亏", "幸亏", "幸好", "好在", "可惜", "遗憾", "不幸", "幸运", "可喜", "可贺", "可悲", "可叹", "可怜", "可恨", "可恶", "可气", "可笑"
    ].iter().cloned().collect();
    
    let mut tokens = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    
    while i < chars.len() {
        if !chars[i].is_alphanumeric() && !('\u{4e00}'..='\u{9fff}').contains(&chars[i]) {
            i += 1;
            continue;
        }
        
        let mut found = false;
        
        if i + 1 < chars.len() {
            let two_char: String = chars[i..i+2].iter().collect();
            if stop_words.contains(two_char.as_str()) {
                tokens.push(two_char);
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

fn get_embedding(text: &str, vocab: &HashMap<&str, usize>, vocab_size: usize) -> Vec<f64> {
    let tokens = tokenize(text);
    let mut vector = vec![0.0; vocab_size];
    
    let mut token_count = HashMap::new();
    for token in &tokens {
        *token_count.entry(token.as_str()).or_insert(0) += 1;
    }
    
    let max_count = token_count.values().copied().max().unwrap_or(1) as f64;
    
    for (token, &count) in &token_count {
        if let Some(&idx) = vocab.get(token) {
            vector[idx] = count as f64 / max_count;
        }
    }
    
    vector
}

fn cosine_similarity(vec1: &[f64], vec2: &[f64]) -> f64 {
    if vec1.len() != vec2.len() {
        return 0.0;
    }
    
    let dot_product: f64 = vec1.iter().zip(vec2.iter()).map(|(a, b)| a * b).sum();
    let norm1: f64 = vec1.iter().map(|a| a * a).sum::<f64>().sqrt();
    let norm2: f64 = vec2.iter().map(|b| b * b).sum::<f64>().sqrt();
    
    if norm1 == 0.0 || norm2 == 0.0 {
        return 0.0;
    }
    
    dot_product / (norm1 * norm2)
}
