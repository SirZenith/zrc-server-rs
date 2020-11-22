#![feature(proc_macro_hygiene, decl_macro)]
use std::collections::HashMap;
use std::env;
use std::path::Path;

extern crate libc;
use libc::{c_char, c_int, size_t};
use std::ffi::CStr;
use std::slice::from_raw_parts;
use std::str::Utf8Error;

#[macro_use]
extern crate rocket;
use rocket::config::{Config, Environment, Value};
use rocket::config::LoggingLevel;
use rocket::request::{Form, LenientForm};
use rocket::request::{FormItems, FromForm};

extern crate rocket_contrib;
use rocket_contrib::database;
use rocket_contrib::json::Json;

extern crate rusqlite;

extern crate strfmt;

extern crate serde;
use serde::{Deserialize, Serialize};

extern crate structopt;
use structopt::StructOpt;

#[database("zrc_db")]
pub struct ZrcDB(rusqlite::Connection);

#[derive(Serialize)]
pub struct ResponseContainer<T: Serialize> {
    success: bool,
    value: T,
    #[serde(skip_serializing_if = "is_zero")]
    error_code: i32,
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_zero<T: Into<f64> + Copy>(num: &T) -> bool {
    num.clone().into() == 0.
}

const STATIC_USER_ID: isize = 1;
const FILE_SERVER_PREFIX: &str = "/file";
const SONG_FILE_DIR: &str = "static/songs";

pub mod character;
pub mod download;
pub mod info;
pub mod score;
pub mod sql_stmt;
pub mod world;

pub unsafe fn convert_double_pointer_to_vec(
    data: &mut &mut c_char,
    len: size_t,
) -> Result<Vec<String>, Utf8Error> {
    from_raw_parts(data, len)
        .iter()
        .map(|arg| CStr::from_ptr(*arg).to_str().map(ToString::to_string))
        .collect()
}

#[no_mangle]
pub unsafe extern "C" fn rust_main(
    argc: c_int,
    data: &mut &mut c_char,
    _envp: &mut &mut c_char,
) -> c_int {
    let argv = convert_double_pointer_to_vec(data, argc as size_t);

    if let Ok(argv) = argv {
        start_serving(argv);
        0
    } else {
        1
    }
}

#[derive(StructOpt)]
pub struct Cli {
    // Address of server instance.
    #[structopt(short, long, default_value = "127.0.0.1")]
    address: String,

    // Port number used by server.
    #[structopt(short, long, default_value = "8080")]
    port: u16,
    // Path of data base file.
    #[structopt(short, long = "db", default_value = "./ZrcaeaDB.db")]
    db_path: String,

    // Root directory of static files.
    #[structopt(short = "r", long = "root", default_value = "./")]
    document_root: String,

    // Prefix for API
    #[structopt(long, default_value = "/glad-you-came")]
    prefix: String,
}

pub fn start_serving(argv: Vec<String>) {
    let cli = Cli::from_iter(argv.iter());
    let mut database_config = HashMap::new();
    let mut databases = HashMap::new();

    database_config.insert("url", Value::from(cli.db_path));
    databases.insert("zrc_db", Value::from(database_config));

    let hostname = format!("{}:{}", cli.address, cli.port);

    let document_root = Path::new(&cli.document_root).canonicalize().unwrap();
    env::set_current_dir(document_root).unwrap();

    let config = Config::build(Environment::Development)
        .address(cli.address)
        .port(cli.port)
        .log_level(LoggingLevel::Normal)
        .extra("databases", databases)
        .finalize()
        .unwrap();

    rocket::custom(config)
        .manage(hostname)
        .attach(crate::ZrcDB::fairing())
        .mount(
            crate::FILE_SERVER_PREFIX,
            rocket_contrib::serve::StaticFiles::from(std::env::current_dir().unwrap()),
        )
        .mount(
            &format!("{}{}", cli.prefix, "/"),
            routes![
                crate::info::login,
                crate::info::aggregate,
                crate::info::game_info,
                crate::info::pack_info,
                crate::info::user_info,
                crate::download::get_download_list
            ],
        )
        .mount(
            &format!("{}{}", cli.prefix, "/user/me/character"),
            routes![
                crate::character::change_character,
            ],
        )
        .mount(
            &format!("{}{}", cli.prefix, "/user/me/characters"),
            routes![crate::character::toggle_uncap],
        )
        .mount(
            &format!("{}{}", cli.prefix, "/score"),
            routes![crate::score::token, crate::score::score_upload],
        )
        .launch();
}
