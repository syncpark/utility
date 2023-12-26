use anyhow::Result;
use std::{collections::HashMap, io::BufRead, io::BufReader};

const REGEX_SERVICE: &str = r"^([a-zA-Z0-9\-]+)\s+(\d+)/(tcp|udp)\s*";

// /etc/services 파일을 읽어서 포트 번호를 서비스 이름으로 변환하는 함수를 구현한다.
pub fn build(path: &str) -> Result<HashMap<String, String>> {
    let re = regex::Regex::new(REGEX_SERVICE)?;
    let file = std::fs::File::open(path)?;
    let rdr = BufReader::new(file);
    let mut services = std::collections::HashMap::new();
    for line in rdr.lines().flatten() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let s = line.find('#').map_or(line.as_str(), |idx| &line[..idx]);
        if let Some(x) = re.captures(s) {
            let Some(service) = x.get(1).map(|v| v.as_str().to_uppercase()) else {
                continue;
            };
            let Some(port) = x.get(2).map(|v| v.as_str().to_string()) else {
                continue;
            };
            let Some(proto) = x.get(3).map(|v| v.as_str().to_string()) else {
                continue;
            };
            services.insert(format!("{port}/{proto}"), service);
        }
    }
    Ok(services)
}
