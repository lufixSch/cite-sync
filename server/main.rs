use poem::{EndpointExt, Route, listener::TcpListener, middleware::Cors};
use poem_openapi::OpenApiService;

use clap::Parser;
use eyre::{Result, eyre};
use sqlx::SqlitePool;

mod items;
mod opds;
mod tags;
mod welcome;

/// Command line arguments for the CiteSync server.
#[derive(Parser, Debug)]
#[command(name = "citesync-server")]
#[command(
    version,
    about = "Server for organizing and sharing research papers and other citable sources"
)]
struct Args {
    /// URL to the API (make sure to change this when changing the port and using the API docs).
    #[arg(
        short,
        long,
        help = "URL to the API (make sure to change this when changing the port and using the API docs)",
        env = "CITESYNC_URL",
        default_value_t = String::from("http://127.0.0.1:3000")
    )]
    url: String,

    /// Disable the SwaggerUI API docs (/docs).
    #[arg(
        short,
        long,
        help = "Disable the SwaggerUI API docs (/docs)",
        env = "CITESYNC_DISABLE_DOCS",
        default_value_t = false
    )]
    disable_docs: bool,

    /// The port on which the server listens for requests.
    #[arg(
        short,
        long,
        help = "The port on which the server listens for requests.",
        env = "CITESYNC_PORT",
        default_value_t = 3000
    )]
    port: u16,

    /// Database URL/Connection string
    #[arg(
        long,
        help = "Database URL/Connection string",
        env = "CITESYNC_DATABASE",
        default_value_t = String::from("sqlite:citesync.db")
    )]
    database_url: String,

    /// Database URL/Connection string
    #[arg(
        long,
        help = "Path to CiteSync data",
        env = "CITESYNC_DATA_DIR",
        default_value_t = String::from("data")
    )]
    data_dir: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Create an OpenAPI service with the provided API and server URL.
    let api_service = OpenApiService::new(
        (welcome::Router, opds::Router, items::Router),
        "CiteSync",
        "0.1.0",
    )
    .server(args.url);

    let db = SqlitePool::connect(&args.database_url)
        .await
        .map_err(|e| eyre!(format!("Database connection failed with error: {e}")))?;

    // Generate SwaggerUI documentation for the API.
    let docs = api_service.swagger_ui();

    let mut server = Route::new().nest("/", api_service);
    if !args.disable_docs {
        server = server.nest("docs", docs);
    }

    // Start the server with CORS middleware enabled.
    poem::Server::new(TcpListener::bind(format!("0.0.0.0:{}", args.port)))
        .run(server.with(Cors::new()).data(db).data(args.data_dir))
        .await
        .map_err(|e| eyre!(format!("Server failed with error: {e}")))
}
