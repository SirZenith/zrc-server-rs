use super::data_access::DLRequest;
use super::*;

mod auth;
mod character;
mod download;
pub mod error;
mod info;
mod save;
mod score;

use error::ZrcSVError;
use auth::with_auth;

type ZrcSVResult<T> = std::result::Result<T, warp::Rejection>;

#[derive(Serialize)]
pub struct ResponseContainer<T: Serialize> {
    success: bool,
    value: T,
    #[serde(skip_serializing_if = "is_zero")]
    error_code: i32,
    #[serde(skip_serializing_if = "String::is_empty")]
    error_msg: String,
}

fn respond_ok<T: Serialize>(result: T) -> ZrcSVResult<impl warp::Reply> {
    Ok(warp::reply::with_status(
        warp::reply::json(&result),
        warp::http::StatusCode::OK,
    ))
}

pub fn api_filter(
    pool: SqlitePool,
    hostname: String,
    document_root: std::path::PathBuf,
    prefix: String,
    prefix_static_file: String,
    songs_dirname: String,
    no_auth: bool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let welcome = warp::path("welcome")
        .map(|| "Welcome to Zrcaea Server");
    let file_server = warp::path(prefix_static_file.clone())
        .and(warp::fs::dir(document_root));
    let login_auth = login(pool.clone());
    let get_info = game_info(pool.clone())
        .or(pack_info(pool.clone()))
        .or(present_me(pool.clone()))
        .or(score_lookup(pool.clone()));
    let game_play = aggregate(no_auth, pool.clone())
        .or(user_info(no_auth, pool.clone()))
        .or(world_map(no_auth, pool.clone()))
        .or(user_setting(no_auth, pool.clone()))
        .or(get_download_list(
            no_auth, 
            pool.clone(),
            hostname.clone(),
            prefix_static_file.clone(),
            songs_dirname.clone(),
        ))
        .or(change_character(no_auth, pool.clone()))
        .or(toggle_uncap(no_auth, pool.clone()))
        .or(score_token(no_auth, pool.clone()))
        .or(score_upload(no_auth, pool.clone()))
        .or(upload_backup_data(no_auth, pool.clone()))
        .or(download_backup_data(no_auth, pool.clone()));
    

    let mut route = welcome.or(file_server).or(login_auth).or(get_info).or(game_play).boxed();
    if !prefix.is_empty() {
        route = warp::path(prefix).and(route).boxed();
    }
    route.recover(api::error::handle_rejection).boxed()
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
    no_auth: bool,
    pool: SqlitePool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("compose" / "aggregate")
        .and(warp::get())
        .and(warp::query())
        .and(with_auth(no_auth))
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
    no_auth: bool,
    pool: SqlitePool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("user" / "info")
        .and(warp::get())
        .and(with_auth(no_auth))
        .and(with_db_access_manager(pool))
        .and_then(info::user_info)
}

// GET /world/map/me
fn world_map(
    no_auth: bool,
    pool: SqlitePool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("world" / "map" / "me")
        .and(warp::get())
        .and(with_auth(no_auth))
        .and(with_db_access_manager(pool))
        .and_then(info::world_map)
}

// POST /user/me/setting/:option
fn user_setting(
    no_auth: bool,
    pool: SqlitePool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("user" / "me" / "setting" / String)
        .and(warp::post())
        .and(warp::body::form())
        .and(with_auth(no_auth))
        .and(with_db_access_manager(pool))
        .and_then(info::user_setting)
}

// GET /serve/download/me/song?url&sid
fn get_download_list(
    no_auth: bool,
    pool: SqlitePool,
    hostname: String,
    prefix_static_file: String,
    songs_dirname: String,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("serve" / "download" / "me" / "song")
        .and(warp::get())
        .map(move || {
            (
                hostname.clone(),
                prefix_static_file.clone(),
                songs_dirname.clone(),
            )
        })
        .and(warp::query::<DLRequest>())
        .and(with_auth(no_auth))
        .and(with_db_access_manager(pool))
        .and_then(download::get_download_list)
}

// POST /user/me/characters
fn change_character(
    no_auth: bool,
    pool: SqlitePool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("user" / "me" / "character")
        .and(warp::post())
        .and(warp::body::form())
        .and(with_auth(no_auth))
        .and(with_db_access_manager(pool))
        .and_then(character::change_character)
}

// POST /user/me/characters/<part_id>/toggle_uncap
fn toggle_uncap(
    no_auth: bool,
    pool: SqlitePool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("user" / "me" / "characters" / isize / "toggle_uncap")
        .and(warp::post())
        .and(with_auth(no_auth))
        .and(with_db_access_manager(pool))
        .and_then(character::toggle_uncap)
}

// GET score/token
fn score_token(
    no_auth: bool,
    pool: SqlitePool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!["score" / "token"]
        .and(warp::get())
        .and(with_auth(no_auth))
        .and(with_db_access_manager(pool))
        .and_then(score::score_token)
}

// POST score/song
fn score_upload(
    no_auth: bool,
    pool: SqlitePool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!["score" / "song"]
        .and(warp::post())
        .and(warp::body::form())
        .and(with_auth(no_auth))
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
    no_auth: bool,
    pool: SqlitePool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("user" / "me" / "save")
        .and(warp::post())
        .and(warp::body::form())
        .and(with_auth(no_auth))
        .and(with_db_access_manager(pool))
        .and_then(save::upload_backup_data)
}

// GET /user/me/save
fn download_backup_data(
    no_auth: bool,
    pool: SqlitePool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("user" / "me" / "save")
        .and(warp::get())
        .and(with_auth(no_auth))
        .and(with_db_access_manager(pool))
        .and_then(save::download_backup_data)
}
