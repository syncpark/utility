mod mapper;
mod netgroup;

use crate::mapper::SearchTree;
use ipnet::{IpAddrRange, IpNet, IpSub, Ipv4AddrRange, Ipv4Net, Ipv6Net};
use std::{
    net::{IpAddr, Ipv4Addr},
    str::FromStr,
};
use tracing::error;

fn main() {
    tracing_subscriber::fmt::init();

    rangeinclusive_int();
    rangeinclusive_ipaddr();
    ipnet_test();

    let Ok(file) = std::fs::read_to_string("sample_network_groups.lst") else {
        error!("fail to read sample_network_groups.lst file");
        std::process::exit(1);
    };
    let networks = file.split('\n').collect::<Vec<_>>();
    match SearchTree::build(&networks) {
        Err(e) => println!("{e:?}"),
        Ok(Some(tree)) => {
            println!("{tree}");

            #[cfg(feature = "estimation")]
            tree.estimate();

            let ip_list: Vec<IpAddr> = vec![
                "172.30.1.208".parse().unwrap(),
                "204.87.199.20".parse().unwrap(),
                "204.87.176.10".parse().unwrap(),
                "168.151.32.190".parse().unwrap(),
                "89.41.29.45".parse().unwrap(),
                "89.41.29.46".parse().unwrap(),
                "204.87.176.22".parse().unwrap(),
            ];
            for ip in ip_list {
                let r = tree.search(ip);
                println!("search {ip}: {r}");
            }
        }
        _ => println!("fail to build search tree"),
    };
}

fn rangeinclusive_int() {
    let v = 3..=9;
    println!("RangeInclusive = {v:?}");
}

fn rangeinclusive_ipaddr() {
    let ipv4_begin = Ipv4Addr::new(192, 168, 1, 23);
    let ipv4_end = Ipv4Addr::new(192, 168, 1, 37);
    let v = IpAddr::V4(ipv4_begin)..=IpAddr::V4(ipv4_end);
    println!("RangeInclusive = {v:?}");

    let ip_range = IpAddrRange::from(Ipv4AddrRange::new(ipv4_begin, ipv4_end));
    println!("IpRange = {ip_range:?}");

    let diff = ipv4_end.saturating_sub(ipv4_begin);
    println!("{ipv4_end} - {ipv4_begin} = {diff}");

    let ipv4_begin_net = Ipv4Net::new(ipv4_begin, 10).unwrap();
    let ipv4_max_prefix = ipv4_begin_net.max_prefix_len();
    println!("max prefix of {ipv4_begin_net}= {ipv4_max_prefix}");

    let ipv6_net: Ipv6Net = "fd00::/64".parse().unwrap();
    let ipv6_max_prefix = ipv6_net.max_prefix_len();
    println!("max prefix of {ipv6_net} = {ipv6_max_prefix}");

    let mut supernet = ipv4_begin_net;
    loop {
        let Some(s) = supernet.supernet() else { break; };
        if s.contains(&ipv4_end) {
            break;
        }
        supernet = s;
    }
    println!("supernet of {ip_range:?} = {supernet}");
}

fn ipnet_test() {
    let net = IpNet::from_str("89.41.28.0/23").unwrap();
    let ip = IpAddr::from_str("89.41.29.45").unwrap();
    println!("{net} contains {ip} = {}", net.contains(&ip));
}
