<h2 align="center">
  <i>mvnr</i>
</h2>
  <p align="center">A simple high performance Maven 2 repository server written in Rust</p>
    <p align="center">

<p align="center">
<a target="_blank"><img src="https://img.shields.io/github/license/CatCoderr/ProtocolSidebar" alt="License" /></a>
</p>

## Usage

Download the latest release from the [releases page](https://github.com/CatCoderr/mvnr/releases) and run it
with `./mvnr` or `mvnr.exe` on Windows.
```bash
$ ./mvnr --help

A simple high performance Maven 2 repository server written in Rust

Usage: mvnr [OPTIONS] --password <PASSWORD>

Options:
  -p, --password <PASSWORD>  Basic auth password for any user
  -r, --repo <REPO>          Path to repository directory [default: ./repository]
  -h, --host <HOST>          Web server host and port [default: 0.0.0.0:8080]
  -h, --help                 Print help
  -V, --version              Print version

```

```bash
$ ./mvnr --password password

Serving directory ./repository
Listening on http://0.0.0.0:8080
```


## TODO
* Add support for HTTPS
* More flexible permission system
* Web UI (search, upload, etc.)

## Donations
[![Buy Me a Coffee](https://img.shields.io/badge/Buy%20Me%20a%20Coffee-Donate%20Now-yellow?style=for-the-badge&logo=buy-me-a-coffee)](https://www.buymeacoffee.com/catcoderr)
