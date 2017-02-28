extern crate getopts;
extern crate toml;
extern crate yaml_rust;

use std::env;
use std::process::{Command, ExitStatus};
use std::io::{Write, Read};
use std::fs::OpenOptions;
use std::collections::HashMap;
use std::thread;
use getopts::Options;
use yaml_rust::YamlLoader;
use toml::Value;

fn print_usage(program: &str, opts: Options) {
  let brief = format!("Usage: {} CONF_FILE_PATH [Options]\n\nRequires:\n    CONF_FILE_PATH: path to (yaml|toml) config file", program);
  print!("{}", opts.usage(&brief));
}

fn yaml_multiple_opt_join(conf: &Vec<yaml_rust::Yaml>) -> Option<String> {
  let mut res: Vec<&str> = Vec::with_capacity(conf.len());
  for x in conf {
    res.push(x.as_str().unwrap());
  }
  Some(res.join(","))
}

fn toml_multiple_opt_join(conf: &Vec<Value>) -> Option<String> {
  let mut res: Vec<&str> = Vec::with_capacity(conf.len());
  for x in conf {
    res.push(x.as_str().unwrap());
  }
  Some(res.join(","))
}

fn execute_command(command: String, func_opts: Vec<String>, target_opts: Vec<String>) -> ExitStatus {
  let mut child = Command::new(command)
    .args(&func_opts)
    .args(&target_opts)
    .spawn()
    .expect("failed to execute process");
  let status = child.wait().unwrap();
  return status;
}

fn is_ok(toml: &Value, key: &str) -> bool{
  match toml.get(key) {
    Some(_) => { true }
    None => { false }
  }
}

fn toml_loader(conf_body: &str, mode: String, mt: bool, debug: bool, all_debug: bool) {
  let conf = conf_body.parse::<Value>().unwrap();
  if is_ok(&conf, "snmp") {
    let snmp = &conf["snmp"];
    if (mode == "all" || mode == "snmp") && is_ok(&snmp, "bin") && is_ok(&snmp, "hosts"){
      toml_snmp_metrics(snmp, mt, debug, all_debug);
    }
  }
  if is_ok(&conf, "response-url") {
    let resp = &conf["response-url"];
    if (mode == "all" || mode == "url") && is_ok(&resp, "bin") && is_ok(&resp, "metrics") {
      toml_url_response_metrics(resp, mt, debug, all_debug);
    }
  }
}

fn yaml_loader(conf_body: &str, mode: String, mt: bool, debug: bool, all_debug: bool) {
  let yaml = YamlLoader::load_from_str(conf_body).unwrap();
  let conf = &yaml[0];
  if !conf["snmp"].is_badvalue() {
    let snmp = &conf["snmp"];
    if (mode == "all" || mode == "snmp") && !snmp["bin"].is_badvalue() && !snmp["hosts"].is_badvalue() {
      yaml_snmp_metrics(snmp, mt, debug, all_debug);
    }
  }
  if !conf["response-url"].is_badvalue() {
    let resp = &conf["response-url"];
    if (mode == "all" || mode == "url") && !resp["bin"].is_badvalue() && !resp["metrics"].is_badvalue() {
      yaml_url_response_metrics(resp, mt, debug, all_debug);
    }
  }
}

