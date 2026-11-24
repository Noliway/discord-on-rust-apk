use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;

#[derive(Clone, Serialize)]
struct Msg {
    id: u64,
    author: String,
    text: String,
}

#[derive(Deserialize)]
struct NewMsg {
    author: String,
    text: String,
}

#[derive(Deserialize)]
struct After {
    after: Option<u64>,
}

#[derive(Default)]
struct Store {
    next_id: u64,
    channels: HashMap<String, Vec<Msg>>,
}

type Shared = Arc<Mutex<Store>>;

async fn list(
    State(st): State<Shared>,
    Path(ch): Path<String>,
    Query(q): Query<After>,
) -> Json<Vec<Msg>> {
    let st = st.lock().unwrap();
    let after = q.after.unwrap_or(0);
    let out = st
        .channels
        .get(&ch)
        .map(|v| v.iter().filter(|m| m.id > after).cloned().collect())
        .unwrap_or_default();
    Json(out)
}

async fn post(
    State(st): State<Shared>,
    Path(ch): Path<String>,
    Json(new): Json<NewMsg>,
) -> Result<Json<Msg>, StatusCode> {
    let text = new.text.trim().to_string();
    let author = new.author.trim().to_string();
    if text.is_empty() || text.len() > 2000 || author.is_empty() || author.len() > 32 {
        return Err(StatusCode::BAD_REQUEST);
    }
    let mut st = st.lock().unwrap();
    st.next_id += 1;
    let msg = Msg { id: st.next_id, author, text };
    let v = st.channels.entry(ch).or_default();
    v.push(msg.clone());
    if v.len() > 500 {
        v.remove(0);
    }
    Ok(Json(msg))
}

#[tokio::main]
async fn main() {
    let state: Shared = Arc::new(Mutex::new(Store::default()));
    let app = Router::new()
        .route("/channels/:ch/messages", get(list).post(post))
        .layer(CorsLayer::permissive())
        .with_state(state);
    let addr = std::env::var("BIND").unwrap_or_else(|_| "0.0.0.0:8080".into());
    println!("listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
