use async_stream::stream;
use futures_util::{SinkExt, StreamExt};
use poem::{
    error::InternalServerError,
    get, handler,
    listener::TcpListener,
    middleware::{AddData, Tracing},
    web::{
        sse::{Event, SSE},
        websocket::{Message, WebSocket},
        Data, Html, Path,
    },
    EndpointExt, IntoResponse, Route, Server,
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tera::{Context, Tera};
use tokio::sync::broadcast::Sender;
use tokio::time::{sleep, Duration};

use uuid::Uuid;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = match Tera::new("templates/**/*") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        tera.autoescape_on(vec![".html", ".sql"]);
        tera
    };
}

struct AppState {
    clients: Mutex<HashMap<String, Sender<String>>>,
}

async fn process(channel: Sender<String>) {
    for i in 1..10 {
        let progress = i * 10;
        // TODO - investigate why unwarp causes error
        let _ = channel.send(progress.to_string());
        sleep(Duration::from_secs(10)).await;
    }
}

#[handler]
fn index(state: Data<&Arc<AppState>>) -> Result<Html<String>, poem::Error> {
    let mut s = state.clients.lock().unwrap();
    let id = Uuid::new_v4();
    let sender = tokio::sync::broadcast::channel::<String>(32).0;
    let sender_worker = sender.clone();
    s.insert(id.to_string(), sender);

    tokio::spawn(async {
        process(sender_worker).await;
    });

    let mut context = Context::new();
    context.insert("id", &id.to_string());
    TEMPLATES
        .render("index.html.tera", &context)
        .map_err(InternalServerError)
        .map(Html)
}

#[handler]
async fn event(Path(name): Path<String>, state: Data<&Arc<AppState>>) -> SSE {
    let s = state.clients.lock().unwrap();
    let sender = s.get(&name).unwrap();
    let mut rx = sender.subscribe();

    let stream = stream! {
        while let Ok(item) = rx.recv().await {
            yield item;
        }
    };

    SSE::new(stream.map(|item| Event::message(item))).keep_alive(Duration::from_secs(5))
}

#[handler]
fn ws(Path(name): Path<String>, ws: WebSocket, state: Data<&Arc<AppState>>) -> impl IntoResponse {
    let client = state.clients.lock().unwrap();
    let sender = client.get(&name).unwrap().clone();
    let mut receiver = sender.subscribe();
    ws.on_upgrade(move |socket| async move {
        let (mut sink, mut stream) = socket.split();

        tokio::spawn(async move {
            while let Some(Ok(msg)) = stream.next().await {
                if let Message::Text(text) = msg {
                    if sender.send(format!("{}: {}", name, text)).is_err() {
                        break;
                    }
                }
            }
        });

        tokio::spawn(async move {
            while let Ok(msg) = receiver.recv().await {
                if sink.send(Message::Text(msg)).await.is_err() {
                    break;
                }
            }
        });
    })
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "poem=debug");
    }
    tracing_subscriber::fmt::init();

    let state = Arc::new(AppState {
        clients: Mutex::new(HashMap::new()),
    });

    let app = Route::new()
        .at("/", get(index))
        .at("/ws/:id", get(ws))
        .at("/event/:id", get(event))
        .with(AddData::new(state))
        .with(Tracing);

    Server::new(TcpListener::bind("127.0.0.1:3000"))
        .run(app)
        .await
}
