// TODO: library
// TODO: modules
// TODO: next_id has to be stored here as well and passed into build_app so it will synchronize across threads.
// TODO: spawn a repository in a separate thread and use channels to communicate
// TODO: use a for-real database
// TODO: go all-in on hal

extern crate actix;
extern crate actix_web;
extern crate futures;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate url;
extern crate url_serde;

use actix_web::http::header;
use actix_web::middleware::cors;
use actix_web::{server, App, HttpRequest, HttpResponse, Json, Path, Result};
use std::env;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use url::Url;

#[derive(Debug, Deserialize, Serialize, Clone)]
struct TodoInput {
    title: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Todo {
    id: usize,
    title: String,
    completed: bool,
    #[serde(with = "url_serde")]
    url: Url,
}

impl Todo {
    fn new(id: usize, title: String, url: Url) -> Todo {
        Todo {
            id,
            title,
            completed: false,
            url,
        }
    }
}

impl From<TodoInput> for Todo {
    fn from(input: TodoInput) -> Todo {
        Todo {
            id: 0,
            title: input.title,
            completed: false,
            url: "/".parse().unwrap(),
        }
    }
}

struct TodoCollection {
    next_id: Arc<AtomicUsize>,
    todos: Arc<RwLock<Vec<Todo>>>,
}

impl TodoCollection {
    fn new(todos: Arc<RwLock<Vec<Todo>>>) -> TodoCollection {
        TodoCollection {
            next_id: Arc::new(AtomicUsize::new(0)),
            todos,
        }
    }
}

fn get_index(req: HttpRequest<TodoCollection>) -> Result<HttpResponse> {
    let todos = Arc::clone(&req.state().todos);
    let todos = todos.read().unwrap();
    Ok(HttpResponse::Ok().json(&*todos))
}

fn post_index((todo, req): (Json<TodoInput>, HttpRequest<TodoCollection>)) -> Result<HttpResponse> {
    let next_id = Arc::clone(&req.state().next_id).fetch_add(1, Ordering::Relaxed);
    let url = req.url_for("todo", &[format!("{}", next_id)]).unwrap();
    let todo = Todo::new(next_id, todo.0.title, url);
    let todos = Arc::clone(&req.state().todos);
    {
        let mut todos = todos.write().unwrap();
        todos.push(todo.clone());
    }
    Ok(HttpResponse::Ok().json(todo))
}

fn delete_index(req: HttpRequest<TodoCollection>) -> Result<HttpResponse> {
    let todos = Arc::clone(&req.state().todos);
    let mut todos = todos.write().unwrap();
    todos.clear();
    Ok(HttpResponse::Ok().finish())
}

fn get_todo((todo_id, req): (Path<usize>, HttpRequest<TodoCollection>)) -> Result<HttpResponse> {
    let todo_id = *todo_id;
    let todos = Arc::clone(&req.state().todos);
    let todos = todos.read().unwrap();
    let todo = todos.iter().filter(|t| t.id == todo_id).nth(0);
    Ok(todo.map(|t| HttpResponse::Ok().json(t))
        .unwrap_or_else(|| HttpResponse::NotFound().finish()))
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
