mod larkbot;

use axum::{
    extract::State,
    response::{Html, IntoResponse},
    routing::get,
    Json, Router,
};
use chrono::Local;
use reqwest::StatusCode;
use serde_json::json;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let bot = match larkbot::newbot() {
        Some(bot) => bot,
        None => return,
    };

    // build our application with a route
    let app = Router::new()
        .route("/notice", get(sample).post(notice))
        .route("/healthz", get(healthz))
        .fallback(handler_404)
        .with_state(bot);
    // run it
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn sample() -> Html<&'static str> {
    println!("{}: read sample", Local::now());
    Html(
        r#"<h1>Sample</h1><code>curl -X POST -H "Content-Type: application/json" -d '{"event": "New User","user":"wurui","event_time":"2023-02-16 11:05:10.651917 UTC", "description":"For testing"}' http://127.0.0.1:3000/notice"#,
    )
}

async fn notice(
    State(bot): State<Box<(dyn larkbot::Bot + Sync + Send + 'static)>>,
    Json(event): Json<larkbot::Event>,
) -> impl IntoResponse {
    println!("{}: {:?}", Local::now(), event);

    let result = bot.send(&event).await;

    (StatusCode::OK, Json(result))
}

async fn healthz() -> impl IntoResponse {
    println!("{}: health check", Local::now());
    (StatusCode::OK, Json(json!({"msg": "server alive"})))
}

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, Json(json!({"msg": "not found"})))
}
