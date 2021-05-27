pub mod api;
pub mod data_access;

use std::collections::HashMap;
// use std::env;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use data_access::*;
use lazy_static::lazy_static;
use log;
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use simple_logger::SimpleLogger;
use structopt::StructOpt;
use warp::Filter;

const STATIC_USER_ID: isize = 1;

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
    SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .unwrap();

    let cli = Cli::from_iter(argv.iter());

    // TODO: Add existance check.
    let db_path = match Path::new(&cli.db_path).canonicalize() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };
    let sqlite_connection_manager = SqliteConnectionManager::file(&db_path);
    let sqlite_pool = r2d2::Pool::new(sqlite_connection_manager)
        .expect("Failed to create r2d2 SQLite connection pool");
    let pool_arc = Arc::new(sqlite_pool);
    log::info!("Connected to database: {}", cli.db_path);

    let document_root = match Path::new(&cli.document_root).canonicalize() {
        Ok(p) => p,
        Err(e) => {
            log::error!("{}", e);
            return;
        }
    };
    log::info!("Document root path: {}", cli.document_root);

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
            log::error!(
                "invalid hostname, IP: {}, Port: {}\n\t{}",
                cli.ip,
                cli.port,
                e
            );
            return;
        }
    };
    warp::serve(routes).run(socket_addr).await;
}
