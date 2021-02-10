use super::*;
use crate::data_access::LookupedScore;

use askama::Template;

// GET /score/token
pub async fn score_token(pool: DBAccessManager) -> Result<impl warp::Reply, warp::Rejection> {
    Ok(warp::reply::with_status(
        format!(
            r#"{{"success": true, "value": {{"token": "{}"}}}}"#,
            pool.gen_score_token()
        ),
        warp::http::StatusCode::OK,
    ))
}

// POST /score/song
pub async fn score_upload(
    score_record: data_access::ScoreRecord,
    mut conn: DBAccessManager,
) -> Result<impl warp::Reply, warp::Rejection> {
    let user_id = STATIC_USER_ID;
    respond(
        conn.score_upload(&score_record, user_id, None).unwrap(),
        warp::http::StatusCode::OK,
    )
}

#[derive(Template)]
#[template(path = "score_page.html")]
struct RecordsTemplate {
    r10: f64,
    b30: f64,
    records: Vec<LookupedScore>,
}

// GET /score/:user_id
pub async fn score_lookup(
    user_id: isize,
    conn: DBAccessManager,
) -> Result<impl warp::Reply, warp::Rejection> {
    match conn.score_lookup(user_id) {
        Err(_) => Ok(warp::reply::html("".to_string())),
        Ok(records) => {
            let (r10, b30) = conn.get_r10_and_b30(user_id).unwrap();
            let template = RecordsTemplate {
                r10,
                b30,
                records: records,
            };
            let res = template.render().unwrap();
            Ok(warp::reply::html(res))
        }
    }
}
