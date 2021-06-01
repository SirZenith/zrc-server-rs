use super::*;
use super::auth;

#[derive(Serialize)]
pub struct LoginToken {
    access_token: String,
    token_type: String,
    success: bool,
    #[serde(skip_serializing_if = "is_zero")]
    error_code: i32,
}

#[derive(Serialize)]
pub struct SignupResponse {
    user_id: isize,
    access_token: String,
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

// POST /user/
pub async fn signup(form: HashMap<String, String>, mut conn: DBAccessManager) -> ZrcSVResult<impl warp::Reply> {
    // name=abcd&password=00000000&email=a%40b.com&device_id=4C8C520B-28CF-422A-B773-47126BA5F800&platform=ios
    let name = get_from_form(&form, "name").map_err(
        |e| warp::reject::custom(e)
    )?;
    let password = get_from_form(&form, "password").map_err(
        |e| warp::reject::custom(e)
    )?;
    let email = get_from_form(&form, "email").map_err(
        |e| warp::reject::custom(e)
    )?;
    let device_id = get_from_form(&form, "device_id").map_err(
        |e| warp::reject::custom(e)
    )?;
    let _platform = get_from_form(&form, "platform").map_err(
        |e| warp::reject::custom(e)
    )?;

    let pwd_hash = auth::hash_pwd(password);
    let user_id = conn.signup(name, &pwd_hash, email, device_id).map_err(|e| {
        let err = match e {
            ZrcDBError::UserNameExists => ZrcSVError::UserNameExists,
            ZrcDBError::EmailExists => ZrcSVError::EmailExists,
            _ => ZrcSVError::DBError(e)
        };
        warp::reject::custom(err)
    })?;
    let access_token = auth::create_jwt(user_id).map_err(|e| warp::reject::custom(e))?;

    respond_ok(ResponseContainer {
        success: true,
        value: SignupResponse {
            user_id,
            access_token,
        },
        error_code: 0,
        error_msg: String::new()
    })
}

// GET /auth/login
pub async fn login(jwt: String) -> ZrcSVResult<impl warp::Reply> {
    let result = LoginToken {
        access_token: jwt,
        token_type: auth::BEARER.trim().to_string(),
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
                serde_json::to_string(
                    &conn.get_user_info(user_id)
                        .map_err(|e| warp::reject::custom(ZrcSVError::DBError(e)))?
                ).unwrap()
            }
            "/purchase/bundle/pack" => {
                serde_json::to_string(
                    &conn.get_pack_info()
                        .map_err(|e| warp::reject::custom(ZrcSVError::DBError(e)))?
                ).unwrap()
            }
            "/serve/download/me/song?url=false" => {
                serde_json::to_string(
                    &conn.get_all_purchase_dl(user_id)
                        .map_err(|e| warp::reject::custom(ZrcSVError::DBError(e)))?
                ).unwrap()
            }
            "/game/info" => serde_json::to_string(
                &conn.get_game_info()
                    .map_err(|e| warp::reject::custom(ZrcSVError::DBError(e)))?
            ).unwrap(),
            "/world/map/me" => {
                serde_json::to_string(
                    &conn.get_map_info(user_id)
                        .map_err(|e| warp::reject::custom(ZrcSVError::DBError(e)))?
                ).unwrap()
            }
            _ => serde_json::to_string(&Vec::<()>::new()).unwrap(),
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

// GET /purchase/bundle/single
pub async fn single_info(conn: DBAccessManager) -> ZrcSVResult<impl warp::Reply> {
    match conn.get_single_info() {
        Ok(result) => respond_ok(ResponseContainer {
            success: true,
            value: result,
            error_code: 0,
            error_msg: String::new(),
        }),
        Err(e) => Err(warp::reject::custom(ZrcSVError::DBError(e)))
    }
}

// GET /present/me
pub async fn present_me(_: DBAccessManager) -> ZrcSVResult<impl warp::Reply> {
    respond_ok(Vec::<()>::new())
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
    let value = get_from_form(&setting, "value").map_err(
        |e| warp::reject::custom(e)
    )?;
    if option == "favorite_character" {
        // TODO: remove this unwrap
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
