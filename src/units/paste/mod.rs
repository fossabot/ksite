/*
账户登录
文件存储
加密
CRUI 轻量的界面
在用户之间分享
尽量优化性能
有限制的自增和其他结合的 ID
*/
use crate::db;
use crate::utils::slot;
use axum::extract::{Form, Path};
use axum::response::{Html, Redirect};
use axum::routing::MethodRouter;
use axum::Router;
use serde::Deserialize;

fn db_init() {
    db!("CREATE TABLE paste_users (uid TEXT PRIMARY KEY, password BLOB)").ok();
    db!("CREATE TABLE paste_data (id TEXT PRIMARY KEY, uid TEXT)").ok();
}
fn db_users_reg(id: &str, password: &[u8]) -> bool {
    todo!()
}
fn db_users_match(id: &str, password: &[u8]) -> bool {
    todo!()
}
fn db_insert(data: &str) -> u64 {
    db!("INSERT INTO paste VALUES (NULL, ?)", [data]).unwrap();
    db!("SELECT last_insert_rowid() FROM paste", [], |r| r.get(0)).unwrap()[0]
}
fn db_update(id: u64, data: &str) {
    db!("UPDATE paste SET data = ?1 WHERE id = ?2;", [data, id]).unwrap();
}
fn db_get(id: u64) -> Option<String> {
    let r = db!("SELECT data FROM paste WHERE id = ?", [id], |r| r.get(0));
    r.unwrap().pop()
}

fn escape(v: &str) -> String {
    askama_escape::escape(v, askama_escape::Html).to_string()
}

async fn read(id: Option<u64>) -> Html<String> {
    let value = id.and_then(db_get);
    let value = value.unwrap_or_else(|| "New entry".to_string());
    const PAGE: [&str; 2] = slot(include_str!("page.html"));
    Html([PAGE[0], &value, PAGE[1]].join(""))
}

#[derive(Deserialize)]
struct Data {
    value: String,
}

async fn insert(form: Form<Data>) -> Redirect {
    let id = db_insert(&escape(&form.value));
    Redirect::to(&format!("/paste/{id}"))
}

pub fn service() -> Router {
    db_init();
    Router::new().route(
        "/paste",
        MethodRouter::new().get(|| read(None)).post(insert),
    )
}
