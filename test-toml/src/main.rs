use anyhow::Result;
use serde_derive::Deserialize;
use std::{collections::HashMap, process};

fn main() {
    match test_structure_based_toml() {
        Err(e) => {
            eprintln!("fail to parse toml. {e:?}");
            process::exit(100);
        }
        Ok(toml) => {
            println!("{toml:#?}");
        }
    }
}

#[derive(Debug, Deserialize)]
struct Config {
    envs: Vec<String>,
    _interval: Option<u16>,
    service: Vec<ServiceConf>,
}

#[derive(Debug, Deserialize)]
struct ServiceConf {
    after: Vec<String>,
    exec: String,
}

fn test_structure_based_toml() -> Result<Config> {
    let services_toml = r#"
    envs=[
        "TOPIC = httpthreat",
        "BROKER = 127.0.0.1:9092",
        "INPUT=/data/spool/log/http.log",
        "HOG_SERVER=192.168.0.140:38390",
        "PEEKCONF=/usr/local/aice/conf/peek.conf"
    ]
    interval=5

    [[service]]
    after=[
        "file://modified@/var/spool/http.log",
        "tcp://$BROKER"
        ]
    exec="/usr/local/aice/bin/reproduce -t $TOPIC -b $BROKER -i $INPUT -e"

    [[service]]
    after=[
        "file://modified@/var/spool/dns.log",
        "nic://eno1:up"
        ]
    exec="/usr/local/aice/bin/peek $PEEKCONF"
    "#;

    let mut decoded: Config = toml::from_str(services_toml)?;
    let mut envs_banks = HashMap::new();
    for line in &decoded.envs {
        let s = line.trim();
        if s.starts_with('#') {
            continue;
        }
        if let Some(pos) = s.find('=') {
            let var = &s[..pos].trim();
            let value = &s[pos + 1..].trim();
            envs_banks.insert(format!("${var}"), (*value).to_string());
        }
    }
    println!("DEBUG: replaced environment variables\n{envs_banks:#?}");

    for (key, value) in &envs_banks {
        for service in &mut decoded.service {
            for after in &mut service.after {
                if after.contains(key) {
                    *after = after.replace(key, value);
                }
            }
            if service.exec.contains(key) {
                service.exec = service.exec.replace(key, value);
            }
        }
    }
    println!("evaluated config\n{decoded:#?}");
    Ok(decoded)
}
