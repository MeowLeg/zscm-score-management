use super::*;
use tokio::process::Command;

pub struct VectorizeTitles;

#[derive(Debug, Deserialize)]
pub struct VectorizeTitlesReq {
    pub force: Option<bool>,
    pub rebuild_vocab: Option<bool>,
}

impl ExecSql<VectorizeTitlesReq> for VectorizeTitles {
    async fn handle_get(
        _cfg: Extension<Arc<Config>>,
        prms: Option<Query<VectorizeTitlesReq>>,
    ) -> Result<Json<Value>, WebErr> {
        let prms = prms.unwrap_or_else(|| Query(VectorizeTitlesReq {
            force: None,
            rebuild_vocab: None,
        }));

        let mut args = vec![];
        
        if prms.force.unwrap_or(false) {
            args.push("--force");
        }
        
        if prms.rebuild_vocab.unwrap_or(false) {
            args.push("--rebuild-vocab");
        }

        let output = Command::new("python3")
            .arg("tool/vectorize_zscm_titles.py")
            .args(&args)
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        Ok(Json(json!({
            "success": output.status.success(),
            "stdout": stdout,
            "stderr": stderr,
            "exit_code": output.status.code()
        })))
    }
}
