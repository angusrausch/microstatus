use std::io::Write;
use std::fs::{File, create_dir_all, read_to_string};
use std::str::FromStr;
use std::time::Duration;
use askama::Template;
use yaml_rust2::YamlLoader;
use chrono::Utc;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

use microstatus::{check_http, check_ping, check_port};

#[derive (Debug, Clone)]
enum ServiceType {
    Ping,
    Port,
    Http,
}

impl FromStr for ServiceType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ping" => Ok(ServiceType::Ping),
            "port" => Ok(ServiceType::Port),
            "http" => Ok(ServiceType::Http),
            other => Err(format!("unknown service type: {}", other)),
        }
    }
}

pub struct Service {
    name: String,
    svc_type: ServiceType,
    host: String,
    up: bool,
    port: Option<u16>,
    ssl: bool,
}

fn load_yaml(file: String) -> Vec<Service>  {
    let yaml_contents = read_to_string(file)
    .expect("Should have been able to read the file");

    let yaml_docs = YamlLoader::load_from_str(&yaml_contents).unwrap();
    let yaml = &yaml_docs[0];

    let mut service_list: Vec<Service> = Vec::new();
    if let Some(services) = yaml["services"].as_vec() {
        for service in services {
            service_list.push(
                Service {
                    name: service["name"].as_str().unwrap().to_string(),
                    svc_type: service["svc_type"].as_str().unwrap().parse().unwrap(),
                    host: service["host"].as_str().unwrap().to_string(),
                    up: false,
                    port: service["port"].as_i64().map(|p| p as u16),
                    ssl: service["ssl"].as_bool().unwrap_or(true),
                }
            )
        }
    }

    service_list
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> { 
    services: &'a [Service], 
    last_updated: u64,
    frequency: u16,
}

fn create_html(file_name: &str, contents: &str) -> std::io::Result<()> {
    if let Some(index) = file_name.rfind('/') {
        let (dir, _) = file_name.split_at(index);
        create_dir_all(dir)?;
    } else {
        println!("No '/' found in the path.");
    }

    let mut file = File::create(file_name)?;
    file.write_all(contents.as_bytes())?;
    Ok(())
}

fn test_service(service: &Service) -> bool {
    match service.svc_type {
        ServiceType::Ping => {
            check_ping(service.host.as_str()).unwrap()
        }
        ServiceType::Port => {
            check_port(service.host.as_str(), service.port.unwrap()).unwrap()
        }
        ServiceType::Http => {
            check_http(service.host.as_str(), service.ssl).unwrap()
        }
    }
}

pub async fn generate(frequency: u16, checks_file: String, output_dir: String) {
    let mut service_list: Vec<Service> = load_yaml(checks_file);
    
    let mut interval = tokio::time::interval(Duration::from_secs(frequency as u64));

    loop {
        interval.tick().await;

        // Get last_update_time to display
        let last_update: u64 = Utc::now().timestamp() as u64;

        // Check each service is up in parallel
        service_list.par_iter_mut().for_each(|service| {
            service.up = test_service(service);
        });

    
        // Serial version
        // for service in service_list.iter_mut() { 
        //     service.up = test_service(service);
        // }
        
        let output = IndexTemplate { services: &service_list, last_updated: last_update, frequency: frequency };
        let contents = output.render().unwrap();
    
        create_html(&format!("{output_dir}/index.html"), &contents).unwrap();
    }
}