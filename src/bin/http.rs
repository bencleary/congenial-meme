use poem::{get, listener::TcpListener, middleware::Tracing, post, EndpointExt, Route, Server};
use std::{collections::HashMap, sync::Arc};
use std::{fs, path::Path};
use tokio::sync::Mutex;
use transcriber::handlers::{index, upload, ws};
use transcriber::AppState;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "poem=debug");
    }
    tracing_subscriber::fmt::init();

    let state = Arc::new(AppState {
        clients: Mutex::new(HashMap::new()),
    });

    // create temporary directory for file uploads
    if !Path::new("temp").exists() {
        match fs::create_dir("temp") {
            Ok(()) => (),
            Err(error) => panic!("{:?}", error),
        };
    }

    let app = Route::new()
        .at("/", get(index))
        .at("/upload/:uuid", post(upload))
        .at("/ws/:id", get(ws))
        .data(state)
        .with(Tracing);

    Server::new(TcpListener::bind("127.0.0.1:3000"))
        .run(app)
        .await
}
