use actix_web::{HttpRequest, HttpResponse, Json, Result};
use collection::CollectionMessages;
use std::sync::mpsc::{sync_channel, SyncSender};
use todo::{Todo, TodoEdit, TodoInput};

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
