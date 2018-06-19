// TODO: use futures or actors for thread communication
// TODO: better error handling
// TODO: use a for-real database
// TODO: go all-in on hal

extern crate actix;
extern crate actix_web;
extern crate env_logger;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate url;
extern crate url_serde;

mod collection;
mod todo;

use actix_web::http::{header, Method};
use actix_web::middleware::{cors, Logger};
use actix_web::{server, App};
use collection::{delete_index, get_index, post_index, TodoClient, TodoCollection};
use std::sync::mpsc::sync_channel;
use std::thread;
use todo::{get_todo, patch_todo};

pub fn run(address: String) {
    env_logger::init();
    let sys = actix::System::new("todo-actix");

    let (tx, rx) = sync_channel(1028);

    thread::spawn(move || {
        let mut todo_server = TodoCollection::new();
        for msg in rx {
            todo_server.execute(msg);
        }
    });

    let client = TodoClient::new(tx);
    let client_clone = client.clone();
    server::new(move || build_app(client_clone.clone()))
        .bind(&address)
        .expect(&format!("Cannot bind to {}", &address))
        .shutdown_timeout(0)
        .start();

    println!("Starting http server: {}", &address);
    let _ = sys.run();
}

fn build_app(client: TodoClient) -> App<TodoClient> {
    App::with_state(client)
        .middleware(Logger::default())
        .configure(|app| {
            cors::Cors::for_app(app)
                .allowed_origin("https://www.todobackend.com")
                .allowed_methods(vec!["GET", "POST", "DELETE", "PATCH"])
                .allowed_headers(vec![header::CONTENT_TYPE])
                .allowed_header(header::CONTENT_TYPE)
                .max_age(3600)
                .resource("/{id}", |r| {
                    r.name("todo");
                    r.get().with(get_todo);
                    r.method(Method::PATCH).with(patch_todo);
                })
                .resource("/", |r| {
                    r.get().with(get_index);
                    r.post().with(post_index);
                    r.delete().f(delete_index);
                })
                .register()
        })
}
