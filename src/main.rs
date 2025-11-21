use dotenvy::dotenv;
use std::env;

mod cli;
mod web;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let frequency: u16 = env::var("FREQUENCY")
        .unwrap_or_else(|_| "30".to_string()).parse::<u16>().expect("Failed to parse string as u16");
    let check_file: String = env::var("CHECK_FILE")
        .unwrap_or_else(|_| "demo.yaml".to_string());
    let html_output_dir: String = env::var("HTML_OUTPUT_DIR")
        .unwrap_or_else(|_| "output".to_string());

    let cli_flag = std::env::args().any(|arg| arg == "--cli");

    if cli_flag {
        let _ = cli::cli_check();
    } else {
        web::generate(frequency, check_file, html_output_dir).await;
    }
}
