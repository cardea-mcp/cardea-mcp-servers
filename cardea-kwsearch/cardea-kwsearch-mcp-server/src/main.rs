mod search;

use clap::{Parser, ValueEnum};
use once_cell::sync::OnceCell;
use rmcp::transport::{
    sse_server::SseServer,
    streamable_http_server::{StreamableHttpService, session::local::LocalSessionManager},
};
use search::{ConnectionConfig, KeywordSearchServer};
use tokio::sync::RwLock as TokioRwLock;
use tracing_subscriber::{self, layer::SubscriberExt, util::SubscriberInitExt};

const DEFAULT_SOCKET_ADDR: &str = "127.0.0.1:8005";

static CONNECTION_CONFIG: OnceCell<TokioRwLock<ConnectionConfig>> = OnceCell::new();

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The base URL of the kw-search-server
    #[arg(long, default_value = "http://127.0.0.1:12306")]
    base_url: String,
    /// Socket address to bind to
    #[arg(short, long, default_value = DEFAULT_SOCKET_ADDR)]
    socket_addr: String,
    /// Transport type to use
    #[arg(short, long, value_enum, default_value = "stream-http")]
    transport: TransportType,
    /// Index to search
    #[arg(long)]
    index: String,
    /// Maximum number of query results to return
    #[arg(long, default_value = "10")]
    limit: usize,
}

#[derive(Debug, Clone, ValueEnum)]
enum TransportType {
    Sse,
    StreamHttp,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug".to_string().into()),
        )
        .with(tracing_subscriber::fmt::layer().with_line_number(true))
        .init();

    let args = Args::parse();

    let connection_config = ConnectionConfig {
        base_url: args.base_url,
        api_key: None,
        index: args.index,
        limit: args.limit,
    };

    CONNECTION_CONFIG
        .set(TokioRwLock::new(connection_config))
        .map_err(|_| anyhow::anyhow!("Failed to set CONNECTION_CONFIG"))?;

    tracing::info!(
        "Starting Cardea KeywordSearch MCP server on {}",
        args.socket_addr
    );

    match args.transport {
        TransportType::StreamHttp => {
            let service = StreamableHttpService::new(
                || Ok(KeywordSearchServer::new()),
                LocalSessionManager::default().into(),
                Default::default(),
            );

            let router = axum::Router::new().nest_service("/mcp", service);
            let tcp_listener = tokio::net::TcpListener::bind(args.socket_addr).await?;
            let _ = axum::serve(tcp_listener, router)
                .with_graceful_shutdown(async { tokio::signal::ctrl_c().await.unwrap() })
                .await;
        }
        TransportType::Sse => {
            let ct = SseServer::serve(args.socket_addr.parse()?)
                .await?
                .with_service(KeywordSearchServer::new);

            tokio::signal::ctrl_c().await?;
            ct.cancel();
        }
    }

    Ok(())
}
