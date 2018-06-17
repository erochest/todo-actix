// TODO: spawn a repository in a separate thread and use channels to communicate
// TODO: use a for-real database
// TODO: go all-in on hal

extern crate actix;
extern crate actix_web;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate url;
extern crate url_serde;

mod collection;
mod todo;

use actix_web::http::header;
use actix_web::middleware::cors;
use actix_web::{server, App};
use collection::{delete_index, get_index, post_index, TodoCollection};
use std::sync::{Arc, RwLock};
use todo::{get_todo, Todo};

pub fn run(address: String) {
    let sys = actix::System::new("todo-actix");
    let collection = TodoCollection::new();
    let col_clone = collection.clone();

    server::new(move || build_app(col_clone.clone()))
        .bind(&address)
        .expect(&format!("Cannot bind to {}", &address))
        .shutdown_timeout(0)
        .start();

    println!("Starting http server: {}", &address);
    let _ = sys.run();
}

fn build_app(collection: TodoCollection) -> App<TodoCollection> {
    App::with_state(collection).configure(|app| {
        cors::Cors::for_app(app)
            .allowed_origin("https://www.todobackend.com")
            .allowed_methods(vec!["GET", "POST", "DELETE"])
            .allowed_headers(vec![header::CONTENT_TYPE])
            .allowed_header(header::CONTENT_TYPE)
            .max_age(3600)
            .resource("/{id}", |r| {
                r.name("todo");
                r.get().with(get_todo);
            })
            .resource("/", |r| {
                r.get().with(get_index);
                r.post().with(post_index);
                r.delete().f(delete_index);
            })
            .register()
    })
}
