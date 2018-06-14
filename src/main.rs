extern crate actix;
extern crate actix_web;

use actix_web::{server, App, HttpRequest, HttpResponse, Path, Responder, Result};
use actix_web::http::{header, Method, StatusCode};
use actix_web::middleware::cors;
use std::env;

fn index(mut _req: HttpRequest) -> Result<HttpResponse> {
    Ok(
        HttpResponse::build(StatusCode::OK)
            .content_type("text/plain")
            .body("Hello, world!"),
    )
}

fn greeting(info: Path<(u32, String)>) -> impl Responder {
    format!("Greetings, {}. (ID {})", info.1, info.0)
}

fn main() {
    let port: usize = env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);
    let sys = actix::System::new("todo-actix");
    let addr = format!("0.0.0.0:{}", &port);

    server::new(build_app)
        .bind(&addr)
        .expect(&format!("Cannot bind to {}", &addr))
        .shutdown_timeout(0)
        .start();

    println!("Starting http server: {}", &addr);
    let _ = sys.run();
}

fn build_app() -> App {
    App::new()
        .configure(|app| {
            cors::Cors::for_app(app)
                .allowed_origin("https://www.todobackend.com")
                .allowed_methods(vec!["GET", "POST", "DELETE"])
                .allowed_headers(vec![header::CONTENT_TYPE])
                .allowed_header(header::CONTENT_TYPE)
                .max_age(3600)
                .resource("/greeting/{id}/{name}/", |r| r.method(Method::GET).with(greeting))
                .resource("/", |r| r.f(index))
                .register()
        })
}
