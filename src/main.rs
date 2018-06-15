extern crate actix;
extern crate actix_web;
extern crate futures;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use actix_web::http::{header, Method};
use actix_web::middleware::cors;
use actix_web::{server, App, HttpRequest, HttpResponse, Json, Path, Responder, Result};
use std::env;
use std::sync::{Arc, RwLock};

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Todo {
    title: String,
}

struct TodoCollection {
    pub todos: Arc<RwLock<Vec<Todo>>>,
}

impl TodoCollection {
    fn new(todos: Arc<RwLock<Vec<Todo>>>) -> TodoCollection {
        TodoCollection { todos }
    }
}

fn index(req: HttpRequest<TodoCollection>) -> Result<HttpResponse> {
    let todos = Arc::clone(&req.state().todos);
    let todos = todos.read().unwrap();
    Ok(HttpResponse::Ok().json(&*todos))
}

fn post_index((todo, req): (Json<Todo>, HttpRequest<TodoCollection>)) -> Result<HttpResponse> {
    let todos = Arc::clone(&req.state().todos);
    {
        let mut todos = todos.write().unwrap();
        todos.push(todo.0.clone());
    }
    Ok(HttpResponse::Ok().json(todo.0))
}

fn delete_index(req: HttpRequest<TodoCollection>) -> Result<HttpResponse> {
    let todos = Arc::clone(&req.state().todos);
    let mut todos = todos.write().unwrap();
    todos.clear();
    Ok(HttpResponse::Ok().finish())
}

fn main() {
    let port: usize = env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);
    let sys = actix::System::new("todo-actix");
    let addr = format!("0.0.0.0:{}", &port);
    let todos = Arc::new(RwLock::new(Vec::new()));
    let todos_cloned = Arc::clone(&todos);

    server::new(move || build_app(Arc::clone(&todos_cloned)))
        .bind(&addr)
        .expect(&format!("Cannot bind to {}", &addr))
        .shutdown_timeout(0)
        .start();

    println!("Starting http server: {}", &addr);
    let _ = sys.run();
}

fn build_app(todos: Arc<RwLock<Vec<Todo>>>) -> App<TodoCollection> {
    App::with_state(TodoCollection::new(todos)).configure(|app| {
        cors::Cors::for_app(app)
            .allowed_origin("https://www.todobackend.com")
            .allowed_methods(vec!["GET", "POST", "DELETE"])
            .allowed_headers(vec![header::CONTENT_TYPE])
            .allowed_header(header::CONTENT_TYPE)
            .max_age(3600)
            .resource("/", |r| {
                r.get().with(index);
                r.post().with(post_index);
                r.delete().f(delete_index);
            })
            .register()
    })
}
