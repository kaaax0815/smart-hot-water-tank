use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use influxdb::{Client, InfluxDbWriteable};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
struct StateService {
    client: Client,
}

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let database_user = std::env::var("DATABASE_USER").expect("DATABASE_USER must be set");
    let database_password = std::env::var("DATABASE_PASS").expect("DATABASE_PASS must be set");
    let client = Client::new(database_url, "db0").with_auth(database_user, database_password);

    let state = StateService { client };

    let app = Router::new()
        .route("/ping", get(pingpong))
        .route("/create", post(create_entry))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn pingpong() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(StatusResponse {
            message: "Pong!".to_string(),
        }),
    )
}

async fn create_entry(
    State(state_service): State<StateService>,
    Json(payload): Json<CreateEntry>,
) -> impl IntoResponse {
    tracing::info!("creating entry: {:?}", payload);

    let entry = TempReading {
        time: Utc::now(),
        temp: payload.temp,
    }
    .into_query("temp");

    match state_service.client.query(entry).await {
        Ok(result) => (StatusCode::OK, Json(StatusResponse { message: result })),
        Err(e) => {
            tracing::error!("failed to write entry: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(StatusResponse {
                    message: "failed to write entry".to_string(),
                }),
            )
        }
    }
}

#[derive(Deserialize, Debug)]
struct CreateEntry {
    temp: i32,
}

#[derive(InfluxDbWriteable)]
struct TempReading {
    time: DateTime<Utc>,
    temp: i32,
}

#[derive(Serialize, Debug)]
struct StatusResponse {
    message: String,
}
