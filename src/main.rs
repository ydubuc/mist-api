#![allow(dead_code)]
#![allow(unused_variables)]

use std::{env, net::SocketAddr};

#[macro_use]
extern crate lazy_static;

use axum::{
    http::Method,
    http::header::{CONTENT_TYPE, AUTHORIZATION},
    routing::{delete, get, patch, post},
    Router,
};
use b2_backblaze::{Config, B2};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tower_http::cors::{Any, CorsLayer};

use crate::app::env::Envy;

mod app;
mod auth;
mod devices;
mod generate_media_requests;
mod mail;
mod media;
mod posts;
mod users;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub b2: B2,
    pub envy: Envy,
}

#[tokio::main]
async fn main() {
    // tracing
    tracing_subscriber::fmt::init();

    // environment
    let app_env = env::var("APP_ENV").unwrap_or("development".to_string());
    let _ = dotenvy::from_filename(format!(".env.{}", app_env));
    let envy = match envy::from_env::<Envy>() {
        Ok(config) => config,
        Err(e) => panic!("{:#?}", e),
    };

    // properties
    let port = envy.port.to_owned();
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_headers([CONTENT_TYPE, AUTHORIZATION])
        .allow_methods([Method::POST, Method::GET, Method::PATCH, Method::DELETE]);
    // let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);

    let database_url = envy.database_url.to_string();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("failed to connect to database");

    println!("connected to db");

    let backblaze_key_id = envy.backblaze_key_id.to_string();
    let backblaze_app_key = envy.backblaze_app_key.to_string();
    let backblaze_bucket_id = envy.backblaze_bucket_id.to_string();
    let mut b2 = B2::new(Config::new(backblaze_key_id, backblaze_app_key));
    b2.set_bucket_id(backblaze_bucket_id);
    b2.login().await.expect("failed to login to backblaze");

    println!("logged in to backblaze");

    let state = AppState { pool, b2, envy };

    // app
    let app = Router::with_state(state)
        .route("/", get(app::controller::get_root))
        // auth
        .route("/auth/register", post(auth::controller::register))
        .route("/auth/login", post(auth::controller::login))
        .route(
            "/auth/email",
            post(auth::controller::request_email_update_mail),
        )
        .route(
            "/auth/email",
            patch(auth::controller::process_email_edit),
        )
        .route(
            "/auth/password",
            post(auth::controller::request_password_update_mail),
        )
        .route(
            "/auth/password",
            patch(auth::controller::process_password_edit),
        )
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

    let addr = SocketAddr::from(([0, 0, 0, 0], port.unwrap_or(3000)));
    println!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
