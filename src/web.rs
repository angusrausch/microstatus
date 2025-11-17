use std::io::Write;
use std::fs::{File, create_dir_all, read_to_string};
use std::process::exit;
use std::str::FromStr;
use askama::Template;
use yaml_rust2::{YamlLoader, YamlEmitter};
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

impl Default for Service {
    fn default() -> Self {
        Service {
            name: String::new(),
            svc_type: ServiceType::Ping,
            host: String::new(),
            up: false,
            port: None,
            ssl: true,
        }
    }
}

fn load_yaml(file: &str) -> Vec<Service>  {
    let yaml_contents = read_to_string(file)
    .expect("Should have been able to read the file");

    let yaml_docs = YamlLoader::load_from_str(&yaml_contents).unwrap();
    let yaml = &yaml_docs[0];
    print!("{:?}\n\n", yaml);

    if let Some(services) = yaml["services"].as_vec() {
        for service in services {
            println!("{:?}", service);
        }
    }

    vec![ //Fake Service return to allow compile
        Service {
            name: "HTTP Down".to_string(),
            host: "notup".to_string(),
            svc_type: "http".parse::<ServiceType>().unwrap(),
            ..Default::default()
        },
    ]
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> { 
    services: &'a [Service], 

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

pub fn generate() {
    let service_list: Vec<Service> = load_yaml("demo.yaml");
    
    let output = IndexTemplate { services: &service_list };
    let contents = output.render().unwrap();

    println!("{}", contents);
    create_html("output/test_index.html", &contents).unwrap();
}