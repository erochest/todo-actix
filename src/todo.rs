use actix_web::{HttpRequest, HttpResponse, Path, Result};
use collection::TodoClient;
use url::Url;
use url_serde;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TodoInput {
    pub title: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Todo {
    pub id: usize,
    pub title: String,
    pub completed: bool,
    #[serde(with = "url_serde")]
    pub url: Url,
}

impl Todo {
    pub fn new(id: usize, title: String, url: Url) -> Todo {
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

pub fn get_todo((todo_id, req): (Path<usize>, HttpRequest<TodoClient>)) -> Result<HttpResponse> {
    Ok(req.state()
        .get_item(todo_id.into_inner())
        .map(|todo| HttpResponse::Ok().json(todo))
        .unwrap_or_else(|| HttpResponse::NotFound().finish()))
}