fn yaml_snmp_metrics(snmp: &yaml_rust::Yaml, mt: bool, debug: bool, all_debug: bool) {
  let mut snmp_hosts: HashMap<&str, Vec<String>> = HashMap::new();
  let hosts = snmp["hosts"].as_vec().unwrap();
  for target in hosts {
    let mut cmd_opt: Vec<String> = Vec::with_capacity(hosts.len());
    let mut name;

    match target["host"].as_str() {
      Some(m) => {
        cmd_opt.push("-h".to_string());
        cmd_opt.push(m.to_string());
        name = m;
      }
      None => { name = "127.0.0.1" }
    };
    match target["name"].as_str() {
      Some(m) => {
        cmd_opt.push("-n".to_string());
        cmd_opt.push(m.to_string());
        name = m
      }
      None => { }
    };
    match target["port"].as_str() {
      Some(m) => {
        cmd_opt.push("-p".to_string());
        cmd_opt.push(m.to_string());
      }
      None => { }
    };
    match target["community"].as_str() {
      Some(m) => {
        cmd_opt.push("-c".to_string());
        cmd_opt.push(m.to_string());
      }
      None => { }
    };
    snmp_hosts.insert(name.clone(), cmd_opt.clone());
  }

  if debug {
    println!("metrics-snmp binary: {}", snmp["bin"].as_str().unwrap());
    println!("hosts: {:?}", snmp_hosts);
  }
  if !snmp["metrics"].is_badvalue() {
    for metric in snmp["metrics"].as_vec().unwrap() {
      let mut cmd_opt: Vec<String> = Vec::new();
      match metric["metric_name"].as_str() {
        Some(m) => { cmd_opt.push(m.to_string()); }
        None => {
          let _ = writeln!(&mut std::io::stderr(), "Metric name must be specified.", );
          continue
        }
      };
      if all_debug {
        cmd_opt.push("-D".to_string());
      }
      match metric["oids"].as_vec() {
        Some(m) => {
          match yaml_multiple_opt_join(m) {
            Some(o) => {
              cmd_opt.push("-o".to_string());
              cmd_opt.push(o);
            }
            None => {
              let _ = writeln!(&mut std::io::stderr(), "Metric {} contains invalid oids.", cmd_opt[0]);
            }
          }
        }
        None => { }
      };
      if !metric["targets"].is_badvalue() {
        let mut children = vec![];
        for target in metric["targets"].as_vec().unwrap() {
          let target = target.as_str().unwrap();
          if snmp_hosts.contains_key(target) {
            let binary = snmp["bin"].as_str().unwrap().to_string();
            let host = snmp_hosts[target].clone();
            let opts1 = cmd_opt.clone();
            if debug {
              println!("{} {:?} {:?}", binary, opts1, host);
            }
            if mt {
              children.push(thread::spawn(move || {
                  let _ = execute_command(binary, opts1, host);
                }
              ));
            } else {
              let _ = execute_command(binary, opts1, host);
            }
          } else {
            let _ = writeln!(&mut std::io::stderr(), "Target {} is not defined.", target);
          }
        }
        if mt {
          for child in children {
            let _ = child.join();
          }
        }
      }
    }
  }
}

fn yaml_url_response_metrics(resp: &yaml_rust::Yaml, mt: bool, debug: bool, all_debug: bool) {
  if debug {
    println!("metrics-snmp binary: {}", resp["bin"].as_str().unwrap());
  }
  if !resp["metrics"].is_badvalue() {
    let mut children = vec![];
    for metric in resp["metrics"].as_vec().unwrap() {
      let mut cmd_opt: Vec<String> = Vec::new();
      match metric["urls"].as_vec() {
        Some(m) => {
          match yaml_multiple_opt_join(m) {
            Some(o) => {
              cmd_opt.push(o);
            }
            None => {
              let _ = writeln!(&mut std::io::stderr(), "Metric contains invalid urls.");
              continue
            }
          }
        }
        None => {
          let _ = writeln!(&mut std::io::stderr(), "Metric urls must be specified.");
          continue
        }
      };
      if all_debug {
        cmd_opt.push("-D".to_string());
      }
      match metric["prefix"].as_str() {
        Some(m) => {
          cmd_opt.push("-p".to_string());
          cmd_opt.push(m.to_string());
        }
        None => { }
      };
      match metric["timeout"].as_bool() {
        Some(m) => {
          cmd_opt.push("-t".to_string());
          cmd_opt.push(m.to_string());
        }
        None => { }
      };
      match metric["http2"].as_bool() {
        Some(m) => {
          if m {
            cmd_opt.push("-2".to_string());
          }
        }
        None => { }
      };
      match metric["ssl_verify_none"].as_bool() {
        Some(m) => {
          if m {
            cmd_opt.push("-i".to_string());
          }
        }
        None => { }
      };
      let binary = resp["bin"].as_str().unwrap().to_string();
      let opts1 = cmd_opt.clone();
      if debug {
        println!("{} {:?}", binary, opts1);
      }
      if mt {
        children.push(thread::spawn(move || {
            let _ = execute_command(binary, opts1, Vec::new());
          }
        ));
      } else {
        let _ = execute_command(binary, opts1, Vec::new());
      }
    }
    if mt {
      for child in children {
        let _ = child.join();
      }
    }
  }
}

