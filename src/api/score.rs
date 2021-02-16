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
    user_name: String,
    user_code: String,
    rating_integer: isize,
    rating_fraction: isize,
    rating_level: i8,
    favourite_character: i8,
    is_uncapped: bool,
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
            let user_info = conn.get_minimum_user_info(user_id).unwrap();
            let rating_level = user_info.get_rating_level();
            let template = RecordsTemplate {
                user_name: user_info.name,
                user_code: user_info.user_code,
                rating_integer: user_info.rating / 100,
                rating_fraction: user_info.rating % 100,
                rating_level,
                favourite_character: user_info.favorite_character,
                is_uncapped: user_info.is_uncapped && !user_info.is_uncapped_override,
                r10,
                b30,
                records: records,
            };
            let res = template.render().unwrap();
            Ok(warp::reply::html(res))
        }
    }
}
