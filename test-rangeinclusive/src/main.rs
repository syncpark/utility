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

    let ip_list: Vec<Ipv4Addr> = vec![
        "172.30.1.208".parse().unwrap(),
        "204.87.199.20".parse().unwrap(),
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
    netmask: Ipv4Addr,
    tree: HashMap<Ipv4Addr, Vec<Ipv4Net>>,
}

impl SearchTree {
    fn search(&self, ip: Ipv4Addr) -> bool {
        let key = ip.bitand(self.netmask);
        if let Some(v) = self.tree.get(&key) {
            for net in v {
                if net.contains(&ip) {
                    return true;
                }
            }
        }
        false
    }
}

fn build_search_tree() -> SearchTree {
    let nets = vec![
        "204.14.80.0/22",
        "204.19.38.0/23",
        "204.27.155.0/24",
        "204.44.32.0/20",
        "204.44.208.0/20",
        "204.44.224.0/20",
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
        .map(|n| Ipv4Net::from_str(n).unwrap())
        .collect::<Vec<_>>();
    let netmask = nets
        .iter()
        .min_by(|a, b| a.prefix_len().cmp(&b.prefix_len()))
        .map(Ipv4Net::netmask)
        .unwrap();
    println!("netmask = {:b}", u32::from(netmask));
    let mut tree: HashMap<Ipv4Addr, Vec<Ipv4Net>> = HashMap::new();
    for x in nets {
        let ip = x.addr().bitand(netmask);
        tree.entry(ip)
            .and_modify(|e| e.push(x))
            .or_insert_with(|| vec![x]);
        println!("{x} => {ip}");
    }

    SearchTree { netmask, tree }
}

