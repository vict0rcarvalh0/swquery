mod db;
mod models;
mod routes;
mod middlewares;

use {
    axum::{
        http::Method,
        routing::{get, post},
        Router,
    },
    db::connect,
    dotenvy::dotenv,
    routes::{
        agent::{generate_query, generate_report},
        chatbot::{chatbot_interact, get_chat_by_id, get_chats_for_user},
        credits::{buy_credits, refund_credits},
        users::{create_user, get_user_by_pubkey, get_users},
    },
    // middlewares::rate_limiter::{RateLimiter, rate_limit},
    // std::{
    //     sync::{Arc, Mutex},
    //     time::Duration,
    // },
    tower_http::cors::{Any, CorsLayer},
    // tower::ServiceBuilder,
};

pub const AGENT_API_URL: &str = "http://localhost:8000";

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let pool = connect().await;

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    // let rate_limiter = Arc::new(Mutex::new(RateLimiter::new(5, Duration::from_secs(10))));

    let agent_router = Router::new()
        .route("/generate-query", post(generate_query))
        .route("/generate-report", post(generate_report));
    let chatbot_router = Router::new()
        .route("/interact", post(chatbot_interact))
        .route("/chats", get(get_chats_for_user))
        .route("/chats/:id", get(get_chat_by_id));

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/users", get(get_users).post(create_user))
        .route("/users/:pubkey", get(get_user_by_pubkey))
        .route("/credits/buy", post(buy_credits))
        .route("/credits/refund", post(refund_credits))
        .nest("/agent", agent_router)
        .nest("/chatbot", chatbot_router)
        .with_state(pool)
        .layer(cors);
        // .layer(
        //     ServiceBuilder::new()
        //         .layer(axum::middleware::from_fn_with_state(rate_limiter.clone(), rate_limit))
        // );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:5500").await.unwrap();

    println!("Listening on: http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
