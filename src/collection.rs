use actix_web::{HttpRequest, HttpResponse, Json, Result};
use std::sync::mpsc::{sync_channel, SyncSender};
use todo::{Todo, TodoEdit, TodoInput};

#[derive(Debug)]
pub enum CollectionMessages {
    GetList(SyncSender<Vec<Todo>>),
    PostList(TodoInput, String, SyncSender<Todo>),
    DeleteList(SyncSender<bool>),
    GetItem(usize, SyncSender<Option<Todo>>),
    DeleteItem(usize, SyncSender<Option<bool>>),
    PatchItem(usize, TodoEdit, SyncSender<Option<Todo>>),
}

/// This runs in one thread and takes requests sequentially from clients.
pub struct TodoCollection {
    pub next_id: usize,
    pub todos: Vec<Todo>,
}

impl TodoCollection {
    pub fn new() -> TodoCollection {
        TodoCollection {
            next_id: 0,
            todos: Vec::new(),
        }
    }

    pub fn execute(&mut self, message: CollectionMessages) {
        let _ = match message {
            CollectionMessages::GetList(tx) => tx.send(self.get_list()).unwrap(),
            CollectionMessages::PostList(todo_input, url_template, tx) => {
                tx.send(self.post_list(todo_input, url_template)).unwrap()
            }
            CollectionMessages::DeleteList(tx) => tx.send(self.delete_list()).unwrap(),
            CollectionMessages::GetItem(id, tx) => tx.send(self.get_item(id)).unwrap(),
            CollectionMessages::DeleteItem(id, tx) => tx.send(self.delete_item(id)).unwrap(),
            CollectionMessages::PatchItem(id, todo_edit, tx) => {
                tx.send(self.patch_item(id, todo_edit)).unwrap()
            }
        };
    }

    fn get_list(&self) -> Vec<Todo> {
        self.todos.clone()
    }

    fn post_list(&mut self, todo_input: TodoInput, url_template: String) -> Todo {
        let id = self.next_id;
        let url = url_template.replace("ID", &id.to_string()).parse().unwrap();
        let todo = Todo::new(id, todo_input.title, todo_input.order, url);
        self.next_id += 1;
        self.todos.push(todo.clone());
        todo
    }

    fn delete_list(&mut self) -> bool {
        self.todos.clear();
        true
    }

    fn find(&self, id: usize) -> Option<(usize, &Todo)> {
        self.todos
            .iter()
            .enumerate()
            .filter(|(_i, todo)| todo.id == id)
            .nth(0)
    }

    fn get_item(&self, id: usize) -> Option<Todo> {
        self.find(id).map(|(_i, todo)| todo.clone())
    }

    fn delete_item(&mut self, id: usize) -> Option<bool> {
        let original_size = self.todos.len();
        self.todos.retain(|todo| todo.id != id);
        if original_size > self.todos.len() {
            Some(true)
        } else {
            None
        }
    }

    fn patch_item(&mut self, id: usize, todo_edit: TodoEdit) -> Option<Todo> {
        self.todos
            .iter_mut()
            .filter(|todo| todo.id == id)
            .nth(0)
            .map(move |todo| {
                todo_edit
                    .title
                    .iter()
                    .for_each(|title| todo.title = title.to_string());
                todo_edit
                    .completed
                    .iter()
                    .for_each(|completed| todo.completed = *completed);
                todo
            })
            .map(|todo| todo.clone())
    }
}

/// This runs in the server threads and takes responses from servers.
#[derive(Clone)]
pub struct TodoClient {
    pub tx: SyncSender<CollectionMessages>,
}

impl TodoClient {
    pub fn new(tx: SyncSender<CollectionMessages>) -> TodoClient {
        TodoClient { tx }
    }

    fn send_message<R, F>(&self, message_builder: F) -> R
    where
        F: FnOnce(SyncSender<R>) -> CollectionMessages,
    {
        let (tx, rx) = sync_channel(0);
        let message = message_builder(tx);
        self.tx.send(message).unwrap();
        rx.recv().unwrap()
    }

    pub fn get_list(&self) -> Vec<Todo> {
        self.send_message(|tx| CollectionMessages::GetList(tx))
    }

    pub fn post_list(&self, todo_input: TodoInput, url_template: String) -> Todo {
        self.send_message(|tx| CollectionMessages::PostList(todo_input, url_template, tx))
    }

    pub fn delete_list(&self) -> bool {
        self.send_message(|tx| CollectionMessages::DeleteList(tx))
    }

    pub fn get_item(&self, id: usize) -> Option<Todo> {
        self.send_message(|tx| CollectionMessages::GetItem(id, tx))
    }

    pub fn delete_item(&self, id: usize) -> Option<bool> {
        self.send_message(|tx| CollectionMessages::DeleteItem(id, tx))
    }

    pub fn patch_item(&self, id: usize, todo_edit: TodoEdit) -> Option<Todo> {
        self.send_message(|tx| CollectionMessages::PatchItem(id, todo_edit, tx))
    }
}

pub fn get_index(req: HttpRequest<TodoClient>) -> Result<HttpResponse> {
    let todos = &req.state().get_list();
    Ok(HttpResponse::Ok().json(&*todos))
}

pub fn post_index((todo, req): (Json<TodoInput>, HttpRequest<TodoClient>)) -> Result<HttpResponse> {
    let url_template: String = req.url_for("todo", &[String::from("ID")])
        .unwrap()
        .as_str()
        .into();
    let todo = &req.state().post_list(todo.0, url_template);
    Ok(HttpResponse::Ok().json(todo))
}

pub fn delete_index(req: HttpRequest<TodoClient>) -> Result<HttpResponse> {
    let _ok = &req.state().delete_list();
    Ok(HttpResponse::Ok().finish())
}
