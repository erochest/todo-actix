use actix_web::{HttpRequest, HttpResponse, Json, Path, Result};
use client::TodoClient;
use url::Url;
use url_serde;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TodoInput {
    pub title: String,
    pub order: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TodoEdit {
    pub title: Option<String>,
    pub completed: Option<bool>,
    pub order: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Todo {
    pub id: usize,
    pub title: String,
    pub completed: bool,
    pub order: Option<u32>,
    #[serde(with = "url_serde")]
    pub url: Url,
}

impl Todo {
    pub fn new(id: usize, title: String, order: Option<u32>, url: Url) -> Todo {
        Todo {
            id,
            title,
            completed: false,
            order,
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
            order: input.order,
            url: "/".parse().unwrap(),
        }
    }
}

pub fn get_todo((todo_id, req): (Path<usize>, HttpRequest<TodoClient>)) -> Result<HttpResponse> {
    Ok(req.state()
        .get_item(todo_id.into_inner())
        .map(|todo| HttpResponse::Ok().json(todo))
        .unwrap_or_else(|| HttpResponse::NotFound().finish()))
}

pub fn delete_todo((todo_id, req): (Path<usize>, HttpRequest<TodoClient>)) -> Result<HttpResponse> {
    Ok(req.state()
        .delete_item(todo_id.into_inner())
        .map(|_flag| HttpResponse::Ok().finish())
        .unwrap_or_else(|| HttpResponse::NotFound().finish()))
}

pub fn patch_todo(
    (todo_id, todo_edit, req): (Path<usize>, Json<TodoEdit>, HttpRequest<TodoClient>),
) -> Result<HttpResponse> {
    Ok(req.state()
        .patch_item(todo_id.into_inner(), todo_edit.0)
        .map(|todo| HttpResponse::Ok().json(todo))
        .unwrap_or_else(|| HttpResponse::NotFound().finish()))
}
