mod hosts;
mod net;
mod services;

use crate::net::IpMapper;
use anyhow::Result;
use hosts::Hosts;
use std::collections::HashMap;
use structopt::StructOpt;

#[derive(Debug, structopt::StructOpt)]
struct Config {
    #[structopt(short, long, help = "filename to read")]
    filename: String,
    // #[structopt(short, long, help = "option")]
    // opt: Option<String>,
}

const LOCAL_NETWORKS: [&str; 3] = ["10.0.0.0/8", "172.16.0.0/12", "192.168.0.0/16"];

fn main() {
    tracing_subscriber::fmt::init();

    let conf = Config::from_args();
    let Ok(services) = services::build("/etc/services") else {
        eprintln!("Error: fail to parse /etc/services");
        std::process::exit(1);
    };

    let local_net = IpMapper::build(
        &LOCAL_NETWORKS
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
    )
    .unwrap_or(IpMapper::new());

    // println!("services {:#?}", services);
    println!("services {} entries", services.len());

    if let Err(e) = read_csv_file(&conf.filename, &services, &local_net) {
        eprintln!("Error: {e}");
    }
}

const PROTO_TCP: &str = "tcp";
const PROTO_UDP: &str = "udp";

// 다음과 같은 필드로 구성된 TAB separated CSV 파일을 읽어서 destination ip address, destination port 종류와 등장 횟수를 계산하는 함수를 작성하시오.
// fields: timestamp, source name, source ip address, source port, destination ip address, destination port, protocol, session end time, service name, sent bytes, received bytes, send packets, received packets
fn read_csv_file(
    filename: &str,
    services: &HashMap<String, String>,
    local_net: &IpMapper,
) -> Result<()> {
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .flexible(true)
        .from_path(filename)?;
    let mut hosts = Hosts::new();
    for rec in rdr.records().flatten() {
        let Some((src, src_is_local)) = rec.get(2).map(|v| (v.to_string(), local_net.contains(v)))
        else {
            continue;
        };
        let Some((dest, dest_is_local)) =
            rec.get(4).map(|v| (v.to_string(), local_net.contains(v)))
        else {
            continue;
        };
        if !(src_is_local || dest_is_local) {
            continue;
        }
        let Some(dport) = rec.get(5).map(|p| p.parse::<u16>().unwrap_or_default()) else {
            continue;
        };

        let Some(proto) = rec.get(6).and_then(|p| {
            if p == "6" {
                Some(PROTO_TCP)
            } else if p == "17" {
                Some(PROTO_UDP)
            } else {
                None
            }
        }) else {
            continue;
        };

        if src_is_local {
            hosts.insert(src.as_str(), 0, proto);
        }

        if dest_is_local {
            hosts.insert(dest.as_str(), dport, proto);
        }
    }

    println!("hosts {} entries", hosts.hosts().len());
    for host in hosts.hosts() {
        println!("{host}");
    }

    println!("\nservers {} entries", hosts.servers().len());
    for (server, port, proto) in hosts.servers() {
        let service = format!("{port}/{proto}");
        println!(
            "{server}\t{port}\t{proto}\t{}",
            services.get(&service).unwrap_or(&service)
        );
    }

    Ok(())
}
