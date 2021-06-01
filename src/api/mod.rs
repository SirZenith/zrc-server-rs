use crate::api::auth::with_basic_auth;

use super::data_access::DLRequest;
use super::*;

mod auth;
mod character;
mod dlc;
pub mod error;
mod friend;
mod info;
mod save;
mod score;

use auth::with_auth;
use error::ZrcSVError;

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

fn get_from_form<'a>(form: &'a HashMap<String, String>, key: &str) -> Result<&'a String, ZrcSVError> {
    match form.get(&key.to_string()) {
        Some(v) => Ok(v),
        None => return Err(ZrcSVError::IncompleteForm(key.to_string()))
    }
}

pub fn api_filter(
    pool: SqlitePool,
    hostname: String,
    document_root: std::path::PathBuf,
    prefix: String,
    prefix_static_file: String,
    songs_dirname: String,
    is_auth_off: bool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let welcome = warp::path("welcome").map(|| "Welcome to Zrcaea Server");
    let file_server = warp::path(prefix_static_file.clone())
        .and(with_auth(true))
        .and(warp::fs::dir(document_root))
        .map(|_, it| it);
    let signup_route = signup(pool.clone());
    let login_auth = login(is_auth_off, pool.clone());
    let get_info = game_info(pool.clone())
        .or(pack_info(pool.clone()))
        .or(single_info(pool.clone()))
        .or(present_me(pool.clone()))
        .or(score_lookup(pool.clone()));
    let game_play = aggregate(is_auth_off, pool.clone())
        .or(user_info(is_auth_off, pool.clone()))
        .or(world_map(is_auth_off, pool.clone()))
        .or(user_setting(is_auth_off, pool.clone()))
        .or(get_download_list(
            is_auth_off,
            pool.clone(),
            hostname.clone(),
            prefix_static_file.clone(),
            songs_dirname.clone(),
        ))
        .or(purchase_item(is_auth_off, pool.clone()))
        .or(change_character(is_auth_off, pool.clone()))
        .or(toggle_uncap(is_auth_off, pool.clone()))
        .or(score_token(is_auth_off, pool.clone()))
        .or(score_upload(is_auth_off, pool.clone()))
        .or(upload_backup_data(is_auth_off, pool.clone()))
        .or(download_backup_data(is_auth_off, pool.clone()))
        .or(add_friend(is_auth_off, pool.clone()))
        .or(delete_friend(is_auth_off, pool.clone()));

    let mut route = welcome
        .or(file_server)
        .or(signup_route)
        .or(login_auth)
        .or(get_info)
        .or(game_play)
        .boxed();
    if !prefix.is_empty() {
        route = warp::path(prefix).and(route).boxed();
    }
    route.recover(api::error::handle_rejection).boxed()
}

// ----------------------------------------------------------------------------
// info

// POST /user/
fn signup(pool: SqlitePool) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("user")
        .and(warp::post())
        .and(warp::body::form())
        .and(with_db_access_manager(pool))
        .and_then(info::signup)
}

// POST /auth/login
fn login(
    is_auth_off: bool,
    pool: SqlitePool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("auth" / "login")
        .and(warp::post())
        .and(with_basic_auth(is_auth_off, pool))
        .and_then(info::login)
}

