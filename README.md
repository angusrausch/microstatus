# MicroStatus

MicroStatus is a simple, lightweight status page to monitor the status of services. 

--- 
## Features

- Monitor endpoints
    - Ping 
    - Port 
    - HTTP
- Customisable services through YAML configuration
- Customisable update frequency with `.env`
- Auto refresh web page when new update is expected
- Built in low overhead webserver to share results

---
## Installation 

### Manual Compile

Ensure *rustup* is installed
To compile run the following command from the project root:
    - Debug
    ```bash 
    cargo build
    ```
    - Release
    ```bash
    cargo build --release
    ```

This will create a binary for your local system, it is not compatible between Operating Systems or Architecture

### Download

Download the latest build for your OS and Arch from [GitHub](https://github.com/angusrausch/microstatus/actions).<br><small>Currently only on actions. Releases may be used in future.</small>

---
## Manual Checks

Functionality to complete a manual check of service using CLI tool.
Can be used to check ping, port or HTTP services using the same methods as the automated checks.
This allows you to verify services availability and applications reliability.

To use run following command
```bash
microstatus --cli <args>
```
or on Windows
```bash 
microstatus.exe --cli <args>
```

Use below arguments to make request
|CLI Arg|Definition|Default|
|---|---|---|
|`--cli`|Use CLI interface for manual check|
|`--host` `-a`|Address to search (IP or Hostname)|
|`--type` `-t`|Type of check to complete<br>`ping`, `port`, `http` options available|
|`--port` `-p`|Port to check. Only used for `port` type|
|`--ssl` `-s`|Boolean value on whether to use SSL<br>Will also change behavior on HTTP if no method is selected. Will use HTTPS if true else HTTP. Only available for `http` type|`True`|


---
## Configuration 

**Configuration is required for Web use.**

### `.env`
Environment file containing values and files to use.
Can be copied from the `.env_template` file.
```env
FREQUENCY=30 # Update frequency 
CHECK_FILE=demo.yaml # File containing checks to be completed
HTML_OUTPUT_DIR=output # Output directory for HTML file
#WEBSERVER_PORT=8081 # Port for internal webserver to bind to. Leave as comment to not use webserver
```

### check_file
YAML file containing checks to be completed.
Example found in `demo.yaml`
```yaml
services:
  - name: "Ping Up"
    host: "google.com"
    svc_type: "ping"

  - name: "Ping Down"
    host: "not-up"
    svc_type: "ping"

  - name: "Port Up"
    host: "1.1.1.1"
    svc_type: "port"
    port: 53

  - name: "Port Down"
    host: "not-up"
    svc_type: "port"
    port: 32143

  - name: "HTTP Up"
    host: "google.com"
    svc_type: "http"

  - name: "HTTP Down"
    host: "not-up"
    svc_type: "http"
```

---
## Usage

1. From the directory containing `.env` and your chosen `.yaml` file run `./microstatus` (with extension if needed).  
2. This will generate a file `index.html` in your specified output dir. 
3. To keep script running you can send to background with `nohup ./microstatus &` or equivalent  Windows command
    - Can setup a service to run server on boot. See [alpine_rc-service](example_files/apline_rc-service_conf)

To serve page, use one of the following:
  - Internal webserver. Uncomment `WEBSERVER_PORT=????` in `.env`. See [Configuration](#configuration)
  - Use a web server such as NGINX to serve page. See [Nginx Conf](example_files/nginx.conf)

---
## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
