use anyhow::{anyhow, bail, Result};
use ipnet::{IpBitAnd, IpNet, Ipv4Net, Ipv6Net};
use std::{
    cmp::Ordering,
    collections::HashMap,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    ops::RangeInclusive,
    str::FromStr,
};
use tracing::warn;

#[derive(Clone, Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct IpMapper {
    netmask: IpAddr,
    tree: HashMap<IpAddr, IpNode>,
}

#[derive(Clone, Debug, Default)]
pub struct IpNode {
    netmask: Option<IpAddr>,
    values: Vec<NetGroup>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
enum NetGroup {
    IpNet(IpNet),
    IpRange((IpNet, RangeInclusive<IpAddr>)),
}

impl Ord for NetGroup {
    fn cmp(&self, other: &Self) -> Ordering {
        let (left, right) = match (self, other) {
            (NetGroup::IpNet(x), NetGroup::IpNet(y)) => (x.addr(), y.addr()),
            (NetGroup::IpRange((_, x)), NetGroup::IpRange((_, y))) => (*x.start(), *y.start()),
            (NetGroup::IpNet(x), NetGroup::IpRange((_, y))) => (x.addr(), *y.start()),
            (NetGroup::IpRange((_, x)), NetGroup::IpNet(y)) => (*x.start(), y.addr()),
        };
        left.cmp(&right)
    }
}

impl PartialOrd for NetGroup {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl std::fmt::Display for NetGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NetGroup::IpNet(x) => {
                write!(f, "{x}")
            }
            NetGroup::IpRange((_, x)) => {
                write!(f, "{x:?}")
            }
        }
    }
}

fn rangeinclusive_to_ipnet(v: &RangeInclusive<IpAddr>) -> Result<IpNet> {
    let start = v.start();
    let end = v.end();
    let max_prefix_len = IpNet::new(*start, 32)?.max_prefix_len();
    let Ok(mut find_super_net) = IpNet::new(*start, max_prefix_len) else {
        bail!("fail to find super net.");
    };
    loop {
        let Some(s) = find_super_net.supernet() else {
            bail!("fail to find super net.");
        };
        if s.contains(end) {
            break;
        }
        find_super_net = s;
    }
    Ok(find_super_net)
}

fn rangeinclusive_ipaddr_from_str(range: &str) -> Result<RangeInclusive<IpAddr>> {
    let s = range.split("..=").collect::<Vec<_>>();
    if s.len() == 2 {
        if let Some(first) = s.first() {
            if let Ok(first) = IpAddr::from_str(first) {
                if let Some(last) = s.last() {
                    if let Ok(last) = IpAddr::from_str(last) {
                        return Ok(RangeInclusive::new(first, last));
                    }
                }
            }
        }
    }
    Err(anyhow!("invalid RangeInclusive<IpAddr> value {}", range))
}

impl NetGroup {
    fn network(&self) -> &IpNet {
        match self {
            NetGroup::IpNet(x) | NetGroup::IpRange((x, _)) => x,
        }
    }

    fn prefix_len(&self) -> u8 {
        self.network().prefix_len()
    }

    fn netmask(&self) -> IpAddr {
        self.network().netmask()
    }

    fn contains(&self, ip: IpAddr) -> bool {
        match self {
            NetGroup::IpNet(x) => x.contains(&ip),
            NetGroup::IpRange((_, x)) => x.contains(&ip),
        }
    }

    fn bitand(&self, netmask: IpAddr) -> Option<IpAddr> {
        match (self.network(), netmask) {
            (IpNet::V4(x), IpAddr::V4(y)) => Some(IpAddr::V4(x.addr().bitand(y))),
            (IpNet::V6(x), IpAddr::V6(y)) => Some(IpAddr::V6(x.addr().bitand(y))),
            _ => None,
        }
    }
}

