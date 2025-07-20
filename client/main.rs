use std::path::Path;

use crate::config::CiteSyncClientConfig;
use clap::Parser;
use tracing::error;
use tracing_subscriber;
use biblatex::Bibliography;

use libcitesync::bib;

mod config;

/// CLI arguments for the CiteSync client
#[derive(Parser, Debug)]
#[command(name = "citesync-client")]
#[command(
    version,
    about = "Client for syncing your local bibliography to a CiteSync server"
)]
struct Args {}

fn main() {
    tracing_subscriber::fmt::init();

    let cs_config = CiteSyncClientConfig::load().unwrap_or_else(|_| CiteSyncClientConfig::default());
    let args = Args::parse();

    let bibs: Vec<Bibliography> = cs_config.bib_files.into_iter().map(|f| {
        let path = Path::new(&f);
        let b = bib::load_bibtex(&path);
        if b.is_err() {
            error!("Failed to load '{}'! Skipping File...", f)
        }

        b
    }).flatten().collect();

    // let items = bibs.into_iter().map(|b| {
    // }).flatten().collect();
}
