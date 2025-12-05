use std::net::{TcpListener, TcpStream};
use std::io::{BufReader, prelude::*};
use std::path::PathBuf;
use std::thread;
use std::sync::{mpsc, Arc, Mutex};

struct ThreadPool {
    _workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut _workers = Vec::with_capacity(size);

        for id in 0..size {
            _workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { _workers, sender }
    }

    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.sender.send(Box::new(f)).unwrap();
    }
}

struct Worker {
    _id: usize,
    _thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv();
            match job {
                Ok(job) => {
                    job();
                }
                Err(_) => break,
            }
        });

        Worker { _id: id, _thread: thread }
    }
}

pub async fn run_server(port: u16, html_dir: Arc<PathBuf>) {
    let bind_addr: String = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&bind_addr).unwrap();

    println!("Starting listener @ {bind_addr}");

    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let html_dir = Arc::clone(&html_dir);
        pool.execute(move || {
            handle_connection(stream, html_dir);
        });
    }
}

fn handle_connection(mut stream: TcpStream, html_dir: Arc<PathBuf>) {
    let buf_reader = BufReader::new(&stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

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

