use std::net::{TcpListener, TcpStream};
use std::io::{BufReader, prelude::*};
use std::fs::read_to_string;


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
    let http_request: Vec<_> = buf_reader.lines().map(|result| result.unwrap()).take_while(|line| !line.is_empty()).collect();

    let html_contents = open_html_file(format!("{html_dir}/index.html"));

    let response = format!("HTTP/1.1 200 OK\r\n\r\n{html_contents}");

    stream.write_all(response.as_bytes()).unwrap();
}

fn open_html_file(file_name: String) -> String {
    let html_contents = std::fs::read_to_string(&file_name)
        .expect("Should have been able to read the file");

    html_contents
}