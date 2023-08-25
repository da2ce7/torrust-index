//! Import Tracker Statistics command.
//!
//! It imports the number of seeders and leechers for all torrent from the linked tracker.
//!
//! You can execute it with: `cargo run --bin importer`
use torrust_index_backend::console::commands::import_tracker_statistics::run_importer;

#[tokio::main]
async fn main() {
    run_importer().await;
}
