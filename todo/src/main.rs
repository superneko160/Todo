use actix_web::{get, http::header, post, web::{self, Data}, App, HttpResponse, HttpServer, ResponseError};
use serde::Deserialize;
use askama::Template;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
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

    #[error("Failed to get connection")]
    ConnectionPoolError(#[from]r2d2::Error),

    #[error("Failed SQL execution")]
    SQLiteError(#[from]rusqlite::Error),
}

impl ResponseError for MyError {}

#[get("/")]
async fn index(
    db: web::Data<Pool<SqliteConnectionManager>>
) -> Result<HttpResponse, MyError> {
    // DB接続
    let conn = db.get()?;
    // SQL準備
    let mut statement = conn.prepare("SELECT id, text FROM todo")?;
    // SQL実行
    let rows = statement.query_map(params![], |row| {
        let id = row.get(0)?;
        let text = row.get(1)?;
        Ok(TodoEntry {id, text})
    })?;
    // データ取得
    let mut entries = Vec::new();
    for row in rows {
        entries.push(row?);
    }
    // HTMLテンプレートに埋め込み
    let html = IndexTemplate{ entries };
    let response_body = html.render()?;
    // HTTP:200
    Ok(HttpResponse::Ok().content_type("text/html").body(response_body))
}

#[post("/add")]
async fn add_todo(
    params: web::Form<AddParams>,
    db: web::Data<r2d2::Pool<SqliteConnectionManager>>,
) -> Result<HttpResponse, MyError> {
    // DB接続
    let conn = db.get()?;
    // SQL実行
    conn.execute("INSERT INTO todo (text) VALUES (?)", &[&params.text])?;
    // HTTP:303(SeeOther())
    Ok(HttpResponse::SeeOther().append_header((header::LOCATION, "/")).finish())
}

#[post("/delete")]
async fn delete_todo(
    params: web::Form<DeleteParams>,
    db: web::Data<r2d2::Pool<SqliteConnectionManager>>,
) -> Result<HttpResponse, MyError> {
    // DB接続
    let conn = db.get()?;
    // SQL実行
    conn.execute("DELETE FROM todo WHERE id = ?", &[&params.id])?;
    // HTTP:303(SeeOther())
    Ok(HttpResponse::SeeOther().append_header((header::LOCATION, "/")).finish())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // env_logger有効化
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    // DBに接続しテーブルが存在しなければ作成
    let manager = SqliteConnectionManager::file("todo.db");
    let pool = Pool::new(manager).expect("Failed to initialize the connection pool.");
    let conn = pool.get().expect("Failed to get the connection from the pool");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS todo (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            text TEXT NOT NULL
        )",
        params![],
    )
    .expect("Failed to create a table `todo`.");
    // サーバ実行
    HttpServer::new(move || {  // poolの所有権をクロージャに移動
        App::new()
        .app_data(Data::new(pool.clone())) // poolをData::newでラップし登録
        .service(index)
        .service(add_todo)
        .service(delete_todo)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
