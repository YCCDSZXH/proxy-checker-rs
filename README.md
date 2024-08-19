# Proxy Checker

This repository provides an open-source server-side API for detecting if a client is using a proxy. The detection method compares the Round-Trip Time (RTT) of both the TLS and TCP layers. This differential analysis helps identify the presence of a proxy.

## Usage

Clone the repository:

```bash
git clone https://github.com/YCCDSZXH/proxy-checker-rs.git
cd proxy-checker-rs
```

Build

```bash
cargo b
```

Configure certificate 
```bash
openssl req -new -newkey rsa:2048 -days 365 -nodes -x509 -keyout server.key -out server.crt
```

Run the server:

```bash
cargo r
```

Make a request to the detection endpoint:

```bash
curl http://localhost:8443 -k
```
If you use self generate certificate, you need `-k` to skip certificate verify

The API will return a JSON response indicating whether a proxy was detected.

## Contributing

Contributions are welcome! Feel free to submit issues, fork the repository, and create pull requests.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for more details.