fn toml_snmp_metrics(snmp: &Value, mt: bool, debug: bool, all_debug: bool) {
  let mut snmp_hosts: HashMap<&str, Vec<String>> = HashMap::new();
  let hosts = snmp["hosts"].as_array().unwrap();
  for target in hosts {
    let mut cmd_opt: Vec<String> = Vec::with_capacity(hosts.len());
    let mut name;
    if is_ok(&target, "host") {
      let m = target["host"].as_str().unwrap();
      cmd_opt.push("-h".to_string());
      cmd_opt.push(m.to_string());
      name = m;
    } else {
      name = "127.0.0.1";
    }
    if is_ok(&target, "name") {
      let m = target["name"].as_str().unwrap();
      cmd_opt.push("-n".to_string());
      cmd_opt.push(m.to_string());
      name = m;
    }
    if is_ok(&target, "port") {
      let m = target["port"].as_str().unwrap();
      cmd_opt.push("-p".to_string());
      cmd_opt.push(m.to_string());
    }
    if is_ok(&target, "community") {
      let m = target["community"].as_str().unwrap();
      cmd_opt.push("-c".to_string());
      cmd_opt.push(m.to_string());
    }

    snmp_hosts.insert(name.clone(), cmd_opt.clone());
  }

  if debug {
    println!("metrics-snmp binary: {}", snmp["bin"].as_str().unwrap());
    println!("hosts: {:?}", snmp_hosts);
  }
  if is_ok(&snmp, "metrics") {
    for metric in snmp["metrics"].as_array().unwrap() {
      let mut cmd_opt: Vec<String> = Vec::new();
      if is_ok(&metric, "metric_name") {
        let m = metric["metric_name"].as_str().unwrap();
        cmd_opt.push(m.to_string());
      } else {
        let _ = writeln!(&mut std::io::stderr(), "Metric name must be specified.", );
        continue;
      }
      if all_debug {
        cmd_opt.push("-D".to_string());
      }
      if is_ok(&metric, "oids") {
        let m =  metric["oids"].as_array().unwrap();
        match toml_multiple_opt_join(m) {
          Some(o) => {
            cmd_opt.push("-o".to_string());
            cmd_opt.push(o);
          }
          None => {
            let _ = writeln!(&mut std::io::stderr(), "Metric {} contains invalid oids.", cmd_opt[0]);
          }
        };
      }
      if is_ok(&metric ,"targets") {
        let mut children = vec![];
        for target in metric["targets"].as_array().unwrap() {
          let target = target.as_str().unwrap();
          if snmp_hosts.contains_key(target) {
            let binary = snmp["bin"].as_str().unwrap().to_string();
            let host = snmp_hosts[target].clone();
            let opts1 = cmd_opt.clone();
            if debug {
              println!("{} {:?} {:?}", binary, opts1, host);
            }
            if mt {
              children.push(thread::spawn(move || {
                  let _ = execute_command(binary, opts1, host);
                }
              ));
            } else {
              let _ = execute_command(binary, opts1, host);
            }
          } else {
            let _ = writeln!(&mut std::io::stderr(), "Target {} is not defined.", target);
          }
        }
        if mt {
          for child in children {
            let _ = child.join();
          }
        }
      }
    }
  }
}

