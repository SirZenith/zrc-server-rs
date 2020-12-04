use super::*;

pub mod character;
pub mod download;
pub mod info;
pub mod score;

fn respond<T: Serialize>(
    result: T,
    status: warp::http::StatusCode,
) -> Result<impl warp::Reply, warp::Rejection> {
    Ok(warp::reply::with_status(warp::reply::json(&result), status))
}

pub fn api_filter(
    pool: SqlitePool,
    hostname: String,
    api_prefix: String,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path(api_prefix).and(
        warp::path("welcome")
            .map(|| "Welcome to Zrcaea Server")
            .or(login(pool.clone()))
            .or(aggregate(pool.clone()))
            .or(get_download_list(pool.clone(), hostname.clone()))
            .or(change_character(pool.clone()))
            .or(toggle_uncap(pool.clone()))
            .or(score_token(pool.clone()))
            .or(score_upload(pool.clone())),
    )
}

// GET /auth/login
fn login(
    pool: SqlitePool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("auth" / "login")
        .and(with_db_access_manager(pool))
        .and_then(info::login)
}

// GET /compose/aggregate?<calls>
fn aggregate(
    pool: SqlitePool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("compose" / "aggregate")
        .and(warp::get())
        .and(warp::query())
        .and(with_db_access_manager(pool))
        .and_then(info::aggregate)
}

// GET /serve/download/me/song?url&sid
fn get_download_list(
    pool: SqlitePool,
    hostname: String,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("serve" / "download" / "me" / "song")
        .and(warp::get())
        .map(move || hostname.clone())
        .and(warp::query::<download::DLRequest>())
        .and(with_db_access_manager(pool))
        .and_then(download::get_download_list)
}

// POST /user/me/characters
fn change_character(
    pool: SqlitePool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("user" / "me" / "characters")
        .and(warp::post())
        .and(warp::body::form())
        .and(with_db_access_manager(pool))
        .and_then(character::change_character)
}

// POST /user/me/characters/<part_id>/toggle_uncap
fn toggle_uncap(
    pool: SqlitePool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("user" / "me" / "characters" / isize / "toggle_uncap")
        .and(warp::post())
        .and(with_db_access_manager(pool))
        .and_then(character::toggle_uncap)
}

// GET score/token
fn score_token(
    pool: SqlitePool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!["score" / "token"]
    .and(warp::get())
    .and(with_db_access_manager(pool))
    .and_then(score::score_token)
}

// POST score/song
fn score_upload(
    pool: SqlitePool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!["score" / "song"]
    .and(warp::post())
    .and(warp::body::form())
    .and(with_db_access_manager(pool))
    .and_then(score::score_upload)
}
