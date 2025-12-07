use dotenvy::dotenv;
use std::env;
use std::path::PathBuf;
use std::sync::Arc;

mod cli;
mod web;
mod server;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let frequency: u16 = env::var("FREQUENCY")
        .unwrap_or_else(|_| "30".to_string()).parse::<u16>().expect("Failed to parse string as u16");
    let check_file: String = env::var("CHECK_FILE")
        .unwrap_or_else(|_| "demo.yaml".to_string());
    let html_output_dir: Arc<PathBuf> = Arc::new(
        PathBuf::from(
            env::var("HTML_OUTPUT_DIR")
                .unwrap_or_else(|_| "output".to_string())
        )
    );
    let webserver: u16 = env::var("WEBSERVER_PORT")
        .unwrap_or_else(|_| "0".to_string()).parse::<u16>().expect("Failed to parse string as u16");
    let max_history: u32 = env::var("MAX_HISTORY")
        .unwrap_or_else(|_| "2,880".to_string()).parse::<u32>().expect("Failed to parse string as u32");

    let cli_flag = std::env::args().any(|arg| arg == "--cli");

    if cli_flag {
        let _ = cli::cli_check().await;
    } else {
        let _ = web::generate(frequency, check_file, html_output_dir, webserver, max_history).await;
    }
}
