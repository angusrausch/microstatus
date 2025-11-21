use std::net::{TcpStream, ToSocketAddrs};
use std::process::{Command, Stdio};
use reqwest::Client;
use std::io;
use std::time::Duration;

pub fn check_ping(host: &str) -> io::Result<bool> {
    let output = Command::new("ping")
        .args(["-c", "1", "-W", "1", host]) 
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;

    Ok(output.success())
}

pub async fn check_http(host: &str, ssl: bool) -> io::Result<bool> {
    let url = if host.starts_with("http://") || host.starts_with("https://") {
        host.to_string()
    } else if ssl {
        format!("https://{}", host)
    } else {
        format!("http://{}", host)
    };

    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;


    match client.get(&url).send().await {
        Ok(resp) => {
            Ok(resp.status().is_success())
        }
        Err(_) => Ok(false),
    }
}

pub fn check_port(host: &str, port: u16) -> io::Result<bool> {
    let timeout = Duration::from_secs(3);

    let addr = (host, port)
        .to_socket_addrs()?
        .next()
        .ok_or(io::Error::new(io::ErrorKind::Other, "no socket address found"))?;

    Ok(TcpStream::connect_timeout(&addr, timeout).is_ok())
}