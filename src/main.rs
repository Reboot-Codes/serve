use log::{LevelFilter, info, warn};
use warp::{Filter, http::Response, path::FullPath};
use clap::{Parser, crate_name};
use std::{fs, path::PathBuf};

// This will parse command line input, and return a tuple.
fn input() -> (u16, String, bool) {
    #[derive(Parser, Debug)]
    #[clap(author, version, about)]
    struct Args {
        /// HTTP port to serve files from.
        #[clap(short = 'p', long = "port", default_value = "1234")]
        port: u16,
        /// Path to directory which will be served.
        #[clap(default_value = ".")]
        path: String,
        /// Use a JSON logger instead. (Follows https://github.com/trentm/node-bunyan format.)
        #[clap(short = 'j', long = "log-with-json")]
        log_with_json: bool
    }
    let args = Args::parse();

    (args.port, args.path, args.log_with_json)
}
    

#[tokio::main]
async fn main() {
    let (port, path, log_with_json) = input();
    if log_with_json {
        json_logger::init(crate_name!(), LevelFilter::Info).unwrap();
    } else {
        let mut logger = env_logger::Builder::new();
        logger.filter_module("serve", LevelFilter::Info);
        logger.filter_module("serve::path", LevelFilter::Info);
        logger.init();
    }

    info!("Serving \"{path}\" on port: \"{port}\".");

    let log = warp::log("serve::path");
    let any = warp::any()
        .and(warp::path::full())
        .map(move |url: FullPath| {
            if url.as_str().ends_with("/") {
                let indexed_url = format!("{}index.html", url.as_str());
                let file_path = PathBuf::from(format!("{path}{}", indexed_url));
                info!("file_path: {:?}", file_path);
                let content = fs::read_to_string(file_path.as_path());
                if content.is_ok() {
                    Response::builder()
                        .body(format!("{}", content.unwrap().as_str()))
                } else {
                    warn!("Unable to find \"{}\"", indexed_url);
                    Response::builder()
                        .status(404)
                        .body(format!("404: Could not find \"{}\"", indexed_url))
                }
            } else {
                let file_path = PathBuf::from(format!("{path}{}", url.as_str()));
                info!("file_path: {:?}", file_path);
                let content = fs::read_to_string(file_path.as_path());
                if content.is_ok() {
                    Response::builder()
                        .body(format!("{}", content.unwrap().as_str()))
                } else {
                    warn!("Unable to find \"{}\"", url.as_str());
                    Response::builder()
                        .status(404)
                        .body(format!("404: Could not find \"{}\"", url.as_str()))
                }
            }
        })
        .with(log); 

    warp::serve(any)
        .run(([127, 0, 0, 1], port))
        .await;
}
