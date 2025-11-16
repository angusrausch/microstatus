# MicroStatus

MicroStatus is a simple, lightweight status page to monitor the status of services. 

--- 
## Features

- Monitor endpoints
    - Ping 
    - Port 
    - HTTP
- Customisable services through YAML configuration

---
## Installation 

### TODO

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

### TODO

---
## Startup

### TODO

---
## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

---
## License

This project is open source and available under the [MIT License](LICENSE).