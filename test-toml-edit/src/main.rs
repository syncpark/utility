use std::process::exit;

use toml_edit::{value, Document};

fn main() {
    let g_toml = r#"
cert = "/data/work/aice/test/cert.pem"
key = "/data/work/aice/test/key.pem"
roots = ["/data/work/aice/test/cert.pem"]

graphql_address = "0.0.0.0:8444"
ingest_address = "0.0.0.0:38370"
publish_address = "0.0.0.0:38371"

retention = "2000d"
data_dir = "/data/work/aice/test/giganto"
log_dir = "/data/logs/apps"
export_dir = "/data/exports"    
    "#;

    let mut doc = match g_toml.parse::<Document>() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{e}");
            exit(1);
        }
    };
    println!("input:\n{doc}");
    doc["graphql_address"] = value("0.0.0.0:9000");
    doc["ingest_address"] = value("0.0.0.0:9001");
    doc["publish_address"] = value("0.0.0.0:9002");
    println!("output:\n{doc}");
}
