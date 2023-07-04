use crate::postgres::Cluster;
use anyhow::{bail, Result};
use chrono::NaiveDateTime;
use reqwest::{Client as request_client, Method};
use rustls::{Certificate, ClientConfig, RootCertStore};
use serde::Deserialize;
use std::{fmt::Display, fs::File, io::BufWriter, io::Write};
use tokio::runtime;

pub fn get_events(
    giganto: String,
    ca_cert: &str,
    clusters: Vec<Cluster>,
    output: String,
) -> Result<()> {
    let Ok(client) = client(&vec![ca_cert]) else {
        bail!("fail to create client");
    };
    let h = std::thread::spawn(move || {
        runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .expect("Cannot create runtime for control")
            .block_on(run(giganto, client, clusters, output))
            .unwrap_or_else(|e| eprintln!("control task terminated unexpectedly: {e}"));
    });

    h.join().expect("cannot join giganto handle");
    Ok(())
}

#[derive(Deserialize)]
struct Doc {
    data: Data,
}

impl Display for Doc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for node in &self.data.httpRawEvents.nodes {
            write!(f, "{node}")?;
        }
        Ok(())
    }
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct Data {
    httpRawEvents: HttpRawEvents,
}

#[derive(Deserialize)]
struct HttpRawEvents {
    nodes: Vec<Node>,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct Node {
    timestamp: String,
    origAddr: String,
    origPort: u16,
    respAddr: String,
    respPort: u16,
    proto: u8,
    method: String,
    host: String,
    uri: String,
    referrer: String,
    version: String,
    userAgent: String,
    requestLen: usize,
    responseLen: usize,
    statusCode: u32,
    statusMsg: String,
    username: String,
    password: String,
    cookie: String,
    contentEncoding: String,
    contentType: String,
    cacheControl: String,
}

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}", 
        self.timestamp, self.origAddr, self.origPort, self.respAddr, self.respPort, self.proto, self.method, self.host, self.uri, self.referrer, self.version, self.userAgent, self.requestLen, self.responseLen,
    self.statusCode, self.statusMsg, self.username, self.password, self.cookie, self.contentEncoding, self.contentType, self.cacheControl)
    }
}

// TODO: Use graphql query builder and schema
// TODO: Parallel processing when outliers are much more than 100_000
// TODO: Change output timestamp format

#[allow(unused)]
const QUERY_EXPORT: &str = r#"{"query":" query export{\n export(exportType: \"csv\", filter: {protocol: \"http\", sourceId: \"sun\"}) }"}"#;
const QUERY_HTTP_PRE: &str =
    r#"{"query": "query test { httpRawEvents(first: 1, filter: {time: {start: \""#;
const QUERY_HTTP_MIDDLE: &str = r#"\"}, source: \""#;
const QUERY_HTTP_POST: &str = r#"\" } ) { nodes { timestamp origAddr origPort respAddr respPort proto method host uri referrer version userAgent requestLen responseLen statusCode statusMsg username password cookie contentEncoding contentType cacheControl } } }" }"#;
async fn run(
    giganto: String,
    client: request_client,
    clusters: Vec<Cluster>,
    output: String,
) -> Result<()> {
    let mut writer = BufWriter::new(File::create(output)?);
    let mut outcount = 0;
    for c in clusters {
        let _ = writeln!(writer, "\n\n{c}");
        for (source, id) in &c.event_ids {
            let Ok(Some(uid)) = from_i64(*id) else { continue; };
            if source.is_empty() {
                continue;
            }
            let body = format!(
                "{QUERY_HTTP_PRE}{}{QUERY_HTTP_MIDDLE}{}{QUERY_HTTP_POST}",
                uid.format("%FT%T%.9fZ"),
                source
            );
            let builder = client.request(Method::POST, &giganto);
            let request = builder
                .header("Accept-Encoding", "gzip, deflate, br")
                .header("Content-Type", "application/json")
                .header("DNT", "1")
                .body(body)
                .build()?;
            let response = client.execute(request).await?;
            let Ok(node) =  serde_json::from_str::<Doc>(&response.text().await?) else { continue; };
            let _ = writeln!(writer, "{node}");
            outcount += 1;
        }
    }
    println!("{outcount} events are saved to output.log");
    Ok(())
}

fn client(ca_certs: &[&str]) -> Result<reqwest::Client> {
    let tls_config = build_client_config(ca_certs)?;
    reqwest::ClientBuilder::new()
        .use_preconfigured_tls(tls_config)
        .build()
        .map_err(Into::into)
}

fn build_client_config(root_ca: &[&str]) -> Result<ClientConfig> {
    let mut root_store = RootCertStore::empty();
    for root in root_ca {
        let certs = read_certificate_from_path(root)?;
        for cert in certs {
            root_store.add(&cert)?;
        }
    }
    let mut builder = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    builder.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec(), b"http/1.0".to_vec()];
    Ok(builder)
}

fn read_certificate_from_path(path: &str) -> Result<Vec<Certificate>> {
    let cert = std::fs::read(path)?;
    Ok(rustls_pemfile::certs(&mut &*cert)?
        .into_iter()
        .map(Certificate)
        .collect())
}

const A_BILLION: i64 = 1_000_000_000;
fn from_i64(input: i64) -> Result<Option<NaiveDateTime>> {
    let nsecs = u32::try_from(input % A_BILLION)?;
    Ok(NaiveDateTime::from_timestamp_opt(input / A_BILLION, nsecs))
}