// GET /compose/aggregate?<calls>
fn aggregate(
    is_auth_off: bool,
    pool: SqlitePool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("compose" / "aggregate")
        .and(warp::get())
        .and(warp::query())
        .and(with_auth(is_auth_off))
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

// GET /purchase/bundle/single
fn single_info(
    pool: SqlitePool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("purchase" / "bundle" / "single")
        .and(warp::get())
        .and(with_db_access_manager(pool))
        .and_then(info::single_info)
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
    is_auth_off: bool,
    pool: SqlitePool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("user" / "info")
        .and(warp::get())
        .and(with_auth(is_auth_off))
        .and(with_db_access_manager(pool))
        .and_then(info::user_info)
}

// GET /world/map/me
fn world_map(
    is_auth_off: bool,
    pool: SqlitePool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("world" / "map" / "me")
        .and(warp::get())
        .and(with_auth(is_auth_off))
        .and(with_db_access_manager(pool))
        .and_then(info::world_map)
}

// POST /user/me/setting/:option
fn user_setting(
    is_auth_off: bool,
    pool: SqlitePool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("user" / "me" / "setting" / String)
        .and(warp::post())
        .and(warp::body::form())
        .and(with_auth(is_auth_off))
        .and(with_db_access_manager(pool))
        .and_then(info::user_setting)
}

// ----------------------------------------------------------------------------
// dlc

// GET /serve/download/me/song?url&sid
fn get_download_list(
    is_auth_off: bool,
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
        .and(with_auth(is_auth_off))
        .and(with_db_access_manager(pool))
        .and_then(dlc::get_download_list)
}

// POST /purchase/me/pack
fn purchase_item(
    is_auth_off: bool,
    pool: SqlitePool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("purchase" / "me" / "pack")
        .and(warp::post())
        .and(warp::body::form())
        .and(with_auth(is_auth_off))
        .and(with_db_access_manager(pool))
        .and_then(dlc::purcahse_item)
}

// ----------------------------------------------------------------------------
// character

// POST /user/me/characters
fn change_character(
    is_auth_off: bool,
    pool: SqlitePool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("user" / "me" / "character")
        .and(warp::post())
        .and(warp::body::form())
        .and(with_auth(is_auth_off))
        .and(with_db_access_manager(pool))
        .and_then(character::change_character)
}

// POST /user/me/characters/<part_id>/toggle_uncap
fn toggle_uncap(
    is_auth_off: bool,
    pool: SqlitePool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("user" / "me" / "characters" / isize / "toggle_uncap")
        .and(warp::post())
        .and(with_auth(is_auth_off))
        .and(with_db_access_manager(pool))
        .and_then(character::toggle_uncap)
}

// ----------------------------------------------------------------------------
// score

// GET score/token
fn score_token(
    is_auth_off: bool,
    pool: SqlitePool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!["score" / "token"]
        .and(warp::get())
        .and(with_auth(is_auth_off))
        .and(with_db_access_manager(pool))
        .and_then(score::score_token)
}

// POST score/song
fn score_upload(
    is_auth_off: bool,
    pool: SqlitePool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!["score" / "song"]
        .and(warp::post())
        .and(warp::body::form())
        .and(with_auth(is_auth_off))
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

// ----------------------------------------------------------------------------
// data backup

// POST /user/me/save
fn upload_backup_data(
    is_auth_off: bool,
    pool: SqlitePool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("user" / "me" / "save")
        .and(warp::post())
        .and(warp::body::form())
        .and(with_auth(is_auth_off))
        .and(with_db_access_manager(pool))
        .and_then(save::upload_backup_data)
}

// GET /user/me/save
fn download_backup_data(
    is_auth_off: bool,
    pool: SqlitePool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("user" / "me" / "save")
        .and(warp::get())
        .and(with_auth(is_auth_off))
        .and(with_db_access_manager(pool))
        .and_then(save::download_backup_data)
}

// POST /friend/me/add
fn add_friend(
    is_auth_off: bool,
    pool: SqlitePool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("friend" / "me" / "add")
        .and(warp::post())
        .and(warp::body::form())
        .and(with_auth(is_auth_off))
        .and(with_db_access_manager(pool))
        .and_then(friend::add_friend)
}

// POST /friend/me/delete
fn delete_friend(
    is_auth_off: bool,
    pool: SqlitePool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("friend" / "me" / "delete")
        .and(warp::post())
        .and(warp::body::form())
        .and(with_auth(is_auth_off))
        .and(with_db_access_manager(pool))
        .and_then(friend::delete_friend)
}

