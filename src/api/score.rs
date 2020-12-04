use super::*;

// GET score/token
pub async fn score_token(pool: DBAccessManager) -> Result<impl warp::Reply, warp::Rejection> {
    Ok(warp::reply::with_status(
        format!(
            r#"{{"success": true, "value": {{"token": "{}"}}}}"#,
            pool.gen_score_token()
        ),
        warp::http::StatusCode::OK,
    ))
}

// POST score/song
pub async fn score_upload(
    score_record: data_access::score::ScoreRecord,
    mut conn: DBAccessManager,
) -> Result<impl warp::Reply, warp::Rejection> {
    let user_id = STATIC_USER_ID;
    respond(conn.score_upload(score_record, user_id).unwrap(), warp::http::StatusCode::OK)
}

