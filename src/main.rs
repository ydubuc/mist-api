#![allow(dead_code)]
#![allow(unused_variables)]

use std::net::SocketAddr;

use axum::{
    routing::{delete, get, patch, post},
    Router,
};
use b2_backblaze::{Config, B2};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tower_http::cors::{Any, CorsLayer};

use crate::app::env::Env;

mod app;
mod auth;
mod devices;
mod generate_media_requests;
mod media;
mod posts;
mod users;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub b2: B2,
}

#[tokio::main]
async fn main() {
    // tracing
    tracing_subscriber::fmt::init();

    // environment
    if let Err(e) = dotenvy::from_filename(".env.development") {
        tracing::error!(%e);
    }

    let port: u16 = match std::env::var(Env::PORT) {
        Ok(port) => port.parse().expect("env: PORT is not a number"),
        Err(_) => 3000,
    };
    let db_url = std::env::var(Env::DATABASE_URL).expect("env: DATABASE_URL missing");
    let b2_id = std::env::var(Env::BACKBLAZE_KEY_ID).expect("env: BACKBLAZE_KEY_ID missing");
    let b2_key = std::env::var(Env::BACKBLAZE_APP_KEY).expect("env: BACKBLAZE_KEY_ID missing");
    let b2_bucket_id =
        std::env::var(Env::BACKBLAZE_BUCKET_ID).expect("env: BACKBLAZE_BUCKET_ID missing");

    println!("loaded env");

    // properties
    let cors = CorsLayer::new().allow_origin(Any);

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("failed to connect to database");

    println!("connected to db");

    let mut b2 = B2::new(Config::new(b2_id, b2_key));
    b2.set_bucket_id(b2_bucket_id);
    b2.login().await.expect("");

    println!("logged in to backblaze");

    let state = AppState { pool, b2 };

    // app
    let app = Router::with_state(state)
        .route("/", get(app::controller::get_root))
        // auth
        .route("/auth/register", post(auth::controller::register))
        .route("/auth/login", post(auth::controller::login))
        .route("/auth/refresh", post(auth::controller::refresh))
        .route("/auth/devices", get(auth::controller::get_devices))
        .route("/auth/logout", post(auth::controller::logout))
        // devices
        .route(
            "/devices/:id",
            patch(devices::controller::edit_device_by_id),
        )
        // users
        .route("/users", get(users::controller::get_users))
        .route("/users/me", get(users::controller::get_user_from_request))
        .route("/users/:id", get(users::controller::get_user_by_id))
        .route("/users/:id", patch(users::controller::edit_user_by_id))
        .route("/users/:id", delete(users::controller::delete_user_by_id))
        // posts
        .route("/posts", post(posts::controller::create_post))
        .route("/posts", get(posts::controller::get_posts))
        .route("/posts/:id", get(posts::controller::get_post_by_id))
        .route("/posts/:id", patch(posts::controller::edit_post_by_id))
        .route("/posts/:id", delete(posts::controller::delete_post_by_id))
        // media
        .route("/media/generate", post(media::controller::generate_media))
        .route("/media/import", post(media::controller::import_media))
        .route("/media", get(media::controller::get_media))
        .route("/media/:id", get(media::controller::get_media_by_id))
        .route("/media/:id", delete(media::controller::delete_media_by_id))
        // generate_media_requests
        .route(
            "/generate-media-requests",
            get(generate_media_requests::controller::get_generate_media_requests),
        )
        // layers
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
