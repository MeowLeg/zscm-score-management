use super::*;
use axum::{
    extract::{
        Request,
        State,
    },
    http::StatusCode,
    middleware::Next,
    response::Response,
};

pub async fn auth_account(State(client): State<Arc<Client>>, req: Request, next: Next) -> Result<Response, StatusCode> {
    // 在header里有两个参数分别是account和sessionid
    // account用于到redsi中获取值，然后与sessionid比对是否一致
    // 不然则返回401未授权
    let auth_header = req.headers();
    let header_account = auth_header.get(
        "account"
    )
    .and_then(|header| header.to_str().ok());
    let header_session_id = auth_header.get(
        "sessionid"
    ).and_then(|header| header.to_str().ok());

    if let Some(account_name) = header_account {
        if let Some(session_id) = header_session_id {
            // let mut con = client.get_async_connection().await.unwrap();
            let mut con = client.get_multiplexed_async_connection().await.map_err(|_| StatusCode::UNAUTHORIZED)?;
            let session_id_r: String = con.get(format!("zscm_{}", account_name)).await.unwrap_or("".into());
            if !session_id_r.is_empty() && session_id == session_id_r {
                return Ok(next.run(req).await)
            }
        }
    }
    Err(StatusCode::UNAUTHORIZED)
}
