use anyhow::{bail, Result};
use std::process::Command;

pub const UFW_UNIT: &str = "ufw";
const DEFAULT_PATH_ENV: &str = "/usr/sbin:/usr/bin:/sbin:/bin:/usr/local/aice/bin";

pub fn status() -> Result<Option<String>> {
    if let Ok(active) = is_active() {
        if !active {
            bail!("ufw is not active");
        }
    }
    Ok(run_ufw_output(&["status", "numbered"]))
}

pub fn run_ufw_output(args: &[&str]) -> Option<String> {
    let mut cmd = Command::new(UFW_UNIT);
    cmd.env("PATH", DEFAULT_PATH_ENV);
    for arg in args {
        cmd.arg(arg);
    }
    if let Ok(output) = cmd.output() {
        if output.status.success() {
            return Some(String::from_utf8_lossy(&output.stdout).into_owned());
        }
    }
    None
}

pub fn is_active() -> Result<bool> {
    systemctl::is_active(UFW_UNIT).map_err(Into::into)
}

pub fn enable() -> Result<bool> {
    systemctl::restart(UFW_UNIT)
        .map(|status| status.success())
        .map_err(Into::into)
}

pub fn disable() -> Result<bool> {
    systemctl::stop(UFW_UNIT)
        .map(|status| status.success())
        .map_err(Into::into)
}