impl FromStr for NetGroup {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains('/') {
            if let Ok(ipnet) = s.parse::<IpNet>() {
                return Ok(NetGroup::IpNet(ipnet));
            }
        } else if s.contains("..=") {
            if let Ok(range) = rangeinclusive_ipaddr_from_str(s) {
                if let Ok(net_for_range) = rangeinclusive_to_ipnet(&range) {
                    return Ok(NetGroup::IpRange((net_for_range, range)));
                }
            }
        } else if s.contains('.') {
            if let Ok(ipv4addr) = s.parse::<Ipv4Addr>() {
                if let Ok(ipv4net) = Ipv4Net::new(ipv4addr, 32) {
                    return Ok(NetGroup::IpNet(IpNet::V4(ipv4net)));
                }
            }
        } else if s.contains(':') {
            if let Ok(ipv6addr) = s.parse::<Ipv6Addr>() {
                if let Ok(ipv6net) = Ipv6Net::new(ipv6addr, 128) {
                    return Ok(NetGroup::IpNet(IpNet::V6(ipv6net)));
                }
            }
        }
        Err("invalid network group".to_string())
    }
}

impl std::fmt::Display for IpNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} {:?}", self.netmask, self.values)
    }
}

impl IpNode {
    fn new(v: NetGroup) -> Self {
        Self {
            netmask: None,
            values: vec![v],
        }
    }

    #[allow(unused)]
    fn len(&self) -> usize {
        self.values.len()
    }

    fn values(&self) -> &Vec<NetGroup> {
        &self.values
    }

    fn netmask(&self) -> Option<IpAddr> {
        self.netmask
    }
}

impl IpMapper {
    pub fn new() -> Self {
        Self {
            netmask: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            tree: HashMap::new(),
        }
    }

    pub fn contains(&self, ipaddr: &str) -> bool {
        let Ok(ipaddr) = IpAddr::from_str(ipaddr) else {
            return false;
        };

        if let Some(masked) = network_by_ipaddr(ipaddr, self.netmask) {
            return detect_by_ipnetworks(&self.tree, ipaddr, masked).is_some();
        }
        false
    }

    pub fn build(patterns: &[String]) -> Option<IpMapper> {
        let mut netgroups = Vec::new();
        for rule in patterns {
            let Ok(net) = NetGroup::from_str(rule.trim()) else {
                warn!("invalid network group {rule}");
                continue;
            };
            netgroups.push(net.clone());
        }
        if netgroups.is_empty() {
            return None;
        }
        netgroups.sort_by_key(NetGroup::prefix_len);
        let Some(netmask) = netgroups.first().map(|f: &NetGroup| f.netmask()) else {
            return None;
        };
        let mut tree: HashMap<IpAddr, IpNode> = HashMap::new();
        for net in netgroups {
            if let Some(ip) = net.bitand(netmask) {
                tree.entry(ip)
                    .and_modify(|e| e.values.push(net.clone()))
                    .or_insert_with(|| IpNode::new(net));
            }
        }
        if tree.is_empty() {
            None
        } else {
            Some(IpMapper { netmask, tree })
        }
    }
}

fn detect_by_ipnetworks(
    tree: &HashMap<IpAddr, IpNode>,
    ipaddr: IpAddr,
    masked: IpAddr,
) -> Option<&NetGroup> {
    if let Some(node) = tree.get(&masked) {
        for net in node.values() {
            if net.contains(ipaddr) {
                return Some(net);
            }
        }
        if let Some(node_netmask) = node.netmask() {
            if let Some(masked_ip) = network_by_ipaddr(ipaddr, node_netmask) {
                if masked != masked_ip {
                    return detect_by_ipnetworks(tree, ipaddr, masked_ip);
                }
            }
        }
    }

    None
}

fn network_by_ipaddr(ipaddr: IpAddr, netmask: IpAddr) -> Option<IpAddr> {
    match (ipaddr, netmask) {
        (IpAddr::V4(x), IpAddr::V4(y)) => Some(IpAddr::V4(x.bitand(y))),
        (IpAddr::V6(x), IpAddr::V6(y)) => Some(IpAddr::V6(x.bitand(y))),
        _ => None,
    }
}
