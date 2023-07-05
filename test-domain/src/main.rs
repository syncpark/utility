use anyhow::Result;
use std::{
    collections::HashMap,
    env,
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::Path,
};

fn main() {
    let args = env::args().collect::<Vec<_>>();
    if let Some(input) = args.get(1) {
        if let Err(e) = run(Path::new(input)) {
            eprintln!("Error: {e}");
        }
    } else {
        println!("Usage: {} <input-file-name>", env!("CARGO_PKG_NAME"));
    }
}

fn run(input: &Path) -> Result<()> {
    let infile = File::open(input)?;
    let reader = BufReader::new(infile);

    let mut filename = input
        .file_name()
        .expect("fail to parse name")
        .to_string_lossy()
        .to_string();
    filename.push_str(".rewrited");
    let mut writer = BufWriter::new(File::create(&filename)?);

    let mut domain_statistics = HashMap::new();
    for domain in reader.lines().flatten() {
        let words: Vec<_> = domain.split('.').collect();

        let post = if words.len() > 3 {
            words.get(words.len() - 3..)
        } else if words.len() > 2 {
            words.get(words.len() - 2..)
        } else {
            words.get(..)
        };

        if let Some(key) = post {
            domain_statistics
                .entry(key.join("."))
                .and_modify(|c| *c += 1)
                .or_insert(1_u32);
        }
    }

    let mut stats = domain_statistics.into_iter().collect::<Vec<_>>();
    stats.sort_by(|a, b| a.1.cmp(&b.1));
    for (d, cnt) in stats {
        let _r = writeln!(writer, "{cnt}\t{d}");
    }

    Ok(())
}
