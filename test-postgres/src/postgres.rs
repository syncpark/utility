use postgres::{Client, Error, NoTls};
use std::fmt::Display;

pub struct Cluster {
    cluster_id: String,
    size: i64,
    labels: Option<Vec<String>>,
    signature: String,
    pub event_ids: Vec<(String, i64)>,
}

impl Display for Cluster {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "cluster_id = {}", self.cluster_id)?;
        writeln!(f, "      size = {}", self.size)?;
        writeln!(f, "    labels = {:?}", self.labels)?;
        writeln!(f, " signature = {}", self.signature)
    }
}

pub fn run(url: &str) -> Result<Vec<Cluster>, Error> {
    let mut client = Client::connect(url, NoTls)?;
    let rows = client.query(
        "SELECT cluster_id,size,labels,signature,event_ids,event_sources FROM cluster WHERE status_id=2",
        &[],
    )?;
    let mut clusters = vec![];
    for row in &rows {
        let eids: Vec<i64> = row.get(4);
        let srcs: Vec<String> = row.get(5);
        let event_ids = srcs.into_iter().zip(eids).collect::<Vec<(String, i64)>>();
        let c = Cluster {
            cluster_id: row.get(0),
            size: row.get(1),
            labels: row.get(2),
            signature: row.get(3),
            event_ids,
        };
        println!(
            "{}\t{}\t{:?}\t{}",
            c.cluster_id, c.size, c.labels, c.signature,
        );
        clusters.push(c);
    }
    Ok(clusters)
}
