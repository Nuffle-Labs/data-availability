use axum::Json;
use clap::{command, Parser, Subcommand};
use std::env::args;
use std::fmt::Display as FmtDisplay;
use std::path::Display;

use clap;
use near_da_http_api_data::ConfigureClientRequest;
use near_da_rpc::near::config::Config;
use near_da_rpc::near::Client;
use near_da_rpc::{Blob, DataAvailability};
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[clap(
        short = 'c',
        long = "config",
        help = "Path to the client configuration. If not specified, the client can be configured via PUT /config after starting the server."
    )]
    config: Option<String>,
    #[command(subcommand)]
    command: Commands,
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

#[derive(Parser, Debug)]
enum Commands {
    Submit(SubmitArgs),
    Get(GetArgs),
}

#[derive(Parser, Debug)]
struct SubmitArgs {
    pub data: Vec<u8>,
}

#[derive(Parser, Debug)]
struct GetArgs {
    pub transaction_id: String,
}

struct AppError(anyhow::Error);

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

impl FmtDisplay for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
async fn submit_blob(state: AppState, submit_args: SubmitArgs) -> anyhow::Result<String, AppError> {
    let client = state
        .client
        .as_ref()
        .ok_or(anyhow::anyhow!("client is not configured"))?;

    let result = client
        .submit(
            (&[near_da_primitives::Blob::new_v0(
                client.config.namespace,
                submit_args.data,
            )]),
        )
        .await
        .map_err(|e| anyhow::anyhow!("failed to submit blobs: {}", e))?;

    Ok(result.0)
}

async fn get_blob(
    state: AppState,
    get_args: GetArgs,
) -> anyhow::Result<Json<near_da_http_api_data::Blob>, AppError> {
    let client = state
        .client
        .as_ref()
        .ok_or(anyhow::anyhow!("client is not configured"))?;

    let blob = client
        .get(
            get_args
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

#[tokio::main]
async fn main() {
    let args = Args::parse();

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

    match args.command {
        Commands::Submit(submit) => match submit_blob(state, submit).await {
            Ok(result) => println!("{}", result),
            Err(e) => println!("{}", e),
        },
        Commands::Get(get) => match get_blob(state, get).await {
            Ok(blob) => println!("{:?}", blob),
            Err(e) => println!("{}", e),
        },
    };
}
