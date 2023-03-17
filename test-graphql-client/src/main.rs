use anyhow::Result;
use chrono::NaiveDateTime;
use clap::Parser;
use reqwest::{Client, Method};
use rustls::{Certificate, ClientConfig, RootCertStore};
use serde::Deserialize;
use std::{
    fmt::Display,
    fs::File,
    io::Write,
    io::{BufRead, BufReader, BufWriter},
    process::exit,
};

#[derive(Parser)]
struct Args {
    #[arg(short, long, default_value = "https://localhost:8444/graphql")]
    url: String,
    #[arg(short, long, default_value = "cert.pem")]
    ca_cert: String,
    #[arg(short, long, default_value = "outliers.lst")]
    input: String,
    #[arg(short, long, default_value = "output.log")]
    output: String,
    #[arg(short, long, default_value = "sun")]
    source: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let url = &args.url;
    let outlier = &args.input;
    let output = &args.output;
    let ca_roots = vec![args.ca_cert.as_str()];
    let source = &args.source;
    let Ok(client) = client(&ca_roots) else {
        eprintln!("fail to create client");
        exit(1);
    };
    if let Err(e) = run(url, client, outlier, output, source).await {
        eprintln!("{e:?}");
    }
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
    duration: i64,
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
        write!(f, "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}", 
        self.timestamp, self.origAddr, self.origPort, self.respAddr, self.respPort, self.proto, self.duration, self.method, self.host, self.uri, self.referrer, self.version, self.userAgent, self.requestLen, self.responseLen,
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
const QUERY_HTTP_POST: &str = r#"\" } ) { nodes { timestamp origAddr origPort respAddr respPort proto duration method host uri referrer version userAgent requestLen responseLen statusCode statusMsg username password cookie contentEncoding contentType cacheControl } } }" }"#;
async fn run(url: &str, client: Client, outlier: &str, output: &str, source: &str) -> Result<()> {
    let outlier = File::open(outlier)?;
    let reader = BufReader::new(outlier);
    let mut writer = BufWriter::new(File::create(output)?);
    let mut outcount = 0;
    for line in reader.lines().flatten() {
        let Some(ids) = line
        .get(1..line.len() - 1)
        .map(|a| a.split(',').collect::<Vec<_>>()) else { continue; };
        for id in ids {
            let Ok(Some(uid)) = from_u64(id) else { continue; };
            let body = format!(
                "{QUERY_HTTP_PRE}{}{QUERY_HTTP_MIDDLE}{}{QUERY_HTTP_POST}",
                uid.format("%FT%T%.9fZ"),
                source
            );
            let builder = client.request(Method::POST, url);
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
fn from_u64(input: &str) -> Result<Option<NaiveDateTime>> {
    let input = input.parse::<i64>()?;
    let nsecs = u32::try_from(input % A_BILLION)?;
    Ok(NaiveDateTime::from_timestamp_opt(input / A_BILLION, nsecs))
}
