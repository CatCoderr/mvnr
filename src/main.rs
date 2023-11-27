use std::convert::Infallible;
use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;

use serde_derive::Serialize;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use warp::{Buf, Filter, Rejection};
use warp::filters::path::FullPath;
use warp::http::StatusCode;
use warp::reject;

use mvnr::auth;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "mvnr")]
#[command(author = "CatCoder <catcoder@gmail.com>")]
#[command(version = "0.1.0")]
#[command(about = "A simple high performance Maven 2 repository server written in Rust", long_about = None)]
pub struct Args {
    #[arg(short, long)]
    password: String,

    #[arg(short, long, default_value = "./repository")]
    repo: String,

    #[arg(short, long, default_value = "0.0.0.0:8080")]
    host: String
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env::set_var("RUST_LOG", "maven::repo");

    pretty_env_logger::init();

    let args = Args::parse();

    let repo_dir = args.repo;
    let password = args.password;
    let repo_path = PathBuf::from(&repo_dir);

    println!("Serving directory {}", repo_path.display());

    std::fs::create_dir_all(&repo_path)?;

    let upload = warp::put()
        .and(warp::path::full())
        .and(warp::body::aggregate())
        .and(warp::any().map(move || repo_dir.clone()))
        .and_then(upload_file);
    let download = warp::get()
        .and(warp::fs::dir(repo_path));
    let endpoints = download.or(upload);

    let routes = warp::any()
        .and(auth::basic_auth(password))
        .and(endpoints)
        .recover(handle_rejection)
        .with(warp::log("maven::repo"));

    let addr: SocketAddr = args.host.parse()
        .expect("Invalid host");

    println!("Listening on http://{}", addr);

    warp::serve(routes).run(addr).await;

    Ok(())
}

#[derive(Debug)]
struct InvalidArtifactPath;

impl reject::Reject for InvalidArtifactPath {}

#[derive(Serialize)]
struct Response {
    code: u16,
    description: String,
    error: bool,
}


fn build_path(path: FullPath, repository_dir: String) -> Result<PathBuf, Rejection> {
    let request_path = path.as_str();
    let repo_dir = PathBuf::from(repository_dir);

    if !request_path.starts_with("/") {
        return Err(reject::custom(InvalidArtifactPath));
    }
    if request_path.ends_with("/") {
        return Err(reject::custom(InvalidArtifactPath));
    }

    let path = repo_dir.join(&path.as_str()[1..]);
    if path.extension().is_none() {
        return Err(reject::custom(InvalidArtifactPath));
    }

    return Ok(path);
}

async fn upload_file(path: FullPath, mut body: impl Buf, repository_dir: String) -> Result<warp::http::Response<&'static str>, Rejection> {
    let file_path = build_path(path, repository_dir)?;

    if let Some(parent) = file_path.parent() {
        tokio::fs::create_dir_all(parent).await.unwrap();
    }

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(&file_path).await.unwrap();

    while body.has_remaining() {
        file.write_buf(&mut body).await.unwrap();
    }

    Ok(warp::http::Response::builder()
        .status(StatusCode::OK)
        .body("Upload complete")
        .unwrap())
}

async fn handle_rejection(err: Rejection) -> Result<warp::http::Response<&'static str>, Infallible> {
    let code;
    let message;

    // https://github.com/seanmonstar/warp/issues/566
    if err.is_not_found() || err.find::<reject::MethodNotAllowed>().is_some() {
        code = StatusCode::NOT_FOUND;
        message = "Resource not found";
    } else if let Some(InvalidArtifactPath) = err.find() {
        code = StatusCode::BAD_REQUEST;
        message = "Invalid artifact path specified";
    } else if let Some(auth::InvalidAuthMethod) = err.find() {
        code = StatusCode::UNAUTHORIZED;
        message = "Unsupported auth method, use Basic HTTP auth"
    } else if let Some(auth::InvalidCredentials) = err.find() {
        code = StatusCode::UNAUTHORIZED;
        message = "Invalid credentials"
    } else {
        code = StatusCode::UNAUTHORIZED;
        message = "Auth challenge required";

        return Ok(warp::http::Response::builder()
            .status(code)
            .header("WWW-Authenticate", "Basic realm=\"mvnr\", charset=\"UTF-8\"")
            .body(message)
            .unwrap());
    }


    Ok(warp::http::Response::builder()
        .status(code)
        .body(message)
        .unwrap())
}