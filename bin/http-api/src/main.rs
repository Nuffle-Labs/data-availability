use std::{net::SocketAddr, path::PathBuf, sync::Arc};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing, Router,
};
use clap::Parser;
use near_da_http_api_data::{BlobRequest, ConfigureClientRequest, SubmitRequest};
use near_da_rpc::{
    near::{config::Config, Client},
    DataAvailability,
};
use tokio::sync::RwLock;
use tower_http::trace::{self, TraceLayer};
use tracing::Level;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct CliArgs {
    /// Run server on port.
    #[arg(short, long, default_value_t = 5888)]
    port: u16,

    /// Path to the client configuration. If not specified, the client can be
    /// configured via PUT /config after starting the server.
    #[arg(short, long)]
    config: Option<PathBuf>,
}

struct AppState {
    client: Option<Client>,
}

fn config_request_to_config(request: ConfigureClientRequest) -> Result<Config, anyhow::Error> {
    Ok(Config {
        key: near_da_rpc::near::config::KeyType::SecretKey(request.account_id, request.secret_key),
        contract: request.contract_id,
        network: request
            .network
            .as_str()
            .try_into()
            .map_err(|e: String| anyhow::anyhow!(e))?,
        namespace: near_da_primitives::Namespace::new(
            request.namespace.version,
            request.namespace.id,
        ),
    })
}

async fn configure_client(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(request): Json<ConfigureClientRequest>,
) -> anyhow::Result<(), AppError> {
    match state.write().await.client {
        Some(_) => Err(anyhow::anyhow!("client has already been configured").into()),
        ref mut c @ None => {
            tracing::info!("client configuration set: {:?}", request);
            *c = Some(Client::new(&config_request_to_config(request)?));
            Ok(())
        }
    }
}

async fn get_blob(
    State(state): State<Arc<RwLock<AppState>>>,
    Query(request): Query<BlobRequest>,
) -> anyhow::Result<Json<near_da_http_api_data::Blob>, AppError> {
    let app_state = state.read().await;
    let client = app_state
        .client
        .as_ref()
        .ok_or(anyhow::anyhow!("client is not configured"))?;

    let blob = client
        .get(
            request
                .transaction_id
                .parse()
                .map_err(|e| anyhow::anyhow!("invalid transaction id: {}", e))?,
        )
        .await
        .map_err(|e| anyhow::anyhow!("failed to get blob: {}", e))?
        .0;

    let blob = near_da_http_api_data::Blob {
        namespace: near_da_http_api_data::Namespace {
            version: blob.namespace.version,
            id: blob.namespace.id,
        },
        share_version: blob.share_version,
        commitment: blob.commitment,
        data: blob.data,
    };

    Ok(Json(blob))
}

async fn submit_blob(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(request): Json<SubmitRequest>,
) -> anyhow::Result<String, AppError> {
    let app_state = state.read().await;
    let client = app_state
        .client
        .as_ref()
        .ok_or(anyhow::anyhow!("client is not configured"))?;

    let result = client
        .submit(&[near_da_primitives::Blob::new_v0(
            client.config.namespace,
            request.data,
        )])
        .await
        .map_err(|e| anyhow::anyhow!("failed to submit blobs: {}", e))?;

    Ok(result.0)
}

// https://github.com/tokio-rs/axum/blob/d7258bf009194cf2f242694e673759d1dbf8cfc0/examples/anyhow-error-response/src/main.rs#L34-L57
struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("something went wrong: {}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

#[tokio::main]
async fn main() {
    let args = CliArgs::parse();

    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    let mut state = AppState { client: None };

    if let Some(path) = args.config {
        let file_contents = tokio::fs::read_to_string(path).await.unwrap();
        let config_parse = serde_json::from_str::<ConfigureClientRequest>(&file_contents)
            .unwrap_or_else(|e| panic!("failed to parse config: {}", e));
        state.client = Some(Client::new(
            &config_request_to_config(config_parse).unwrap(),
        ));
    }

    let state = Arc::new(RwLock::new(state));

    let router = Router::new()
        .route("/health", routing::get(|| async { "" }))
        .route("/configure", routing::put(configure_client))
        .route("/blob", routing::get(get_blob))
        .route("/blob", routing::post(submit_blob))
        .with_state(state)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        );

    let addr = SocketAddr::from(([0; 4], args.port));
    tracing::info!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .unwrap();
}
