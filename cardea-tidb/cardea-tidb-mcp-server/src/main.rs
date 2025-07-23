mod tidb;

use anyhow::{anyhow, bail};
use clap::{Parser, ValueEnum};
use mysql::*;
use once_cell::sync::OnceCell;
use regex::Regex;
use rmcp::transport::{
    sse_server::SseServer,
    streamable_http_server::{StreamableHttpService, session::local::LocalSessionManager},
};
use rustls::crypto::{CryptoProvider, ring::default_provider};
use std::{env, path::PathBuf};
use tidb::TidbServer;
use tokio::sync::RwLock as TokioRwLock;
use tracing::{error, info};
use tracing_subscriber::{self, layer::SubscriberExt, util::SubscriberInitExt};

pub static SSL_CA_PATH: OnceCell<PathBuf> = OnceCell::new();
pub static TIDB_ACCESS_CONFIG: OnceCell<TokioRwLock<TidbAccessConfig>> = OnceCell::new();

const DEFAULT_SOCKET_ADDR: &str = "127.0.0.1:8007";

#[derive(Parser, Debug)]
#[command(author, version, about = "Cardea TiDB MCP server")]
struct Args {
    /// Path to the SSL CA certificate. On macOS, this is typically
    /// `/etc/ssl/cert.pem`. On Debian/Ubuntu/Arch Linux, it's typically
    /// `/etc/ssl/certs/ca-certificates.crt`.
    #[arg(long, required = true)]
    ssl_ca: PathBuf,
    /// Socket address to bind to
    #[arg(short, long, default_value = DEFAULT_SOCKET_ADDR)]
    socket_addr: String,
    /// Transport type to use (sse or stream-http)
    #[arg(short, long, value_enum, default_value = "stream-http")]
    transport: TransportType,
    /// Table name
    #[arg(long, required = true)]
    table_name: String,
    /// Maximum number of query results to return
    #[arg(long, default_value = "10")]
    limit: u64,
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
                .unwrap_or_else(|_| format!("info,{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer().with_line_number(true))
        .init();

    CryptoProvider::install_default(default_provider()).map_err(|e| {
        let err_msg = format!("Failed to install default crypto provider: {e:?}");
        error!("{}", err_msg);
        anyhow!(err_msg)
    })?;

    let args = Args::parse();

    // parse connection string
    let (username, password, host, port, database) = match env::var("TIDB_CONNECTION") {
        Ok(ref conn) => {
            parse_tidb_conn_str(conn.as_str()).ok_or_else(|| anyhow!(
                "Invalid connection string! The pattern should be `mysql://<USERNAME>:<PASSWORD>@<HOST>:<PORT>/<DATABASE>`"
            ))?
        }
        Err(e) => {
            let error_message = format!("Failed to get TIDB_CONNECTION: {e}");
            error!(error_message);
            bail!(error_message);
        }
    };

    // convert port to u16
    let port = port.parse::<u16>().map_err(|e| {
        let error_message = format!("Failed to parse TIDB_PORT: {e}");
        error!(error_message);
        anyhow!(error_message)
    })?;

    // create connection options
    info!("Creating connection options for TiDB Cloud...");
    let opts = OptsBuilder::new()
        .ip_or_hostname(Some(host))
        .tcp_port(port)
        .user(Some(username))
        .pass(Some(password))
        .db_name(Some(database.clone()))
        .ssl_opts(Some(
            SslOpts::default().with_root_cert_path(Some(args.ssl_ca)),
        ));

    // create connection pool
    info!("Creating connection pool...");
    let pool = Pool::new(opts).map_err(|e| {
        let error_message = format!("Failed to create connection pool: {e}");
        error!(error_message);
        anyhow!(error_message)
    })?;

    let config = TidbAccessConfig {
        pool,
        database,
        table_name: args.table_name,
        limit: args.limit,
    };

    TIDB_ACCESS_CONFIG
        .set(TokioRwLock::new(config))
        .map_err(|_| anyhow::anyhow!("Failed to set TIDB_ACCESS_CONFIG"))?;

    info!("Starting Cardea TiDB MCP server on {}", args.socket_addr);

    match args.transport {
        TransportType::StreamHttp => {
            let service = StreamableHttpService::new(
                || Ok(TidbServer::new()),
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
                .with_service(TidbServer::new);

            tokio::signal::ctrl_c().await?;
            ct.cancel();
        }
    }

    Ok(())
}

#[derive(Debug, Clone)]
pub struct TidbAccessConfig {
    pub pool: Pool,
    pub database: String,
    pub table_name: String,
    pub limit: u64,
}

fn parse_tidb_conn_str(conn_str: &str) -> Option<(String, String, String, String, String)> {
    let re = Regex::new(r"^mysql://([^:]+):([^@]+)@([^:/]+):(\d+)/(.+)$").unwrap();
    if let Some(caps) = re.captures(conn_str) {
        let username = caps.get(1)?.as_str().to_string();
        let password = caps.get(2)?.as_str().to_string();
        let host = caps.get(3)?.as_str().to_string();
        let port = caps.get(4)?.as_str().to_string();
        let database = caps.get(5)?.as_str().to_string();
        Some((username, password, host, port, database))
    } else {
        None
    }
}
