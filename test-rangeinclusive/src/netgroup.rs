use anyhow::{anyhow, bail, Result};
use ipnet::{IpBitAnd, IpNet, Ipv4Net, Ipv6Net};
use std::{
    cmp::Ordering,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    ops::RangeInclusive,
    str::FromStr,
};

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum NetGroup {
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
        let (left, right) = match (self, other) {
            (NetGroup::IpNet(x), NetGroup::IpNet(y)) => (x.addr(), y.addr()),
            (NetGroup::IpRange((_, x)), NetGroup::IpRange((_, y))) => (*x.start(), *y.start()),
            (NetGroup::IpNet(x), NetGroup::IpRange((_, y))) => (x.addr(), *y.start()),
            (NetGroup::IpRange((_, x)), NetGroup::IpNet(y)) => (*x.start(), y.addr()),
        };
        Some(left.cmp(&right))
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
    let Ok(mut find_super_net) =
        IpNet::new(*start, max_prefix_len) else { bail!("fail to find super net."); };
    loop {
        let Some(s) = find_super_net.supernet() else { bail!("fail to find super net."); };
        if s.contains(end) {
            break;
        }
        find_super_net = s;
    }
    Ok(find_super_net)
}

impl NetGroup {
    fn network(&self) -> &IpNet {
        match self {
            NetGroup::IpNet(x) | NetGroup::IpRange((x, _)) => x,
        }
    }

    pub fn prefix_len(&self) -> u8 {
        self.network().prefix_len()
    }

    pub fn netmask(&self) -> IpAddr {
        self.network().netmask()
    }

    pub fn contains(&self, ip: IpAddr) -> bool {
        self.network().contains(&ip)
    }

    pub fn bitand(&self, netmask: IpAddr) -> Option<IpAddr> {
        match (self.network(), netmask) {
            (IpNet::V4(x), IpAddr::V4(y)) => Some(IpAddr::V4(x.addr().bitand(y))),
            (IpNet::V6(x), IpAddr::V6(y)) => Some(IpAddr::V6(x.addr().bitand(y))),
            _ => None,
        }
    }
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

// convert IpAddr, IpNet, RangeInclusive<IpAddr> string to NetGroup
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
