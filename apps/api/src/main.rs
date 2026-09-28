use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use misty_api::{config::Config, http_api::{self, ApiState}};
use serde::Serialize;
use sqlx::{PgPool, postgres::PgPoolOptions};
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};
use tracing::{error, info};

#[derive(Clone)]
struct AppState {
    database: PgPool,
}

#[derive(Serialize)]
struct Health {
    status: &'static str,
    service: &'static str,
}

async fn health() -> Json<Health> {
    Json(Health {
        status: "ok",
        service: "misty-api",
    })
}

async fn ready(State(state): State<AppState>) -> Result<Json<Health>, StatusCode> {
    sqlx::query("SELECT 1")
        .execute(&state.database)
        .await
        .map_err(|error| {
            error!(%error, "database readiness check failed");
            StatusCode::SERVICE_UNAVAILABLE
        })?;

    Ok(Json(Health {
        status: "ready",
        service: "misty-api",
    }))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "misty_api=info,tower_http=info".into()),
        )
        .init();

    let config = Config::from_env().unwrap_or_else(|error| {
        error!(%error, "invalid configuration");
        std::process::exit(2);
    });

    let database = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
        .unwrap_or_else(|error| {
            error!(%error, "database connection failed");
            std::process::exit(3);
        });

    sqlx::migrate!("./migrations")
        .run(&database)
        .await
        .unwrap_or_else(|error| {
            error!(%error, "database migration failed");
            std::process::exit(4);
        });

    let state = AppState {
        database: database.clone(),
    };
    let product_api = http_api::router(ApiState {
        database: database.clone(),
    });
    let app = Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .with_state(state)
        .merge(product_api)
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(config.bind_addr)
        .await
        .expect("failed to bind API listener");
    info!(addr = %config.bind_addr, "misty API listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("API server failed");
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}
