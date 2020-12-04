extern crate serde_json;

use super::*;

#[derive(Serialize)]
pub struct LoginToken {
    access_token: String,
    token_type: String,
    success: bool,
    #[serde(skip_serializing_if = "is_zero")]
    error_code: i32,
}

#[derive(Deserialize)]
pub struct AggregateEndPoint {
    endpoint: String,
    id: usize,
}

#[derive(Deserialize)]
pub struct AggregateCall {
    calls: String,
}

// GET /auth/login
pub async fn login(_: DBAccessManager) -> Result<impl warp::Reply, warp::Rejection> {
    let result = LoginToken {
        access_token: String::new(),
        token_type: String::new(),
        success: true,
        error_code: 0,
    };
    respond(result, warp::http::StatusCode::OK)
}

// GET /compose/aggregate?<calls>
pub async fn aggregate(call: AggregateCall, conn: DBAccessManager) -> Result<impl warp::Reply, warp::Rejection> {
    let endpoints: Vec<AggregateEndPoint> = serde_json::from_str(&call.calls).unwrap();
    let mut results = Vec::new();
    for call in endpoints {
        let content = match call.endpoint.as_str() {
            "/user/me" => {
                serde_json::to_string(&conn.get_user_info(STATIC_USER_ID).unwrap())
                    .unwrap()
            }
            "/purchase/bundle/pack" => {
                serde_json::to_string(&conn.get_pack_info().unwrap()).unwrap()
            }
            "/serve/download/me/song?url=false" => {
                serde_json::to_string(&conn.get_all_purchase_dl(STATIC_USER_ID)).unwrap()
            }
            "/game/info" => serde_json::to_string(&conn.get_game_info().unwrap()).unwrap(),
            "/world/map/me" => {
                serde_json::to_string(&conn.get_map_info(STATIC_USER_ID).unwrap()).unwrap()
            }
            _ => "[]".to_string(),
        };
        results.push(format!(r#"{{"id":{},"value":{}}}"#, call.id, content));
    }
    Ok(warp::reply::with_status(format!(r#"{{"success": true,"value": [{}]}}"#, results.join(",")), warp::http::StatusCode::OK))
}

// GET /game/info
pub async fn game_info(conn: DBAccessManager) -> Result<impl warp::Reply, warp::Rejection> {
    respond(conn.get_game_info().unwrap(), warp::http::StatusCode::OK)
}

// GET /purchase/bundle/pack
pub async fn pack_info(conn: DBAccessManager) -> Result<impl warp::Reply, warp::Rejection> {
    respond(conn.get_pack_info().unwrap(), warp::http::StatusCode::OK)
}

// GET /present/me
pub async fn present_me(_: DBAccessManager) -> Result<impl warp::Reply, warp::Rejection> {
    respond("[]", warp::http::StatusCode::OK)
}

// GET /user/me
pub async fn user_info(conn: DBAccessManager) -> Result<impl warp::Reply, warp::Rejection> {
    respond(
        conn.get_user_info(STATIC_USER_ID).unwrap(),
        warp::http::StatusCode::OK,
    )
}

// GET /world/map/me
pub async fn world_map(conn: DBAccessManager) -> Result<impl warp::Reply, warp::Rejection> {
    respond(conn.get_map_info(STATIC_USER_ID).unwrap(), warp::http::StatusCode::OK)
}
