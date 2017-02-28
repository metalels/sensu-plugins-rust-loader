# Sensu-Plugins-Rust-loader

[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE.txt)

Sensu plugins uses Rust.

## Installation ##

  1. git clone https://github.com/metalels/sensu-plugins-rust-loader.git
  2. execute metrics/metrics-rust-loader metrics/metrics-rust-loader.yaml

## Dependencies of compile ##

* Rust
* Cargo
* and see Cargo.toml

## Usage ##

```
Usage: metrics-rust-loader CONF_FILE_PATH [options]

Requires:
    CONF_FILE_PATH: path to config file

Options:
    -m, --mode MODE(snmp|url)     set only monitor mode
    -s, --single                  disable multithreading mode,
                                    and execute single thread
    -d, --debug                   print debug logs
    -D, --debug-all               print all(chain) debug logs
    -h, --help                    print this help menu
```

## Authors ##

[metalels](https://github.com/metalels)

