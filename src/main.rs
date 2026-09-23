use std::{env, error::Error, io};
use tower_http::cors::CorsLayer;
use axum::{
    Router,
    http::{HeaderValue, Method},
    routing::{get, post},
};
use mongodb::{
    Client, IndexModel,
    bson::{Document, doc},
    options::{ClientOptions, IndexOptions},
};
use tower_http::services::ServeDir;

use crate::models::AppState;

mod models;
mod routes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();

    let mongodb_uri = required_env("MONGODB_URI")?;
    let database_name = required_env("DATABASE_NAME")?;
    let jwt_secret = required_env("JWT_SECRET")?;
    if jwt_secret.len() < 32 {
        return Err(configuration_error(
            "JWT_SECRET must be at least 32 characters long.",
        ));
    }

    let server_port = env::var("SERVER_PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .map_err(|_| configuration_error("SERVER_PORT must be a valid port number."))?;
    let cookie_secure = env::var("COOKIE_SECURE")
        .map(|value| value.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    println!("Connecting to MongoDB...");
    let client_options = ClientOptions::parse(&mongodb_uri).await?;
    let client = Client::with_options(client_options)?;
    client
        .database("admin")
        .run_command(doc! { "ping": 1 })
        .await?;

    let database = client.database(&database_name);
    let users = database.collection::<Document>("users");
    let email_index = IndexModel::builder()
        .keys(doc! { "email": 1 })
        .options(IndexOptions::builder().unique(true).build())
        .build();
    users.create_index(email_index).await?;

    let state = AppState {
        database,
        jwt_secret,
        cookie_secure,
    };
    let cors = CorsLayer::new()
    .allow_origin("http://127.0.0.1:5500".parse::<HeaderValue>().unwrap())
    .allow_methods([Method::GET, Method::POST])
    .allow_headers(["content-type".parse().unwrap()])
    .allow_credentials(true);

    let app = Router::new()
        .route("/api/hello", get(hello))
        .route("/signup", post(routes::signup))
        .route("/login", post(routes::login))
        .route("/logout", post(routes::logout))
        .route("/profile", get(routes::profile))
        .route("/users", get(routes::users))
        .layer(cors)
        .fallback_service(ServeDir::new("frontend"))
        .with_state(state);

    let address = format!("127.0.0.1:{server_port}");
    let listener = tokio::net::TcpListener::bind(&address).await?;
    println!("Server running at http://localhost:{server_port}");

    axum::serve(listener, app).await?;
    Ok(())
}

fn required_env(name: &str) -> Result<String, Box<dyn Error>> {
    env::var(name).map_err(|_| configuration_error(&format!("{name} is missing.")))
}

fn configuration_error(message: &str) -> Box<dyn Error> {
    io::Error::new(io::ErrorKind::InvalidInput, message).into()
}

async fn hello() -> &'static str {
    "Hello from Rust API!"
}
