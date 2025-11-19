mod cli;
mod web;

#[tokio::main]
async fn main() {
    let cli_flag = std::env::args().any(|arg| arg == "--cli");

    if cli_flag {
        let _ = cli::cli_check();
    } else {
        web::generate(5, "demo.yaml").await;
    }
}
