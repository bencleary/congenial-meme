use crate::tasks::convert_to_wav;
use crate::AppState;
use crate::TEMPLATES;
use futures_util::{SinkExt, StreamExt};
use poem::{
    error::InternalServerError,
    handler,
    web::{
        websocket::{Message, WebSocket},
        Data, Html, Multipart, Path,
    },
    IntoResponse,
};
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::sync::Arc;
use tera::Context;
use uuid::Uuid;

#[handler]
pub async fn index(state: Data<&Arc<AppState>>) -> Result<Html<String>, poem::Error> {
    let mut s = state.clients.lock().await;
    let id = Uuid::new_v4();
    let sender = tokio::sync::broadcast::channel::<String>(32).0;
    // let sender_worker = sender.clone();
    s.insert(id.to_string(), sender);

    // tokio::spawn(async {
    //     process(sender_worker).await;
    // });

    let mut context = Context::new();
    context.insert("id", &id.to_string());
    TEMPLATES
        .render("index.html.tera", &context)
        .map_err(InternalServerError)
        .map(Html)
}

#[handler]
pub async fn upload(
    state: Data<&Arc<AppState>>,
    Path(uuid): Path<String>,
    mut upload_form: Multipart,
) -> Result<Html<String>, poem::Error> {
    // get client details using URL path
    let client = state.clients.lock().await;
    let sender = client.get(&uuid).unwrap().clone();

    let mut context = Context::new();
    context.insert("id", &uuid);

    while let Ok(Some(field)) = upload_form.next_field().await {
        let file_name = field.file_name().map(ToString::to_string);
        if let Ok(bytes) = field.bytes().await {
            // create working directory for file based on UUID
            let working_dir = format!("temp/{}", uuid);
            if !std::path::Path::new(&working_dir).exists() {
                match fs::create_dir(&working_dir) {
                    Ok(()) => (),
                    Err(error) => panic!("{:?}", error),
                };
            }

            let upload_file_path = format!("temp/{}/{}", uuid, file_name.as_ref().unwrap());
            let mut file = File::create(upload_file_path).unwrap();
            file.write_all(&bytes).unwrap();
        }
    }

    if let Ok(Some(field)) = upload_form.next_field().await {
        println!("Spawing Task");
        tokio::spawn(async move {
            convert_to_wav(uuid, field.file_name().unwrap().to_string(), sender).await;
        });
    }

    TEMPLATES
        .render("partials/progress.html.tera", &context)
        .map_err(InternalServerError)
        .map(Html)
}

#[handler]
pub async fn ws(
    Path(name): Path<String>,
    ws: WebSocket,
    state: Data<&Arc<AppState>>,
) -> impl IntoResponse {
    let client = state.clients.lock().await;
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
