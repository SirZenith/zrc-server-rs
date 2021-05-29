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
pub async fn login(_: DBAccessManager) -> ZrcSVResult<impl warp::Reply> {
    // TODO: add authentication
    let result = LoginToken {
        access_token: "nothing".to_string(),
        token_type: "Bear".to_string(),
        success: true,
        error_code: 0,
    };
    respond_ok(result)
}

// GET /compose/aggregate?<calls>
pub async fn aggregate(call: AggregateCall, user_id: isize, conn: DBAccessManager) -> ZrcSVResult<impl warp::Reply> {
    // TODO: Error handling
    let endpoints: Vec<AggregateEndPoint> = serde_json::from_str(&call.calls).unwrap();
    let mut results = Vec::new();
    for call in endpoints {
        let content = match call.endpoint.as_str() {
            "/user/me" => {
                serde_json::to_string(&conn.get_user_info(user_id).unwrap()).unwrap()
            }
            "/purchase/bundle/pack" => {
                serde_json::to_string(&conn.get_pack_info().unwrap()).unwrap()
            }
            "/serve/download/me/song?url=false" => {
                serde_json::to_string(&conn.get_all_purchase_dl(user_id)).unwrap()
            }
            "/game/info" => serde_json::to_string(&conn.get_game_info().unwrap()).unwrap(),
            "/world/map/me" => {
                serde_json::to_string(&conn.get_map_info(user_id).unwrap()).unwrap()
            }
            _ => serde_json::to_string("[]").unwrap(),
        };
        results.push(format!(r#"{{"id":{},"value":{}}}"#, call.id, content));
    }
    Ok(warp::reply::with_status(
        format!(r#"{{"success": true,"value": [{}]}}"#, results.join(",")),
        warp::http::StatusCode::OK,
    ))
}

// GET /game/info
pub async fn game_info(conn: DBAccessManager) -> ZrcSVResult<impl warp::Reply> {
    match conn.get_game_info() {
        Ok(info) => respond_ok(info),
        Err(e) => Err(warp::reject::custom(ZrcSVError::DBError(e))),
    }
}

// GET /purchase/bundle/pack
pub async fn pack_info(conn: DBAccessManager) -> ZrcSVResult<impl warp::Reply> {
    match conn.get_pack_info() {
        Ok(info) => respond_ok(info),
        Err(e) => Err(warp::reject::custom(ZrcSVError::DBError(e)))
    }
}

// GET /present/me
pub async fn present_me(_: DBAccessManager) -> ZrcSVResult<impl warp::Reply> {
    respond_ok("[]")
}

// GET /user/me
pub async fn user_info(user_id: isize, conn: DBAccessManager) -> ZrcSVResult<impl warp::Reply> {
    match conn.get_user_info(user_id) {
        Ok(info) => respond_ok(info),
        Err(e) => Err(warp::reject::custom(ZrcSVError::DBError(e)))
    }
}

// GET /world/map/me
pub async fn world_map(user_id: isize, conn: DBAccessManager) -> ZrcSVResult<impl warp::Reply> {
    match conn.get_map_info(user_id) {
        Ok(info) => respond_ok(info),
        Err(e) => Err(warp::reject::custom(ZrcSVError::DBError(e)))
    }
}

// POST /user/me/setting/:option
pub async fn user_setting(
    option: String,
    setting: HashMap<String, String>,
    user_id: isize,
    conn: DBAccessManager,
) -> ZrcSVResult<impl warp::Reply> {
    let value = match setting.get("value") {
        Some(v) => v,
        // TODO: make proper rejection.
        None => return Err(warp::reject::not_found()),
    };
    if option == "favorite_character" {
        let char_id = value.parse::<isize>().unwrap();
        if let Err(e) = conn.set_favorite_character(user_id, char_id) {
            return Err(warp::reject::custom(ZrcSVError::DBError(e)));
        };
    } else {
        let value = value.parse::<bool>().unwrap();
        match conn.set_user_setting(user_id, option, value) {
            Err(e) => return Err(warp::reject::custom(ZrcSVError::DBError(e))),
            Ok(_) => {}
        };
    }
    let info = match conn.get_user_info(user_id) {
        Ok(info) => info,
        Err(e) => return Err(warp::reject::custom(ZrcSVError::DBError(e)))
    };
    let result = ResponseContainer {
        success: true,
        value: info,
        error_code: 0,
        error_msg: String::new(),
    };
    respond_ok(result)
}
