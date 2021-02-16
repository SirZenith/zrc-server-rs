extern crate askama;
extern crate libc;
#[macro_use]
extern crate lazy_static;
extern crate r2d2;
extern crate r2d2_sqlite;
#[macro_use]
extern crate rusqlite;
extern crate serde;
extern crate strfmt;
extern crate structopt;
extern crate warp;

pub mod api;
pub mod data_access;

use std::collections::HashMap;
// use std::env;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use libc::{c_char, c_int, size_t};
use std::ffi::CStr;
use std::slice::from_raw_parts;
use std::str::Utf8Error;

use r2d2::{Pool, PooledConnection};

use r2d2_sqlite::SqliteConnectionManager;

use serde::{Deserialize, Serialize};

use structopt::StructOpt;

use warp::Filter;

use data_access::*;

const STATIC_USER_ID: isize = 1;

#[derive(Serialize)]
pub struct ResponseContainer<T: Serialize> {
    success: bool,
    value: T,
    #[serde(skip_serializing_if = "is_zero")]
    error_code: i32,
    // #[serde(skip_serializing_if = "String::is_empty")]
    // error_msg: String,
}

#[derive(StructOpt)]
pub struct Cli {
    // IP address of server instance.
    #[structopt(short, long, default_value = "127.0.0.1")]
    ip: String,

    // Hostname of server instance.
    #[structopt(short, long, default_value = "localhost")]
    hostname: String,
    // Port number used by server.
    #[structopt(short, long, default_value = "8080")]
    port: u16,
    // Path of data base file.
    #[structopt(short, long = "db", default_value = "./ZrcDB.db")]
    db_path: String,

    // Root directory of static files.
    #[structopt(short = "r", long = "root", default_value = "./")]
    document_root: String,
    // Prefix for all API, final access URL will be http://<hostname>/<prefix-all>/<your-api>
    #[structopt(long = "prefix-all", default_value = "")]
    prefix_all: String,

    // Path prefix for static files, final access URL will be http://<hostname>/<prefix-all>/<prefix-static>/<your-filename>
    #[structopt(long = "prefix-static", default_value = "static")]
    prefix_static_file: String,

    // Name of songs directory under document root
    #[structopt(long = "songs-dirname", default_value = "songs")]
    songs_dirname: String,
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_zero<T: Into<f64> + Copy>(num: &T) -> bool {
    num.clone().into() == 0.
}

pub async fn start_serving(argv: Vec<String>) {
    let cli = Cli::from_iter(argv.iter());

    // TODO: Add existance check.
    let db_path = Path::new(&cli.db_path).canonicalize().unwrap();
    let sqlite_connection_manager = SqliteConnectionManager::file(db_path);
    let sqlite_pool = r2d2::Pool::new(sqlite_connection_manager)
        .expect("Failed to create r2d2 SQLite connection pool");
    let pool_arc = Arc::new(sqlite_pool);

    let document_root = Path::new(&cli.document_root).canonicalize().unwrap();
    println!(
        "Setting document root path at: {:?}",
        document_root.as_os_str()
    );

    let routes = api::api_filter(
        pool_arc,
        cli.hostname,
        document_root,
        cli.prefix_all,
        cli.prefix_static_file,
        cli.songs_dirname,
    );

    let socket_addr = match format!("{}:{}", cli.ip, cli.port).parse::<SocketAddr>() {
        Ok(s) => s,
        Err(e) => {
            println!(
                "Wrong hostname with IP: {}, Port: {}\n\t{}",
                cli.ip, cli.port, e
            );
            return;
        }
    };
    warp::serve(routes).run(socket_addr).await;
}

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
pub async unsafe extern "C" fn rust_main(
    argc: c_int,
    data: &mut &mut c_char,
    _envp: &mut &mut c_char,
) -> c_int {
    let argv = convert_double_pointer_to_vec(data, argc as size_t);

    if let Ok(argv) = argv {
        start_serving(argv).await;
        0
    } else {
        1
    }
}
