use futures_util::{stream::BoxStream, StreamExt};
use poem::{listener::TcpListener, Route, Server};
use poem_openapi::{
    payload::{EventStream, PlainText},
    Object, OpenApi, OpenApiService,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::{sync::broadcast::Sender, time::Duration};
use uuid::Uuid;

#[derive(Object)]
struct Event {
    value: i32,
}

type Db = Arc<Mutex<HashMap<String, Sender<String>>>>;

struct Api {
    db: Db,
}

async fn process() -> Event {
    for i in 0.. {
        tokio::time::sleep(Duration::from_secs(1)).await;
        yield Event { value: i };
    }
}

#[OpenApi]
impl Api {
    pub fn new(db: Db) -> Self {
        Self { db: db }
    }

    #[oai(path = "/page", method = "get")]
    async fn page(&self) -> PlainText<String> {
        let mut router = self.db.lock().unwrap();
        let id = Uuid::new_v4();
        router.insert(
            id.to_string(),
            tokio::sync::broadcast::channel::<String>(32).0,
        );
        PlainText(id.to_string())
    }

    #[oai(path = "/events", method = "get")]
    async fn index(&self) -> EventStream<BoxStream<'static, Event>> {
        EventStream::new(
            async_stream::stream! {
                for i in 0.. {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    yield Event { value: i };
                }
            }
            .boxed(),
        )
    }
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "poem=debug");
    }
    tracing_subscriber::fmt::init();

    let db = Arc::new(Mutex::new(HashMap::<String, Sender<String>>::new()));

    let api_service =
        OpenApiService::new(Api::new(db), "Hello World", "1.0").server("http://localhost:3000/api");
    let ui = api_service.swagger_ui();

    Server::new(TcpListener::bind("127.0.0.1:3000"))
        .run(Route::new().nest("/api", api_service).nest("/", ui))
        .await
}
