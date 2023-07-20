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
    let networks = file.trim_end().split('\n').collect::<Vec<_>>();
    match SearchTree::build(&networks) {
        Err(e) => println!("{e:?}"),
        Ok(Some(tree)) => {
            #[cfg(feature = "estimation")]
            {
                // println!("{tree}");
                tree.estimate("after restructuring");
            }
            let _r = tree.search("11.10.1.100".parse().unwrap());
            println!("building search tree is done");
        }
        _ => println!("fail to build search tree"),
    };
}

fn rangeinclusive_int() {
    println!("RangeInclusive<i32> test ...");
    let v = 3..=9;
    println!("  - RangeInclusive = {v:?}");
    println!("    - {v:?} contains {}: {}", 3, v.contains(&3));
    println!("    - {v:?} contains {}: {}", 9, v.contains(&9));
}

fn rangeinclusive_ipaddr() {
    println!("RangeInclusive<IpAddr> test ...");
    let ipv4_begin = Ipv4Addr::new(192, 168, 1, 23);
    let ipv4_end = Ipv4Addr::new(192, 168, 1, 37);
    let ipv4_end_next = Ipv4Addr::new(192, 168, 1, 38);
    let v = IpAddr::V4(ipv4_begin)..=IpAddr::V4(ipv4_end);
    println!("  - RangeInclusive = {v:?}");
    println!(
        "    - {v:?} contains {}: {}",
        ipv4_begin,
        v.contains(&ipv4_begin)
    );
    println!(
        "    - {v:?} contains {}: {}",
        ipv4_end,
        v.contains(&ipv4_end)
    );
    println!(
        "    - {v:?} contains {}: {}",
        ipv4_end_next,
        v.contains(&ipv4_end_next)
    );

    println!("find super net for IpRange ...");
    let ip_range = IpAddrRange::from(Ipv4AddrRange::new(ipv4_begin, ipv4_end));
    println!("  - IpRange = {ip_range:?}");

    let diff = ipv4_end.saturating_sub(ipv4_begin);
    println!("  - difff: {ipv4_end} - {ipv4_begin} = {diff}");

    let ipv4_begin_net = Ipv4Net::new(ipv4_begin, 10).unwrap();
    let ipv4_max_prefix = ipv4_begin_net.max_prefix_len();
    println!("  - max prefix of {ipv4_begin_net}= {ipv4_max_prefix}");

    let ipv6_net: Ipv6Net = "fd00::/64".parse().unwrap();
    let ipv6_max_prefix = ipv6_net.max_prefix_len();
    println!("  - max prefix of {ipv6_net} = {ipv6_max_prefix}");

    let mut supernet = ipv4_begin_net;
    loop {
        let Some(s) = supernet.supernet() else { break; };
        if s.contains(&ipv4_end) {
            break;
        }
        supernet = s;
    }
    println!("  - supernet of {ip_range:?} = {supernet}");
}

fn ipnet_test() {
    println!("IpNet test ...");
    let net = IpNet::from_str("89.41.28.0/23").unwrap();
    let ip = IpAddr::from_str("89.41.29.45").unwrap();
    println!("  - {net} contains {ip} = {}", net.contains(&ip));
}

#[cfg(test)]
mod tests {
    use crate::mapper::SearchTree;

    #[test]
    fn search_test() {
        let networks = vec![
            "5.188.10.0/23",
            "11.10.1.100..=11.10.1.109",
            "24.137.16.0/20",
            "24.170.208.0/20",
            "127.0.0.1",
            "127.0.1.1",
            "204.14.80.0/22",
            "204.19.38.0/23",
            "204.21.72.173",
            "204.21.72.177",
            "204.21.72.185",
            "204.27.155.0/24",
            "204.44.32.0/20",
            "204.44.208.0/20",
            "204.44.224.0/20",
            "204.50.71.220..=204.50.71.255",
            "204.52.96.0/19",
            "204.86.16.0/20",
            "204.87.136.0/24",
            "204.87.175.0/24",
            "204.87.176.1..=204.87.176.21",
            "204.87.199.0/24",
            "204.87.233.0/24",
            "204.88.160.0/20",
            "204.89.224.0/24",
            "168.151.0.0/22",
            "168.151.4.0/23",
            "168.151.6.0/24",
            "168.151.9.0/24",
            "168.151.11.0/24",
            "168.151.16.0/24",
            "168.151.21.0/24",
            "168.151.28.0/24",
            "168.151.32.0/21",
            "168.151.43.0/24",
            "168.151.44.0/22",
            "168.151.48.0/22",
            "168.151.52.0/23",
            "168.151.54.0/24",
            "168.151.56.0/21",
            "168.151.64.0/22",
            "168.151.68.0/23",
            "168.151.72.0/21",
            "168.151.80.0/20",
            "168.151.96.0/19",
            "168.151.128.0/20",
            "168.151.145.0/24",
            "168.151.146.0/23",
            "168.151.148.0/22",
            "168.151.152.0/22",
            "168.151.157.0/24",
            "168.151.158.0/23",
            "168.151.160.0/20",
            "168.151.176.0/21",
            "168.151.184.0/22",
            "168.151.192.0/20",
            "168.151.208.0/21",
            "168.151.216.0/22",
            "168.151.220.0/23",
            "168.151.224.0/22",
            "168.151.228.0/23",
            "168.151.232.0/21",
            "168.151.240.0/21",
            "168.151.248.0/22",
            "168.151.252.0/23",
            "168.151.254.0/24",
        ];

        let tree = SearchTree::build(&networks);
        assert!(tree.is_ok());
        if let Ok(tree) = tree {
            assert!(tree.is_some());
            if let Some(tree) = tree {
                assert!(tree.search("11.10.1.100".parse().unwrap()));
                assert!(tree.search("11.10.1.108".parse().unwrap()));
                assert!(tree.search("11.10.1.109".parse().unwrap()));
                assert!(!tree.search("11.10.1.110".parse().unwrap()));
                assert!(tree.search("127.0.1.1".parse().unwrap()));
                assert!(!tree.search("127.0.1.2".parse().unwrap()));
                assert!(!tree.search("172.30.1.208".parse().unwrap()));
                assert!(tree.search("204.87.199.20".parse().unwrap()));
                assert!(tree.search("204.87.176.10".parse().unwrap()));
                assert!(tree.search("168.151.32.190".parse().unwrap()));
            }
        }
    }
}
