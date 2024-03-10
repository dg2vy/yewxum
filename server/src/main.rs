use std::{
    net::{IpAddr, Ipv4Addr},
    path::PathBuf,
    str::FromStr,
};

use axum::{
    body::Body,
    extract::Request,
    http::StatusCode,
    response::{IntoResponse, Html},
    routing::get, 
    Router,
};

use tokio::{
    fs,
    net::TcpListener,
};

use tower_http::{
    trace::TraceLayer,
    services::ServeDir,
};

use tower::{ServiceBuilder, ServiceExt};
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name="server", about="A server for our wasm Project!!")]
struct Opt {
    #[clap(short='l', long="log", default_value="debug")]
    log_level: String,

    #[clap(short='a', long="addr", default_value="127.0.0.1")]
    addr: String,

    #[clap(short='p', long="port", default_value="8000")]
    port: u16,

    #[clap(long="static-dir", default_value="./dist")]
    static_dir:String,
}

#[tokio::main]
async fn main() {
    let opt = Opt::parse();

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", format!("{}, hyper=info,mio=info", opt.log_level))
    }

    tracing_subscriber::fmt::init();

    let addr = format!("{}:{}",IpAddr::from_str(opt.addr.as_str()).unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST)), opt.port);
    let tcp_addr = TcpListener::bind(&addr).await.unwrap();
    let app = Router::new()
        .route("/api/hello", get(hello))
        .fallback_service(get(
            |req: Request<Body>| async move {
                let res = ServeDir::new(&opt.static_dir).oneshot(req).await.unwrap();
                let status = res.status();
                match status {
                    StatusCode::NOT_FOUND => {
                        let index_path = PathBuf::from(&opt.static_dir).join("index.html");
                        fs::read_to_string(index_path)
                        .await
                        .map(|index_content| (StatusCode::OK, Html(index_content)).into_response())
                        .unwrap_or_else(|_| {
                            (StatusCode::INTERNAL_SERVER_ERROR, "index.html not found")
                                .into_response()
                        })
                    }
                    _ => res.into_response(),
                }
            }
        ))
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));

    // println!("[+] Server Run {}", Addr);
    log::info!("Listening on http://{}", addr);

    axum::serve(tcp_addr, app.into_make_service())
        .await
        .expect("[-] Unable to startr server");
}

async fn hello() -> impl IntoResponse {
    "Hello from Server!"
}