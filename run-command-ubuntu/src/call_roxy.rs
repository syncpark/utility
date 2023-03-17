use crate::ufw::UFW_UNIT;
use anyhow::{anyhow, Result};

pub fn status() -> Result<Option<String>> {
    println!("Roxy called");
    // roxy::get_ufw_rules()
    Err(anyhow!("unsupported!"))
}

pub fn enable() -> Result<bool> {
    roxy::service_control(roxy::common::SubCommand::Enable, UFW_UNIT.to_string())
}

pub fn disable() -> Result<bool> {
    roxy::service_control(roxy::common::SubCommand::Disable, UFW_UNIT.to_string())
}

pub fn is_active() -> Result<bool> {
    roxy::service_control(roxy::common::SubCommand::Status, UFW_UNIT.to_string())
}
