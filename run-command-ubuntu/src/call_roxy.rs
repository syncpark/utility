use crate::ufw::UFW_UNIT;
use anyhow::Result;

pub fn status() -> Result<Option<String>> {
    println!("Roxy called");
    roxy::get_ufw_rules()
}

pub fn enable() -> Result<bool> {
    roxy::service_control(roxy::common::SubCommand::Enable, UFW_UNIT.to_string())
        .map(|s| s == "active")
}

pub fn disable() -> Result<bool> {
    roxy::service_control(roxy::common::SubCommand::Disable, UFW_UNIT.to_string())
        .map(|s| s == "active")
}
