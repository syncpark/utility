use anyhow::{Context, Result};
use semver::{Version, VersionReq};
use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};

fn main() {
    // create VERSION file in current directory
    if let Err(e) = create_version_file(&Path::new("./VERSION"), "0.5.0-alpha.3") {
        eprintln!("Error: {e:?}");
    }

    let (_, version) = retrieve_or_create_version(".").expect("VERSION required");
    let req = VersionReq::parse(">=0.3,<0.5.0").expect("valid version requirement");
    // check if version is inlcuded in req
    if req.matches(&version) {
        println!("{req} matches {version}");
    } else {
        println!("{req} does not matches {version}");
    }

    let compatible =
        VersionReq::parse(">=0.5.0-alpha.4,<0.6.0-alpha").expect("valid version requirement");
    if compatible.matches(&version) {
        println!("Ok: compatible {compatible} matchtes to {version}");
    } else {
        eprintln!("incompatible version {version}, require {compatible}");
    }

    if req.matches(&version) {
        if compatible.matches(&version) {
            println!("Migration can go");
        }
    }
}

fn retrieve_or_create_version<P: AsRef<Path>>(path: P) -> Result<(PathBuf, Version)> {
    let path = path.as_ref();
    let file = path.join("VERSION");

    let version = read_version_file(&file)?;
    Ok((file, version))
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
