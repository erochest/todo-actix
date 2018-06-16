extern crate todo_actix;

use std::env;

fn main() {
    let port: usize = env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);
    let addr = format!("0.0.0.0:{}", &port);
    todo_actix::run(addr);
}
