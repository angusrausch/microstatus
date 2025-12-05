// === src/server.rs ===
use std::io::prelude::*;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream}; // Use tokio's async types
use tokio::io::{AsyncReadExt, AsyncWriteExt}; // For async read/write

// REMOVE ThreadPool, Worker, and Job structs

pub async fn run_server(port: u16, html_dir: Arc<PathBuf>) {
    let bind_addr: String = format!("0.0.0.0:{port}");
    
    // Use tokio::net::TcpListener for async listening
    let listener = match TcpListener::bind(&bind_addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind server to {bind_addr}: {e}");
            return;
        }
    };

    println!("Starting listener @ {bind_addr}");

    loop {
        // Accept connections asynchronously
        match listener.accept().await {
            Ok((stream, _)) => {
                let html_dir_clone = Arc::clone(&html_dir);
                // Spawn a new async task for each connection (non-blocking)
                tokio::spawn(async move {
                    handle_connection(stream, html_dir_clone).await;
                });
            }
            Err(e) => {
                eprintln!("Error accepting connection: {e}");
            }
        }
    }
}

// **MODIFIED:** Now an async function using tokio::io
async fn handle_connection(mut stream: TcpStream, html_dir: Arc<PathBuf>) {
    let mut buffer = [0; 1024];
    // Read the request data asynchronously (non-blocking)
    if let Err(e) = stream.read(&mut buffer).await {
        eprintln!("Error reading stream: {e}");
        return;
    }

    let request = String::from_utf8_lossy(&buffer[..]);
    let request_line = request.lines().next().unwrap_or("");

    let (status_line, html_contents) = if request_line == "GET / HTTP/1.1" {
        // Index page
        match open_html_file_async(html_dir.join("index.html")).await {
            Ok(contents) => ("HTTP/1.1 200 OK".to_string(), contents),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                ("HTTP/1.1 404 NOT FOUND".to_string(), open_html_file_async(html_dir.join("404.html")).await.unwrap_or_else(|_| "404 Not Found".to_string()))
            }
            Err(_) => ("HTTP/1.1 500 INTERNAL SERVER ERROR".to_string(), "500 Internal Server Error".to_string()),
        }
    } else if request_line.split_whitespace().next() != Some("GET") {
        // If not GET
        ("HTTP/1.1 405 METHOD NOT ALLOWED".to_string(), "405 Method Not Allowed".to_string())
    } else if let Some(path) = request_line.split_whitespace().nth(1) {
        // Default (handling /file.html requests)
        let file_path = html_dir.join(&path[1..]); // remove leading '/'
        match open_html_file_async(file_path).await {
            Ok(contents) => ("HTTP/1.1 200 OK".to_string(), contents),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                ("HTTP/1.1 404 NOT FOUND".to_string(), open_html_file_async(html_dir.join("404.html")).await.unwrap_or_else(|_| "404 Not Found".to_string()))
            }
            Err(_) => ("HTTP/1.1 500 INTERNAL SERVER ERROR".to_string(), "500 Internal Server Error".to_string()),
        }
    } else {
        // Malformed request
        ("HTTP/1.1 400 BAD REQUEST".to_string(), "400 Bad Request".to_string())
    };

    let length = html_contents.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{html_contents}");

    // Write the response data asynchronously (non-blocking)
    if let Err(e) = stream.write_all(response.as_bytes()).await {
        eprintln!("Error writing stream: {e}");
    }
}

// **MODIFIED:** New async function using tokio::fs
async fn open_html_file_async(path: PathBuf) -> Result<String, std::io::Error> {
    // Check for existence is optional here, as tokio::fs::read_to_string will return NotFound
    // if the file doesn't exist, but we keep it for now.
    if !path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("File '{}' does not exist", path.display()),
        ));
    }
    
    // Use tokio::fs::read_to_string for non-blocking I/O
    tokio::fs::read_to_string(&path).await
}