use std::collections::HashMap;

pub struct Hosts {
    hosts: HashMap<(String, u16, String), u32>,
}

impl Hosts {
    pub fn new() -> Self {
        Self {
            hosts: HashMap::new(),
        }
    }

    pub fn insert(&mut self, host: &str, port: u16, proto: &str) {
        let key = (host.to_string(), port, proto.to_string());
        self.hosts
            .entry(key)
            .and_modify(|cnt| *cnt += 1)
            .or_insert(1);
    }

    pub fn servers(&self) -> Vec<(String, u16, String)> {
        let mut hosts = self
            .hosts
            .iter()
            .filter(|((_, port, _), cnt)| **cnt > 3 && *port != 0)
            .map(|(k, _)| k.clone())
            .collect::<Vec<_>>();
        hosts.sort();
        hosts.dedup();
        hosts
    }

    pub fn hosts(&self) -> Vec<String> {
        let mut hosts = self.hosts.keys().map(|k| k.0.clone()).collect::<Vec<_>>();
        hosts.sort();
        hosts.dedup();
        hosts
    }
}
