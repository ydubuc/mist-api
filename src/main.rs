#![allow(dead_code)]
// #![allow(unused_variables)]

use std::{env, net::SocketAddr, sync::Arc, time::Duration};

#[macro_use]
extern crate lazy_static;

use app::util::fcm::client::fcm_client::FcmClient;
use axum::{
    error_handling::HandleErrorLayer,
    http::Method,
    routing::{delete, get, patch, post},
    BoxError, Router,
};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tokio::sync::RwLock;
use tower::{buffer::BufferLayer, limit::RateLimitLayer, ServiceBuilder};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};

use crate::{
    app::{
        enums::api_status::ApiStatus,
        envy::Envy,
        errors::DefaultApiError,
        util::{fcm, janitor},
    },
    media::util::backblaze::b2::{self, b2::B2},
};

mod app;
mod auth;
mod blocks;
mod devices;
mod follows;
mod generate_media_requests;
mod mail;
mod media;
mod posts;
mod transactions;
mod users;
mod webhooks;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub client: reqwest::Client,
    pub fcm_client: Arc<RwLock<FcmClient>>,
    pub b2: Arc<RwLock<B2>>,
    pub api_state: Arc<ApiState>,
    pub envy: Envy,
}

#[derive(Clone)]
pub struct ApiState {
    pub api_status: Arc<RwLock<String>>,
}

#[tokio::main]
async fn main() {
    // environment
    let app_env = env::var("APP_ENV").unwrap_or("development".to_string());
    let _ = dotenvy::from_filename(format!(".env.{}", app_env));
    let envy = match envy::from_env::<Envy>() {
        Ok(config) => config,
        Err(e) => panic!("{:#?}", e),
    };

    // tracing
    let log_level = match app_env.as_ref() {
        "production" => "info",
        _ => "debug",
    };
    let log = format!("mist_api={}", log_level);

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            env::var("RUST_LOG").unwrap_or_else(|_| log.into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // properties
    let port = envy.port.to_owned().unwrap_or(3000);
    let role = envy.role.to_owned().unwrap_or("default".to_string());
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_headers(Any)
        .allow_methods([Method::POST, Method::GET, Method::PATCH, Method::DELETE]);
    let client = reqwest::Client::new();

    let fcm_client_email = envy.fcm_client_email.to_string();
    let fcm_private_key = envy.fcm_private_key.to_string();

    let mut fcm_client = FcmClient::new(
        fcm::client::config::Config::new(
            "mist-de479".to_string(),
            fcm_client_email,
            fcm_private_key,
        ),
        None,
    );
    fcm_client.login().await.expect("failed to login to fcm");

    tracing::info!("logged in to fcm");

    let pool = PgPoolOptions::new()
        .max_connections(33)
        .min_connections(1)
        .connect(&envy.database_url)
        .await
        .expect("failed to connect to database");

    tracing::info!("connected to database");

    let backblaze_key_id = envy.backblaze_key_id.to_string();
    let backblaze_app_key = envy.backblaze_app_key.to_string();
    let backblaze_bucket_id = envy.backblaze_bucket_id.to_string();

    let mut b2 = B2::new(b2::config::Config::new(backblaze_key_id, backblaze_app_key));
    b2.set_bucket_id(backblaze_bucket_id);
    b2.login(&client)
        .await
        .expect("failed to login to backblaze");

    tracing::info!("logged in to backblaze");

    let state = Arc::new(AppState {
        pool,
        client,
        fcm_client: Arc::new(RwLock::new(fcm_client)),
        b2: Arc::new(RwLock::new(b2)),
        api_state: Arc::new(ApiState {
            api_status: Arc::new(RwLock::new(ApiStatus::Online.value())),
        }),
        envy,
    });

    if role == "main" {
        janitor::spawn(state.clone());
        tracing::info!("janitor spawned");
    }

    // app
    let app = Router::new()
        .route("/", get(app::controller::get_root))
        .route("/status", get(app::controller::get_api_state))
        .route("/status", patch(app::controller::edit_api_state))
        // TRANSACTIONS
        .route(
            "/transactions",
            post(transactions::controller::handle_webhook),
        )
        // AUTH
        .route("/auth/register", post(auth::controller::register))
        .route("/auth/login", post(auth::controller::login))
        .route(
            "/auth/email",
            post(auth::controller::request_email_update_mail),
        )
        .route("/auth/email", patch(auth::controller::process_email_edit))
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
        .route("/auth/delete", post(auth::controller::delete_account))
        // DEVICES
        .route(
            "/devices/:id",
            patch(devices::controller::edit_device_by_id),
        )
        // USERS
        .route("/users", get(users::controller::get_users))
        .route("/users/me", get(users::controller::get_user_from_request))
        .route("/users/:id", get(users::controller::get_user_by_id))
        .route("/users/:id", patch(users::controller::edit_user_by_id))
        // POSTS
        .route("/posts", get(posts::controller::get_posts))
        .route("/posts/:id", get(posts::controller::get_post_by_id))
        .route("/posts/:id", patch(posts::controller::edit_post_by_id))
        .route(
            "/posts/:id/report",
            post(posts::controller::report_post_by_id),
        )
        .route("/posts/:id", delete(posts::controller::delete_post_by_id))
        // MEDIA
        .route("/media/generate", post(media::controller::generate_media))
        // WEBHOOKS
        .route(
            "/webhooks/modal",
            post(webhooks::modal::webhook::receive_webhook),
        )
        // .route("/media/import", post(media::controller::import_media))
        .route("/media", get(media::controller::get_media))
        .route("/media/:id", get(media::controller::get_media_by_id))
        .route("/media/:id", delete(media::controller::delete_media_by_id))
        // GENERATE_MEDIA_REQUESTS
        .route(
            "/generate-media-requests",
            get(generate_media_requests::controller::get_generate_media_requests),
        )
        // FOLLOWS
        .route("/follow/:id", post(follows::controller::follow))
        .route("/follows", get(follows::controller::get_follows))
        .route("/follow/:id", delete(follows::controller::unfollow))
        // BLOCKS
        .route("/block/:id", post(blocks::controller::block))
        .route("/blocks", get(blocks::controller::get_blocks))
        .route("/block/:id", delete(blocks::controller::unblock))
        // LAYERS
        .layer(cors)
        .layer(tower_http::limit::RequestBodyLimitLayer::new(2097152))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|_err: BoxError| async move {
                    DefaultApiError::InternalServerError.value();
                }))
                .layer(BufferLayer::new(1024))
                .layer(RateLimitLayer::new(5, Duration::from_secs(1))),
        )
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