fn toml_url_response_metrics(resp: &Value, mt: bool, debug: bool, all_debug: bool) {
  if debug {
    println!("metrics-snmp binary: {}", resp["bin"].as_str().unwrap());
  }
  if is_ok(&resp, "metrics") {
    let mut children = vec![];
    for metric in resp["metrics"].as_array().unwrap() {
      let mut cmd_opt: Vec<String> = Vec::new();
      if is_ok(&metric, "urls") {
        let m = metric["urls"].as_array().unwrap();
        match toml_multiple_opt_join(m) {
          Some(o) => {
            cmd_opt.push(o);
          }
          None => {
            let _ = writeln!(&mut std::io::stderr(), "Metric contains invalid urls.");
            continue
          }
        };
      } else {
        let _ = writeln!(&mut std::io::stderr(), "Metric contains invalid urls.");
        continue;
      }
      if all_debug {
        cmd_opt.push("-D".to_string());
      }
      if is_ok(&metric, "prefix") {
        let m = metric["prefix"].as_str().unwrap();
        cmd_opt.push("-p".to_string());
        cmd_opt.push(m.to_string());
      }
      if is_ok(&metric, "timeout") {
        let m = metric["timeout"].as_str().unwrap();
        cmd_opt.push("-t".to_string());
        cmd_opt.push(m.to_string());
      }
      if is_ok(&metric, "http2") {
        let m = metric["http2"].as_bool().unwrap();
        if m {
          cmd_opt.push("-2".to_string());
        }
      }
      if is_ok(&metric, "ssl_verify_none") {
        let m = metric["ssl_verify_none"].as_bool().unwrap();
        if m {
          cmd_opt.push("-i".to_string());
        }
      }
      let binary = resp["bin"].as_str().unwrap().to_string();
      let opts1 = cmd_opt.clone();
      if debug {
        println!("{} {:?}", binary, opts1);
      }
      if mt {
        children.push(thread::spawn(move || {
            let _ = execute_command(binary, opts1, Vec::new());
          }
        ));
      } else {
        let _ = execute_command(binary, opts1, Vec::new());
      }
    }
    if mt {
      for child in children {
        let _ = child.join();
      }
    }
  }
}

fn main() {
  let args: Vec<String> = env::args().collect();
  let program = args[0].clone();
  let mut opts = Options::new();
  opts.optopt("m", "mode", "set only monitor mode", "MODE(snmp|url)");
  opts.optflag("s", "single", "disable multithreading mode, and execute single thread");
  opts.optflag("d", "debug", "print debug logs");
  opts.optflag("D", "debug-all", "print all(chain) debug logs");
  opts.optflag("h", "help", "print this help menu");

  let matches = match opts.parse(&args[1..]) {
    Ok(m) => { m }
    Err(f) => {
      let _ = writeln!(&mut std::io::stderr(), "{}", f);
      return
    }
  };

  if matches.opt_present("h") {
    print_usage(&program, opts);
    return;
  };

  if matches.free.len() < 1 {
    let _ = writeln!(&mut std::io::stderr(), "Config file path must be specified.");
    return;
  };

  let mut multi_thread = true;
  let mut debug = false;
  let mut all_debug = false;

  if matches.opt_present("s") {
    multi_thread = false;
  }
  if matches.opt_present("D") {
    debug = true;
    all_debug = true;
  } else if matches.opt_present("d") {
    debug = true;
  }

  let filepath = matches.free[0].clone();
  let mut file;
  match OpenOptions::new().read(true).open(filepath) {
    Ok(m) => { file = m }
    Err(f) => {
      let _ = writeln!(&mut std::io::stderr(), "{}", f);
      return
    }
  }
  let mut conf_body = String::new();
  match file.read_to_string(&mut conf_body) {
    Ok(_) => {
      if debug {
        println!("Conf file load has success!");
      }
    }
    Err(f) => {
      let _ = writeln!(&mut std::io::stderr(), "{}", f);
      return
    }
  };

  let mode = match matches.opt_str("m") {
    Some(m) => { m }
    None => { "all".to_string() }
  };

  if debug {
    println!("mode: {:?}", mode);
  }

  let filepath = matches.free[0].clone();
  if filepath.ends_with(".toml") {
    toml_loader(&*conf_body, mode, multi_thread, debug, all_debug);
  } else if filepath.ends_with(".yaml") || filepath.ends_with(".yml") {
    yaml_loader(&*conf_body, mode, multi_thread, debug, all_debug);
  } else {
    let _ = writeln!(&mut std::io::stderr(), "config file type is not supported");
    return
  }
}
