use anyhow::Result;
use axum::{
    Extension, Json,
    extract::{Query, rejection::JsonRejection},
    http::StatusCode,
    middleware,
    response::{IntoResponse, Response},
    routing::{Router, get, post},
};
use clap::Parser;
use redis::{AsyncCommands, Client};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::{fs::File, io::Read};

mod handler;
use handler::*;

mod auth;
use auth::*;

#[derive(Debug, Parser)]
#[command(version, about, long_about=None)]
struct Cli {
    /// config file
    #[arg(short, long, default_value = "./config.toml")]
    config: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub port: u32,
    pub db_path: String,
    pub newspaper_zsrb_db_path: String,
    pub newspaper_db_path: String,
}

pub fn read_from_toml(f: &str) -> Result<Config> {
    let mut file = File::open(f)?;
    let mut s = String::new();
    file.read_to_string(&mut s)?;
    let config: Config = toml::from_str(&s)?;
    Ok(config)
}

// type SessionState = Arc<Mutex<HashMap<String, String>>>;

// #[derive(Clone)]
// pub struct AuthAccountMiddleware {
//     session: SessionState,
// }

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let cfg = Arc::new(read_from_toml(&cli.config)?);
    // let session: SessionState = Arc::new(Mutex::new(HashMap::new()));
    // let auth_account_middleware = AuthAccountMiddleware {
    //     session: Arc::clone(&session)
    // };
    let redis_cli_arc = Arc::new(Client::open("redis://127.0.0.1")?);

    let app = Router::new()
        .route("/", get(async || "hello, zscm!".to_string()))
        // update or insert
        .route("/reporter", post(reporter::Reporter::handle_post))
        .route("/score", post(score::Score::handle_post))
        .route("/article", post(article::Article::handle_post))
        .route("/event", post(event::Event::handle_post))
        .route("/cooperation", post(cooperation::Cooperation::handle_post))
        .route(
            "/monthly_add_score",
            post(monthly_add_score::MonthlyAddScore::handle_post),
        )
        .route(
            "/monthly_sub_score",
            post(monthly_sub_score::MonthlySubScore::handle_post),
        )
        // query
        .route(
            "/get_page_meta",
            get(get_page_meta::GetPageMeta::handle_get),
        )
        .route(
            "/get_reporters",
            get(get_reporters::GetReporters::handle_get),
        )
        .route(
            "/get_reporter_categories",
            get(get_reporter_categories::GetReporterCategories::handle_get),
        )
        .route(
            "/get_reporter_events",
            get(get_reporter_events::GetReporterEvents::handle_get),
        )
        .route(
            "/get_cooperation_scores",
            get(get_cooperation_scores::GetCooperationScores::handle_get),
        )
        .route(
            "/get_event_scores",
            get(get_event_scores::GetEventScores::handle_get),
        )
        .route(
            "/get_monthly_add_score",
            get(get_monthly_add_score::GetMonthlyAddScore::handle_get),
        )
        .route(
            "/get_monthly_sub_score",
            get(get_monthly_sub_score::GetMonthlySubScore::handle_get),
        )
        .route("/get_programs", get(get_programs::GetPrograms::handle_get_with_headers))
        // calc
        .route("/calc_monthly", get(calc_monthly::CalcMonthly::handle_get))
        .route("/get_paper_abnormal_articles", get(get_paper_abnormal_articles::GetPaperAbnormalArticles::handle_get_with_headers))
        .route_layer(
            middleware::from_fn_with_state(
                Arc::clone(&redis_cli_arc),
                auth_account
            )
        )
        .route("/score_for_dump", post(score::Score::handle_post))
        .route("/get_articles", get(get_articles::GetArticles::handle_get_with_headers))
        .route("/get_reporter_info_by_name", get(get_reporter_info_by_name::GetReporterInfoByName::handle_get))
        .route("/article_from_dump", post(article_from_dump::Article::handle_post))
        .route("/batch_update_tv_urls", post(batch_update_tv_urls::BatchUpdateTvUrls::handle_post))
        .route("/login", post(login::Login::handle_post_with_redis_cli))
        .route("/search_similar_titles", get(search_similar_titles::SearchSimilarTitles::handle_get))
        .route("/search_similar_content", post(search_similar_content::SearchSimilarContent::handle_post))
        .route("/vectorize_titles", get(vectorize_titles::VectorizeTitles::handle_get))
        .layer(Extension(Arc::clone(&cfg)))
        .layer(Extension(Arc::clone(&redis_cli_arc)));
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", cfg.port)).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
