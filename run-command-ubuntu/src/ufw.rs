use anyhow::{bail, Result};
use std::io::Write;
use std::process::{Command, Stdio};

pub const UFW_UNIT: &str = "ufw";
const DEFAULT_PATH_ENV: &str = "/usr/local/aice/bin:/usr/sbin:/usr/bin:/sbin:/bin";

pub fn status() -> Result<Option<String>> {
    if let Ok(active) = is_active() {
        if active != "active" {
            bail!("ufw is not active");
        }
    }
    Ok(run_ufw_output(&["status", "numbered"]))
}

pub fn is_active() -> Result<String> {
    systemctl::is_active(UFW_UNIT)
        .map(|s| s.to_string())
        .map_err(Into::into)
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

pub fn update(params: &[String]) {
    let params = params.iter().map(String::as_str).collect::<Vec<_>>();
    if params.first() == Some(&"delete") {
        if let Err(e) = run_ufw_interactive(&params) {
            eprintln!("{e}");
        }
    } else if let Some(s) = run_ufw_output(&params) {
        println!("{s}");
    }
    if let Ok(Some(s)) = status() {
        println!("{s}");
    }
}

fn run_ufw_interactive(args: &[&str]) -> Result<bool> {
    let mut child = Command::new(UFW_UNIT)
        .env("PATH", DEFAULT_PATH_ENV)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    if let Some(mut child_stdin) = child.stdin.take() {
        let _r = write!(child_stdin, "y");
    }
    Ok(child.wait()?.success())
}

fn run_ufw_output(args: &[&str]) -> Option<String> {
    let output = Command::new(UFW_UNIT)
        .env("PATH", DEFAULT_PATH_ENV)
        .args(args)
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            return Some(String::from_utf8_lossy(&output.stdout).into_owned());
        }
    }
    None
}
