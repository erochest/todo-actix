use actix_web::{HttpRequest, HttpResponse, Json, Result};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use todo::{Todo, TodoInput};

pub struct TodoCollection {
    pub next_id: Arc<AtomicUsize>,
    pub todos: Arc<RwLock<Vec<Todo>>>,
}

impl TodoCollection {
    pub fn new(todos: Arc<RwLock<Vec<Todo>>>) -> TodoCollection {
        TodoCollection {
            next_id: Arc::new(AtomicUsize::new(0)),
            todos,
        }
    }
}

pub fn get_index(req: HttpRequest<TodoCollection>) -> Result<HttpResponse> {
    let todos = Arc::clone(&req.state().todos);
    let todos = todos.read().unwrap();
    Ok(HttpResponse::Ok().json(&*todos))
}

pub fn post_index(
    (todo, req): (Json<TodoInput>, HttpRequest<TodoCollection>),
) -> Result<HttpResponse> {
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

pub fn delete_index(req: HttpRequest<TodoCollection>) -> Result<HttpResponse> {
    let todos = Arc::clone(&req.state().todos);
    let mut todos = todos.write().unwrap();
    todos.clear();
    Ok(HttpResponse::Ok().finish())
}
