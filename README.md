# Sensu-Plugins-Rust-loader

Sensu plugins uses Rust.

## Installation ##

  1. git clone https://github.com/metalels/sensu-plugins-rust-loader.git
  2. execute metrics/metrics-rust-loader

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
  -s, --show                          only print config envs
  -d, --debug                         print debug logs
  -h, --help                          print help menu
```

## Sample ##

```
metrics-rust-loader metrics-rust-loader.yaml 2>/dev/null
```

## Authors ##

[metalels](https://github.com/metalels)

