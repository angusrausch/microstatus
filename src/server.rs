use std::net::{TcpListener, TcpStream};
use std::io::{BufReader, prelude::*, ErrorKind::*};
use std::fs::read_to_string;
use std::path::{Path, PathBuf};



pub async fn run_server(port: u16, html_dir: String) {
    let bind_addr: String = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&bind_addr).unwrap();

    println!("Starting listener @ {bind_addr}");
    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream, html_dir.clone());
    }
}

fn handle_connection(mut stream: TcpStream, html_dir: String) {
    let buf_reader = BufReader::new(&stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let html_dir: &Path = Path::new(&html_dir); // future: accept Path directly

    let (status_line, html_contents) = if request_line == "GET / HTTP/1.1" {
        // Index page
        match open_html_file(html_dir.join("index.html")) {
            Ok(contents) => ("HTTP/1.1 200 OK".to_string(), contents),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                ("HTTP/1.1 404 NOT FOUND".to_string(), open_html_file(html_dir.join("404.html")).unwrap_or_else(|_| "404 Not Found".to_string()))
            }
            Err(_) => ("HTTP/1.1 500 INTERNAL SERVER ERROR".to_string(), "500 Internal Server Error".to_string()),
        }
    } else if request_line.split_whitespace().nth(0) != Some("GET") {
        // If not GET
        ("HTTP/1.1 405 METHOD NOT ALLOWED".to_string(), "405 Method Not Allowed".to_string())
    } else if let Some(path) = request_line.split_whitespace().nth(1) {
        // Default
        let file_path = html_dir.join(&path[1..]); // remove leading '/'
        match open_html_file(file_path) {
            Ok(contents) => ("HTTP/1.1 200 OK".to_string(), contents),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                ("HTTP/1.1 404 NOT FOUND".to_string(), open_html_file(html_dir.join("404.html")).unwrap_or_else(|_| "404 Not Found".to_string()))
            }
            Err(_) => ("HTTP/1.1 500 INTERNAL SERVER ERROR".to_string(), "500 Internal Server Error".to_string()),
        }
    } else {
        // Malformed request
        ("HTTP/1.1 400 BAD REQUEST".to_string(), "400 Bad Request".to_string())
    };

    let length = html_contents.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{html_contents}");

    stream.write_all(response.as_bytes()).unwrap();
}

fn open_html_file(path: PathBuf) -> Result<String, std::io::Error> {
    if !path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("File '{}' does not exist", path.display()),
        ));
    }

    let html_contents = std::fs::read_to_string(&path)?;
    Ok(html_contents)
}

