use std::io::Write;
use std::fs::{File, create_dir_all};
use askama::Template;
use microstatus::{check_http, check_ping, check_port};


pub struct Service {
    pub name: String,
    pub up: bool,
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
    let services = vec![
        Service {
            name: "Up".to_string(),
            up: check_http("google.com", true).unwrap_or(false),
        },
        Service {
            name: "Down".to_string(),
            up: check_http("noup", true).unwrap_or(false),
        },
        Service {
            name: "Down".to_string(),
            up: check_http("noup", true).unwrap_or(false),
        },
        Service {
            name: "Up".to_string(),
            up: check_http("google.com", true).unwrap_or(false),
        },
        
    ];

    let output = IndexTemplate { services: &services };
    let contents = output.render().unwrap();

    println!("{}", contents);
    create_html("output/test_index.html", &contents).unwrap();
}