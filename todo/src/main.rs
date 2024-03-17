use actix_web::{get, post, http::header, web, App, HttpResponse, HttpServer, ResponseError};
use serde::Deserialize;
use askama::Template;
use thiserror::Error;

struct TodoEntry {
    id: u32,
    text: String,
}

#[derive(Deserialize)]
struct AddParams {
    text: String,
}

#[derive(Deserialize)]
struct DeleteParams {
    id: u32,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    entries: Vec<TodoEntry>,
}

#[derive(Error, Debug)]
enum MyError {
    #[error("Failed to render HTML")]
    AskamaError(#[from] askama::Error),
}

impl ResponseError for MyError {}


#[get("/")]
async fn index() -> Result<HttpResponse, MyError> {
    let mut entries = Vec::new();
    entries.push(TodoEntry {
        id: 1,
        text: "First entry".to_string(),
    });
    entries.push(TodoEntry {
        id: 2,
        text: "Second entry".to_string(),
    });
    let html = IndexTemplate{ entries };
    let response_body = html.render()?;
    Ok(HttpResponse::Ok().content_type("text/html").body(response_body))
}

#[post("/add")]
async fn add_todo(params: web::Form<AddParams>) -> Result<HttpResponse, MyError> {
    println!("{}", params.text);
    // SeeOther -> HTTP:303
    Ok(HttpResponse::SeeOther().append_header((header::LOCATION, "/")).finish())
}

#[post("/delete")]
async fn delete_todo(params: web::Form<DeleteParams>) -> Result<HttpResponse, MyError> {
    println!("{}", params.id);
    // SeeOther -> HTTP:303
    Ok(HttpResponse::SeeOther().append_header((header::LOCATION, "/")).finish())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
        .service(index)
        .service(add_todo)
        .service(delete_todo)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
