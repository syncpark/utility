use anyhow::Result;
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use std::{
    env,
    fs::{self, read_to_string, File},
    io::{BufRead, BufReader, BufWriter, Read, Write},
    path::Path,
    process::exit,
    time::Instant,
};

fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.len() == 1 {
        println!("Usage: {} <input-file-to-compress>", env!("CARGO_PKG_NAME"));
        exit(0);
    }
    let start = Instant::now();
    let name = if let Some(input) = args.get(1) {
        match compress_2(input) {
            Ok(name) => Some(name),
            Err(e) => {
                eprintln!("Error: {e:?}");
                None
            }
        }
    } else {
        None
    };
    let duration = start.elapsed();
    println!("Time elapsed to compress: {duration:?}");

    if let Some(output) = name {
        let start = Instant::now();
        if let Err(e) = uncompress_1(&output) {
            eprintln!("Error: {e:?}");
        }
        let duration = start.elapsed();
        println!("Time elapsed to uncompress: {duration:?}");
    }
}

#[allow(unused)]
fn compress_1(input: &str) -> Result<String> {
    let input = Path::new(input);
    let infile = File::open(input)?;
    let reader = BufReader::new(infile);
    let mut filename = input
        .file_name()
        .expect("fail to parse name")
        .to_string_lossy()
        .to_string();
    filename.push_str(".gz.1");
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    for line in reader.lines().flatten() {
        let buf = format!("{line}\n");
        encoder.write_all(buf.as_bytes())?;
    }
    let tidb = encoder.finish()?;
    let mut writer = BufWriter::new(File::create(&filename)?);
    writer.write_all(&tidb)?;
    Ok(filename)
}

#[allow(unused)]
fn compress_2(input: &str) -> Result<String> {
    let input = Path::new(input);
    let mut filename = input
        .file_name()
        .expect("fail to parse name")
        .to_string_lossy()
        .to_string();
    filename.push_str(".gz.2");
    let buf = read_to_string(input)?;
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(buf.as_bytes())?;
    let tidb_compressed = encoder.finish()?;
    fs::write(&filename, tidb_compressed)?;
    Ok(filename)
}

#[allow(unused)]
fn uncompress_1(output: &str) -> Result<()> {
    let input = Path::new(output);
    let compressed_tidb = fs::read(input)?;
    let mut gz = GzDecoder::new(&compressed_tidb[..]);
    let mut s = String::new();
    gz.read_to_string(&mut s);
    let mut filename = input
        .file_name()
        .expect("fail to parse name")
        .to_string_lossy()
        .to_string();
    filename.push_str(".uncompressed.1");
    let mut writer = BufWriter::new(File::create(&filename)?);
    writer.write_all(s.as_bytes())?;
    Ok(())
}
