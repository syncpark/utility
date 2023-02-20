use anyhow::Result;
use regex::Regex;

fn main() {
    let s = r#"
    Status: active
    
    To                         Action      From
    --                         ------      ----
    [ 1] 22/tcp                     ALLOW IN    Anywhere
    [ 2] 80/tcp                     ALLOW IN    Anywhere
    [ 3] 443/tcp                    ALLOW IN    Anywhere
    [ 4] 25/tcp                     DENY IN     Anywhere
    [ 5] 25/tcp                     DENY OUT    Anywhere
    [ 6] 22/tcp (v6)                ALLOW IN    Anywhere (v6)
    [ 7] 80/tcp (v6)                ALLOW IN    Anywhere (v6)
    [ 8] 443/tcp (v6)               ALLOW IN    Anywhere (v6)
    [ 9] 25/tcp (v6)                DENY IN     Anywhere (v6)
    [10] 25/tcp (v6)                DENY OUT    Anywhere (v6)
    [11] Anywhere                   DENY IN     203.0.113.100
    [12] Anywhere on eth0           ALLOW IN    203.0.113.102"#;
    parse_by_regex(s);
    println!("\n");

    let s = r#"
    Status: active
    
    To                         Action      From
    --                         ------      ----
    22/tcp                     ALLOW IN    Anywhere
    80/tcp                     ALLOW IN    Anywhere
    443/tcp                    ALLOW IN    Anywhere
    25/tcp                     DENY IN     Anywhere
    25/tcp                     DENY OUT    Anywhere
    22/tcp (v6)                ALLOW IN    Anywhere (v6)
    80/tcp (v6)                ALLOW IN    Anywhere (v6)
    443/tcp (v6)               ALLOW IN    Anywhere (v6)
    25/tcp (v6)                DENY IN     Anywhere (v6)
    25/tcp (v6)                DENY OUT    Anywhere (v6)
    Anywhere                   DENY IN     203.0.113.100
    Anywhere on eth0           ALLOW IN    203.0.113.102"#;
    let _r = parse_by_split(s);
}

#[allow(dead_code)]
fn parse_by_regex_old(s: &str) {
    const UFW_RULE_REGEX: &str = //r#"\[(\s\d|\d+)\]\s+([A-Z]\s[A-Z])\s+([a-zA-Z]+)"#;
        r#"\[(\s\d|\d+)\]\s(\d+/[a-z]+)\s+[\(.+\)]\s+(ALLOW\sIN|ALLOW\sOUT|DENY\sIN|DENY\sOUT)\s+([a-zA-Z]+)"#;
    const UFW_RULE_REGEX_V6: &str = r#"\[(\s\d|\d+)\]\s(\d+/[a-z]+)\s[\(v6\)]*\s+(ALLOW\sIN|ALLOW\sOUT|DENY\sIN|DENY\sOUT)\s+([a-zA-Z]+)\s[\(v6\)]*"#;
    let re = Regex::new(UFW_RULE_REGEX).unwrap();
    let re_v6 = Regex::new(UFW_RULE_REGEX_V6).unwrap();
    let lines = s.lines();
    for line in lines {
        let rule = line.trim();
        if rule.starts_with('[') {
            println!("line = {rule}");
            while let Some(cap) = re.captures(rule) {
                println!("\t{},{},{},{}", &cap[1], &cap[2], &cap[3], &cap[4]);
            }
            while let Some(cap) = re_v6.captures(rule) {
                println!("\t{},{},{},{}", &cap[1], &cap[2], &cap[3], &cap[4]);
                continue;
            }
        }
    }
}

fn parse_by_regex(s: &str) {
    let re = Regex::new(
        r#"\[(\s\d|\d+)\]\s(\d+/[a-z]+).+(ALLOW\sIN|ALLOW\sOUT|DENY\sIN|DENY\sOUT)\s+([a-zA-Z]+)"#,
    )
    .unwrap();
    println!("Index,Action,From,To");
    println!("--------------------");
    let lines = s.lines();
    for line in lines {
        if line.trim().starts_with('[') {
            if let Some(cap) = re.captures(line) {
                let idx = cap
                    .get(1)
                    .map_or(0, |c| c.as_str().trim().parse::<u16>().unwrap_or_default());
                let to = cap.get(2).map_or("", |c| c.as_str());
                let action = cap.get(3).map_or("", |c| c.as_str());
                let from = cap.get(4).map_or("", |c| c.as_str());
                let v6 = line.contains("(v6)");
                println!("{idx},{action},{from},{to},{v6}");
            }
        }
    }
}

fn parse_by_split(s: &str) -> Result<()> {
    let re_action = Regex::new(r#"(?P<a>ALLOW|DENY)\s(?P<d>IN|OUT)"#)?;
    let re_dev = Regex::new(r#"(on\s[a-z0-9]+)"#)?;
    let re_proto = Regex::new(r#"(/[a-z]+)"#)?;
    println!("Action,From,To,Proto,Dev");
    println!("------------------------");
    let lines = s.lines();
    for line in lines {
        let mut after = line.replace("Anywhere", "Any");
        let proto = if let Some(cap) = re_proto.captures(&after) {
            if let Some(p) = cap.get(1) {
                let p = p.as_str().to_string();
                after = after.replace(&p, "");
                Some(p.replace('/', ""))
            } else {
                None
            }
        } else {
            None
        };
        let dev = if let Some(cap) = re_dev.captures(&after) {
            if let Some(dev) = cap.get(1) {
                let dev_name = dev.as_str().to_string();
                after = after.replace(&dev_name, "");
                Some(dev_name.replace("on ", ""))
            } else {
                None
            }
        } else {
            None
        };
        let after = re_action.replace_all(&after, ",$a $d,");
        let v = after.split(',').collect::<Vec<_>>();
        if let Some(to) = v.first() {
            if let Some(action) = v.get(1) {
                if let Some(from) = v.get(2) {
                    println!(
                        "{},{},{},{proto:?},{dev:?}",
                        action.trim(),
                        from.trim(),
                        to.trim()
                    );
                }
            }
        }
    }
    Ok(())
}
