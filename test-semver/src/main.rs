use anyhow::{Context, Result};
use semver::{Version, VersionReq};
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

fn main() {
    // create VERSION file in current directory
    let version_path = Path::new("./VERSION");
    if let Err(e) = create_version_file(version_path, "0.12.2") {
        eprintln!("Error: {e:?}");
    }

    let version = read_version_file(version_path).expect("failed to read VERSION");
    println!("Database version is {version}");
    let compatible =
        VersionReq::parse(">0.12.4-alpha,<0.13.0-alpha").expect("valid version requirement");
    println!("Compatible version is {compatible}");

    // check if version is inlcuded in req
    println!("\nChecking ... ");
    if compatible.matches(&version) {
        println!("\t{compatible} matches {version}");
        println!("\tMigration does not required");
        std::process::exit(0);
    } else {
        println!("\t{compatible} does not matches {version}");
        println!("\tMigration is required");
    }

    println!("\nFinding migration requirement ... ");
    let requirement =
        VersionReq::parse(">=0.12.0,<0.12.4-alpha").expect("valid version requirement");
    if requirement.matches(&version) {
        println!("\tOk: requirement {requirement} matchtes to {version}");
        println!("\tMigration can go");
    } else {
        eprintln!("Requirement not found.");
        eprintln!("No migrations.");
    }
}

fn read_version_file(path: &Path) -> Result<Version> {
    let mut ver = String::new();
    File::open(path)
        .context("cannot open VERSION")?
        .read_to_string(&mut ver)
        .context("cannot read VERSION")?;
    Version::parse(&ver).context("cannot parse VERSION")
}

fn create_version_file(path: &Path, version: &str) -> Result<()> {
    let mut f = File::create(path).context("cannot create VERSION")?;
    f.write_all(version.as_bytes())
        .context("cannot write VERSION")?;
    Ok(())
}
