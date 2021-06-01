use super::*;
use crate::data_access::LookupedScore;

use askama::Template;

// GET /score/token
pub async fn score_token(user_id: isize, conn: DBAccessManager) -> ZrcSVResult<impl warp::Reply> {
    let token = conn.gen_score_token(user_id)
        .map_err(|e| warp::reject::custom(ZrcSVError::DBError(e)))?;
    respond_ok(ResponseContainer {
        success: true,
        value: token,
        error_code: 0,
        error_msg: String::new(),
    })
}

// POST /score/song
pub async fn score_upload(
    score_record: data_access::ScoreRecord,
    user_id: isize,
    mut conn: DBAccessManager,
) -> ZrcSVResult<impl warp::Reply> {
    let result = conn.score_upload(&score_record, user_id, None).unwrap();
    respond_ok(ResponseContainer {
        success: true,
        value: result,
        error_code: 0,
        error_msg: String::new(),
    })
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
pub async fn score_lookup(user_id: isize, conn: DBAccessManager) -> ZrcSVResult<impl warp::Reply> {
    match conn.score_lookup(user_id) {
        Err(e) => Err(warp::reject::custom(ZrcSVError::DBError(e))),
        Ok(records) => {
            let (r10, b30) = conn
                .get_r10_and_b30(user_id)
                .map_err(|e| warp::reject::custom(ZrcSVError::DBError(e)))?;
            let user_info = conn.get_minimum_user_info(user_id)
                .map_err(|e| warp::reject::custom(ZrcSVError::DBError(e)))?;
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
            let res = template.render().map_err(|e| warp::reject::custom(ZrcSVError::TemplateError(e)))?;
            Ok(warp::reply::html(res))
        }
    }
}
