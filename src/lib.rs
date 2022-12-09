pub mod handlers;
pub mod tasks;

use std::collections::HashMap;
use tera::Tera;
use tokio::sync::broadcast::Sender;
use tokio::sync::Mutex;

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

pub struct AppState {
    pub clients: Mutex<HashMap<String, Sender<String>>>,
}
