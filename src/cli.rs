use std::io;
use clap::{Arg, ArgAction, ArgMatches, Command};

use microstatus::{check_http, check_ping, check_port};

fn get_arguments() -> ArgMatches {
    Command::new("Service checker")
        .about("Perform ping, HTTP and port checks")
        .arg(
            Arg::new("cli")
                .long("cli")
                .help("Run in CLI mode instead of server")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("host")
                .long("host")
                .short('a')
                .help("Host to check (hostname or IP)")
                .value_parser(clap::value_parser!(String))
                .required(true),
        )
        .arg(
            Arg::new("type")
                .long("type")
                .short('t')
                .help("Type of check to perform")
                .value_parser(["ping", "port", "http"])
                .required(true),
        )
        .arg(
            Arg::new("port")
                .long("port")
                .short('p')
                .help("Port to check for port check")
                .value_parser(clap::value_parser!(u16))
                .required_if_eq("type", "port"),
        )
        .arg(
            Arg::new("ssl")
                .long("ssl")
                .short('s')
                .help("Use SSL for HTTP requests")
                .value_parser(clap::value_parser!(bool))
                .default_value("true"),
        )
        .get_matches()
}

pub async fn cli_check() -> io::Result<bool> {
    let matches = get_arguments();

    let host = matches.get_one::<String>("host").unwrap();
    let check_type = matches.get_one::<String>("type").unwrap().to_lowercase();
    
    match check_type.as_str() {
        "ping" => {
            match check_ping(host)? {
                true => println!("{host} is up"),
                false => println!("{host} is down"),
            }
        }
        "port" => {
            let port = matches.get_one::<u16>("port").unwrap();
            match check_port(host, *port)? {
                true => println!("{host}:{port} is up"),
                false => println!("{host}:{port} is down"),
            }
        }
        "http" => {
            let ssl = matches.get_one::<bool>("ssl").unwrap();
            match check_http(host, *ssl).await? {
                true => println!("{host} is up"),
                false => println!("{host} is down"),
            }
        }
        _ => {
            println!("Type not valid");
        }
    }

    Ok(true)
}