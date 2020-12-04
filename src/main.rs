extern crate zrc_server;

use std::env;

#[tokio::main]
async fn main() {
    let argv: Vec<String> = env::args().collect();
    zrc_server::start_serving(argv).await;
}
