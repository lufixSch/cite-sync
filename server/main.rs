use poem::{Route, listener::TcpListener};
use poem_openapi::{OpenApi, OpenApiService, payload::PlainText};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "citesync-server")]
#[command(
    version,
    about = "Server for organizing and sharing research papers and other citable sources"
)]
struct Args {
    #[arg(
        short,
        long,
        help = "URL to the API (make sure to change this when changing the port and using the API docs)",
        env = "CITESYNC_URL",
        default_value_t = String::from("http://127.0.0.1:3000")
    )]
    url: String,

    #[arg(
        short,
        long,
        help = "Disable the SwaggerUI API docs (/docs)",
        env = "CITESYNC_DISABLE_DOCS",
        default_value_t = false
    )]
    disable_docs: bool,

    #[arg(
        short,
        long,
        help = "The port on which the server listens for requests.",
        env = "CITESYNC_PORT",
        default_value_t = 3000
    )]
    port: u16,
}

struct App;

#[OpenApi]
impl App {
    #[oai(path = "/", method = "get")]
    async fn index(&self) -> PlainText<String> {
        PlainText("Welcome to the CiteSync API!".to_string())
    }
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let args = Args::parse();

    let api_service = OpenApiService::new(App, "CiteSync", "0.1.0").server(args.url);
    let docs = api_service.swagger_ui();

    let mut server = Route::new().nest("/", api_service);
    if !args.disable_docs {
        server = server.nest("docs", docs);
    }

    poem::Server::new(TcpListener::bind(format!("0.0.0.0:{}", args.port)))
        .run(server)
        .await
}
