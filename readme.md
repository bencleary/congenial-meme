# Spawned Process -> Websocket Communication

## Overview

A sample repo showing how using Poem, Tokio and WebSockets you can perform cross thread communication to track the progress of a processing task. In this example the process just sleeps for 10 seconds and iterates from `1..10`.

## Design

The main idea was to create a struct which held and id (in this case a UUID) and a channel `tokio::sync::broadcast::channel` this enabled the struct to act as a router of sorts, but we needed a way to make this available on all routes within Poem. Poem does come with a Middleware called `AddData` and when using `AddData::new(<T>)` it will inject this into all handlers.

```rust
struct AppState {
    clients: Mutex<HashMap<String, Sender<String>>>,
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // other stuff

    let state = Arc::new(AppState {
        clients: Mutex::new(HashMap::new()),
    });

    let app = Route::new()
        .at("/", get(index))
        .at("/ws/:id", get(ws))
        .with(AddData::new(state))
        .with(Tracing);

    // other stuff
}
```

TODO - explain why Arc was needed.

## Inspiration

The following links were a great help.

- https://tokio.rs/tokio/tutorial/shared-state
- https://users.rust-lang.org/t/axum-within-the-standard-chat-example-how-would-you-implement-multiple-chat-rooms/82519
- https://github.com/poem-web/poem/tree/master/examples/poem/mongodb
