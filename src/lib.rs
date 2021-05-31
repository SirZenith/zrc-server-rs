pub mod api;
pub mod data_access;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use data_access::*;
use structopt::clap::arg_enum;
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

arg_enum! {
    #[derive(Debug)]
    enum LogLevel {
        Off,
        Error,
        Warn,
        Info,
        Debug,
        Trace,
    }
}

impl LogLevel {
    fn to_level_filter(&self) -> log::LevelFilter {
        match self {
            LogLevel::Off => log::LevelFilter::Off,
            LogLevel::Error => log::LevelFilter::Error,
            LogLevel::Warn => log::LevelFilter::Warn,
            LogLevel::Info => log::LevelFilter::Info,
            LogLevel::Debug => log::LevelFilter::Debug,
            LogLevel::Trace => log::LevelFilter::Trace,
        }
    }
}

#[derive(StructOpt)]
pub struct Cli {
    #[structopt(short, long, default_value = "127.0.0.1", help = "IP address of server instance.")]
    ip: String,

    #[structopt(short, long, default_value = "localhost", help = "Hostname of server instance.")]
    hostname: String,

    #[structopt(short, long, default_value = "8080", help = "Port number used by server.")]
    port: u16,

    #[structopt(short, long = "db", default_value = "./ZrcDB.db", help = "Path of data base file.")]
    db_path: String,

    #[structopt(short = "r", long = "root", default_value = "./", help = "Root directory of static files.")]
    document_root: String,
    // final access URL will be http://<hostname>/<prefix-all>/<your-api>
    #[structopt(long = "prefix-all", default_value = "", help = "Prefix for all API.")]
    prefix_all: String,

    // final access URL will be http://<hostname>/<prefix-all>/<prefix-static>/<your-filename>
    #[structopt(long = "prefix-static", default_value = "static", help = "Path prefix for static files.")]
    prefix_static_file: String,

    
    #[structopt(long = "songs-dirname", default_value = "songs", help = "Name of songs directory under document root.")]
    songs_dirname: String,

    #[structopt(long = "no-auth", help = "Whether to turn off authentication")]
    is_auth_off: bool,

    #[structopt(long = "log-level", possible_values = &LogLevel::variants(), case_insensitive = true, default_value = "info")]
    log_level: LogLevel
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_zero<T: Into<f64> + Copy>(num: &T) -> bool {
    num.clone().into() == 0.
}

pub async fn start_serving(argv: Vec<String>) {
    let cli = Cli::from_iter(argv.iter());

    SimpleLogger::new()
        .with_level(cli.log_level.to_level_filter())
        .init()
        .unwrap();

    let db_path = match Path::new(&cli.db_path).canonicalize() {
        Ok(p) => p,
        Err(e) => {
            log::error!("{}, {}", cli.db_path, e);
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
            log::error!("{}, {}", cli.document_root, e);
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
        cli.is_auth_off,
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
