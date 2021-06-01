use super::*;

#[derive(Serialize)]
struct InfoWithFriendList {
    user_id: isize,
    #[serde(rename = "updateAt")]
    update_at: String,
    #[serde(rename = "createdAt")]
    create_at: String,
    friends: Vec<UserInfoMinimum>
}

// POST /friend/me/add
pub async fn add_friend(
    form: HashMap<String, String>,
    user_id: isize,
    mut conn: DBAccessManager
) -> ZrcSVResult<impl warp::Reply> {
    let friend_code = get_from_form(&form, "friend_code")
        .map_err(|e| warp::reject::custom(e))?;
    let friend_code = friend_code.parse::<isize>().map_err(|_| {
        warp::reject::custom(ZrcSVError::ImproperFormValue("friend_code".to_string(), friend_code.clone()))
    })?;
    conn.add_friend(user_id, friend_code).map_err(|e| {
        match e {
            ZrcDBError::DataNotFound(_) => warp::reject::custom(ZrcSVError::InvalidFriendCode),
            _ => warp::reject::custom(ZrcSVError::DBError(e)),
        }
    })?;
    let friends = conn.get_friend_list(user_id).map_err(|e| {
        warp::reject::custom(ZrcSVError::DBError(e))
    })?;
    respond_ok(ResponseContainer {
        success: true,
        value: InfoWithFriendList {
            user_id,
            create_at: String::new(),
            update_at: String::new(),
            friends,
        },
        error_code: 0,
        error_msg: String::new()
    })
}

// POST /friend/me/delete
pub async fn delete_friend(
    form: HashMap<String, String>,
    user_id: isize,
    mut conn: DBAccessManager
) -> ZrcSVResult<impl warp::Reply> {
    let friend_id = get_from_form(&form, "friend_id")
        .map_err(|e| warp::reject::custom(e))?;
    let friend_id = friend_id.parse::<isize>().map_err(|_| {
        warp::reject::custom(ZrcSVError::ImproperFormValue("friend_id".to_string(), friend_id.clone()))
    })?;
    conn.delete_friend(user_id, friend_id).map_err(|e| {
        warp::reject::custom(ZrcSVError::DBError(e))
    })?;
    let friends = conn.get_friend_list(user_id).map_err(|e| {
        warp::reject::custom(ZrcSVError::DBError(e))
    })?;
    respond_ok(ResponseContainer {
        success: true,
        value: InfoWithFriendList {
            user_id,
            create_at: String::new(),
            update_at: String::new(),
            friends,
        },
        error_code: 0,
        error_msg: String::new()
    })
}
