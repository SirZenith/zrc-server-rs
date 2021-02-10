use super::*;
use super::data_access::DLRequest;

mod character;
mod download;
mod info;
mod save;
mod score;

fn respond<T: Serialize>(
    result: T,
    status: warp::http::StatusCode,
) -> Result<impl warp::Reply, warp::Rejection> {
    Ok(warp::reply::with_status(warp::reply::json(&result), status))
}

pub fn api_filter(
    pool: SqlitePool,
    hostname: String,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("welcome")
        .map(|| "Welcome to Zrcaea Server")
        .or(login(pool.clone()))
        .or(aggregate(pool.clone()))
        .or(game_info(pool.clone()))
        .or(pack_info(pool.clone()))
        .or(present_me(pool.clone()))
        .or(user_info(pool.clone()))
        .or(world_map(pool.clone()))
        .or(get_download_list(pool.clone(), hostname.clone()))
        .or(change_character(pool.clone()))
        .or(toggle_uncap(pool.clone()))
        .or(score_token(pool.clone()))
        .or(score_upload(pool.clone()))
        .or(score_lookup(pool.clone()))
        .or(upload_backup_data(pool.clone()))
        .or(download_backup_data(pool.clone()))
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

// GET /game/info
fn game_info(
    pool: SqlitePool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("game" / "info")
        .and(warp::get())
        .and(with_db_access_manager(pool))
        .and_then(info::game_info)
}

// GET /purchase/bundle/pack
fn pack_info(
    pool: SqlitePool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("purchase" / "bundle" / "pack")
        .and(warp::get())
        .and(with_db_access_manager(pool))
        .and_then(info::pack_info)
}

// GET /present/me
fn present_me(
    pool: SqlitePool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("present" / "me")
        .and(warp::get())
        .and(with_db_access_manager(pool))
        .and_then(info::present_me)
}

// GET /user/info
fn user_info(
    pool: SqlitePool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("user" / "info")
        .and(warp::get())
        .and(with_db_access_manager(pool))
        .and_then(info::user_info)
}

// GET /world/map/me
fn world_map(
    pool: SqlitePool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("world" / "map" / "me")
        .and(warp::get())
        .and(with_db_access_manager(pool))
        .and_then(info::world_map)
}

// GET /serve/download/me/song?url&sid
fn get_download_list(
    pool: SqlitePool,
    hostname: String,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("serve" / "download" / "me" / "song")
        .and(warp::get())
        .map(move || hostname.clone())
        .and(warp::query::<DLRequest>())
        .and(with_db_access_manager(pool))
        .and_then(download::get_download_list)
}

// POST /user/me/characters
fn change_character(
    pool: SqlitePool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("user" / "me" / "character")
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

// GET /score/:user_id
fn score_lookup(
    pool: SqlitePool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!["score" / isize]
        .and(warp::get())
        .and(with_db_access_manager(pool))
        .and_then(score::score_lookup)
}

// POST /user/me/save
fn upload_backup_data(
    pool: SqlitePool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("user" / "me" / "save")
        .and(warp::post())
        .and(warp::body::form())
        .and(with_db_access_manager(pool))
        .and_then(save::upload_backup_data)
}

// GET /user/me/save
fn download_backup_data(
    pool: SqlitePool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("user" / "me" / "save")
        .and(warp::get())
        .and(with_db_access_manager(pool))
        .and_then(save::download_backup_data)
}
