use std::io::Write;
use std::fs::{File, create_dir_all, read_to_string};
use std::str::FromStr;
use std::collections::HashMap;
use futures::future::join_all;
use std::time::Duration;
use askama::Template;
use yaml_rust2::YamlLoader;
use chrono::Utc;
use serde_json::{Value, Map};

use microstatus::{check_http, check_ping, check_port};

#[derive (Debug, Clone, Copy)]
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

#[derive(Debug, Clone)]
pub struct Service {
    name: String,
    svc_type: ServiceType,
    host: String,
    up: bool,
    port: Option<u16>,
    ssl: bool,
}

fn load_yaml(file: String) -> HashMap<String, Vec<Service>> {
    let yaml_contents = read_to_string(file)
        .expect("Should have been able to read the file");

    let yaml_docs = YamlLoader::load_from_str(&yaml_contents).unwrap();
    let yaml = &yaml_docs[0];

    let mut service_map: HashMap<String, Vec<Service>> = HashMap::new();
    
    if let Some(groups) = yaml.as_vec() {
        for group in groups {
            let title = group["title"].as_str().unwrap().to_string();

            if let Some(services) = group["services"].as_vec() {
                let mut group_services = Vec::new();

                for service in services {
                    group_services.push(Service {
                        name: service["name"].as_str().unwrap().to_string(),
                        svc_type: service["svc_type"].as_str().unwrap().parse().unwrap(),
                        host: service["host"].as_str().unwrap().to_string(),
                        up: false,
                        port: service["port"].as_i64().map(|p| p as u16),
                        ssl: service["ssl"].as_bool().unwrap_or(true),
                    });
                }

                service_map.insert(title, group_services);
            }
        }
    }

    // Backwards compatibility with none titled service list
    if let Some(services) = yaml["services"].as_vec() {
        let title: String = "Services".to_string();
        let mut group_services = Vec::new();
        
        for service in services {
            group_services.push(Service {
                name: service["name"].as_str().unwrap().to_string(),
                svc_type: service["svc_type"].as_str().unwrap().parse().unwrap(),
                host: service["host"].as_str().unwrap().to_string(),
                up: false,
                port: service["port"].as_i64().map(|p| p as u16),
                ssl: service["ssl"].as_bool().unwrap_or(true),
            });
        }
        service_map.insert(title, group_services);
    }

    service_map
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> { 
    services: &'a HashMap<String, Vec<Service>>, 
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

async fn test_service(service: &Service) -> bool {
    match service.svc_type {
        ServiceType::Ping => {
            check_ping(service.host.as_str()).unwrap()
        }
        ServiceType::Port => {
            check_port(service.host.as_str(), service.port.unwrap()).unwrap_or(false)
        }
        ServiceType::Http => {
            check_http(service.host.as_str(), service.ssl).await.unwrap()
        }
    }
}

async fn add_history(services: Vec<Service>, json_file: String, max_length: u32) -> Result<(), serde_json::Error> {
    let mut json: Value;
    if tokio::fs::metadata(&json_file).await.is_ok() {
        let json_string: String = tokio::fs::read_to_string(&json_file).await.expect("Should have been able to read the file");
        if serde_json::from_str::<Value>(&json_string).is_err() {
            println!("JSON INVALID. Rewriting file");
            json = Value::Object(Map::new());
        } else {
            json = serde_json::from_str(&json_string)?;
        }
    } else {
        print!("File Not Found. Writing file");
        json = Value::Object(Map::new());
    }

    let now = Utc::now().to_rfc3339();

    for service in services {
        if let Value::Object(ref mut map) = json {
            let entry = serde_json::json!({
                "timestamp": now,
                "status": service.up
            });

            match map.get_mut(&service.name) {
                Some(Value::Array(arr)) => {
                    arr.push(entry);
                    while arr.len() > max_length as usize {
                        arr.remove(0);
                    }
                }
                Some(_) => {
                    map.insert(service.name.clone(), Value::Array(vec![entry]));
                }
                None => {
                    map.insert(service.name.clone(), Value::Array(vec![entry]));
                }
            }
        }
    }

    if let Value::Object(_) = json {
        let json_string = serde_json::to_string_pretty(&json)?;
        tokio::fs::write(&json_file, json_string).await.expect("Unable to write history file");
    }

    Ok(())
}

pub async fn generate(frequency: u16, checks_file: String, output_dir: String) -> Result<(), serde_json::Error> {
    let mut service_list: HashMap<String, Vec<Service>> = load_yaml(checks_file);
    let mut interval = tokio::time::interval(Duration::from_secs(frequency as u64));

    loop {
        interval.tick().await;

        let last_update: u64 = Utc::now().timestamp() as u64;

        // Collect futures for every service in the hashmap (stable traversal order for values() / values_mut())
        let mut checks = Vec::new();
        for group in service_list.values() {
            for service in group.iter() {
                checks.push(test_service(service));
            }
        }

        // Await all checks in parallel
        let results: Vec<bool> = join_all(checks).await;

        // Apply results back to the services in the same traversal order
        let mut res_iter = results.into_iter();
        for group in service_list.values_mut() {
            for service in group.iter_mut() {
                if let Some(r) = res_iter.next() {
                    service.up = r;
                }
            }
        }
    
        let output = IndexTemplate { services: &service_list, last_updated: last_update, frequency };
        let contents = output.render().unwrap();

        let all_services: Vec<Service> = service_list.values().flat_map(|v| v.iter().cloned()).collect();
        add_history(all_services, "history.json".to_string(), 3).await?;

        create_html(&format!("{output_dir}/index.html"), &contents).unwrap();
    }
}