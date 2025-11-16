mod cli;

fn main() {
    let cli_flag = std::env::args().any(|arg| arg == "--cli");

    if cli_flag {
        let _ = cli::cli_check();
    }
}
