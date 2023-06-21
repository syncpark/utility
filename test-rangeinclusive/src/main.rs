mod netgroup;

use crate::netgroup::NetGroup;
use ipnet::{IpAddrRange, IpBitAnd, IpSub, Ipv4AddrRange, Ipv4Net, Ipv6Net};
use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr},
    str::FromStr,
};

fn main() {
    rangeinclusive_int();
    rangeinclusive_ipaddr();
    let tree = build_search_tree();
    println!("{tree:#?}");
    tree.estimate();

    let ip_list: Vec<IpAddr> = vec![
        "172.30.1.208".parse().unwrap(),
        "204.87.199.20".parse().unwrap(),
        "204.87.176.10".parse().unwrap(),
    ];
    for ip in ip_list {
        let r = tree.search(ip);
        println!("search {ip}: {r}");
    }
}

fn rangeinclusive_int() {
    let v = 3..=9;
    println!("RangeInclusive = {v:?}");
}

fn rangeinclusive_ipaddr() {
    let ipv4_begin = Ipv4Addr::new(192, 168, 1, 23);
    let ipv4_end = Ipv4Addr::new(192, 168, 1, 37);
    let ip_a = IpAddr::V4(ipv4_begin);
    let ip_b = IpAddr::V4(ipv4_end);
    let v = ip_a..=ip_b;
    println!("RangeInclusive = {v:?}");

    let ip_range = IpAddrRange::from(Ipv4AddrRange::new(ipv4_begin, ipv4_end));
    println!("IpRange = {ip_range:?}");

    let diff = ipv4_end.saturating_sub(ipv4_begin);
    println!("IP Address diff = {diff}");

    let ipv4_begin_net = Ipv4Net::new(ipv4_begin, 32).unwrap();
    let ipv4_max_prefix = ipv4_begin_net.max_prefix_len();
    println!("max prefix = {ipv4_max_prefix}");
    let ipv6_net: Ipv6Net = "fd00::/126".parse().unwrap();
    let ipv6_max_prefix = ipv6_net.max_prefix_len();
    println!("max prefix = {ipv6_max_prefix}");

    let mut supernet = ipv4_begin_net;
    loop {
        let Some(s) = supernet.supernet() else { break; };
        if s.contains(&ipv4_end) {
            break;
        }
        supernet = s;
    }
    println!("supernet = {supernet}");
}

#[derive(Debug)]
struct SearchTree {
    netmask: IpAddr,
    tree: HashMap<IpAddr, Vec<NetGroup>>,
}

impl SearchTree {
    fn search(&self, ip: IpAddr) -> bool {
        let key = match (ip, self.netmask) {
            (IpAddr::V4(x), IpAddr::V4(y)) => IpAddr::V4(x.bitand(y)),
            (IpAddr::V6(x), IpAddr::V6(y)) => IpAddr::V6(x.bitand(y)),
            _ => return false,
        };

        if let Some(v) = self.tree.get(&key) {
            for net in v {
                if net.contains(ip) {
                    return true;
                }
            }
        }
        false
    }

    fn estimate(&self) {
        let sum = self.tree.iter().fold(0, |sum, (_, v)| sum + v.len());
        println!("total keys = {}", self.tree.len());
        println!("total elements = {sum}");
        println!("average elements = {}", sum / self.tree.len());
        let max = self
            .tree
            .iter()
            .max_by(|(_, a), (_, b)| a.len().cmp(&b.len()))
            .map(|(_, v)| v.len());
        if let Some(max) = max {
            println!("max elements = {max}");
        }
    }
}

fn build_search_tree() -> SearchTree {
    let nets = vec![
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
        "204.52.184.0/24",
        "204.52.255.0/24",
        "204.57.16.0/20",
        "204.61.96.0/19",
        "204.62.177.0/24",
        "204.63.64.0/18",
        "204.74.32.0/19",
        "204.75.147.0/24",
        "204.75.228.0/24",
        "204.80.164.0/24",
        "204.80.180.0/24",
        "204.80.198.0/24",
        "204.86.16.0/20",
        "204.87.136.0/24",
        "204.87.175.0/24",
        "204.87.176.1..=204.87.176.21",
        "204.87.199.0/24",
        "204.87.233.0/24",
        "204.88.160.0/20",
        "204.89.224.0/24",
        "204.91.136.0/21",
        "204.106.128.0/18",
        "204.106.192.0/19",
        "204.107.132.0/24",
        "204.107.208.0/24",
    ];
    let nets = nets
        .iter()
        .filter_map(|n| NetGroup::from_str(n).ok())
        .collect::<Vec<_>>();
    let netmask = nets
        .iter()
        .min_by(|a, b| a.prefix_len().cmp(&b.prefix_len()))
        .map(NetGroup::netmask)
        .unwrap();
    match netmask {
        IpAddr::V4(x) => println!("netmask = {:b}", u32::from(x)),
        IpAddr::V6(x) => println!("netmask = {:b}", u128::from(x)),
    }

    let mut tree: HashMap<IpAddr, Vec<NetGroup>> = HashMap::new();
    for x in nets {
        if let Some(ip) = x.bitand(netmask) {
            tree.entry(ip)
                .and_modify(|e| e.push(x.clone()))
                .or_insert_with(|| vec![x.clone()]);
            println!("{x} => {ip}");
        }
    }

    SearchTree { netmask, tree }
}
