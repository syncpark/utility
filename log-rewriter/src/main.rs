use anyhow::Result;
use std::{
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

const URI_FIELD: usize = 10;

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

    for line in reader.lines().flatten() {
        let s = line.split('\t').collect::<Vec<_>>();
        if let Some(uri) = s.get(URI_FIELD) {
            let pos = end_of_hostname_in_uri(uri);
            if pos > 0 {
                let new_uri = new_uri(uri, pos);
                let pre = s[..URI_FIELD].join("\t");
                let post = s[URI_FIELD + 1..].join("\t");
                let _r = writeln!(writer, "{pre}\t{new_uri}\t{post}");
            } else {
                let _r = writeln!(writer, "{line}");
            }
        }
    }

    Ok(())
}

fn end_of_hostname_in_uri(uri: &str) -> usize {
    let pos = if uri.contains("http://") {
        "http://".len()
    } else if uri.contains("https://") {
        "https://".len()
    } else {
        0
    };
    if pos > 0 {
        if let Some(p) = uri[pos..].find('/') {
            return pos + p;
        }
    }
    pos
}

const EMPTY_URI: &str = "-";
fn new_uri(uri: &str, pos: usize) -> &str {
    if pos >= uri.len() - 1 {
        EMPTY_URI
    } else {
        uri.get(pos..).unwrap_or(uri)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uri_replace() {
        let uri = "/item/N7604600/3.jpg";
        assert_eq!(end_of_hostname_in_uri(uri), 0);

        let uri = "http://career.jbnu.ac.kr/images/main/banner_07.jpg";
        let pos = "http://career.jbnu.ac.kr/".len() - 1;
        assert_eq!(end_of_hostname_in_uri(uri), pos);
        assert_eq!(new_uri(uri, pos), "/images/main/banner_07.jpg");

        let uri = "-";
        assert_eq!(end_of_hostname_in_uri(uri), 0);

        let uri = "http://fs.arumnet.com/image/N2720OFNEX//item/N7684000/3.jpg";
        let pos = "http://fs.arumnet.com/".len() - 1;
        assert_eq!(end_of_hostname_in_uri(uri), pos);
        assert_eq!(new_uri(uri, pos), "/image/N2720OFNEX//item/N7684000/3.jpg");

        let uri = "https://search.naver.com/";
        let pos = "https://search.naver.com/".len() - 1;
        assert_eq!(end_of_hostname_in_uri(uri), pos);
        assert_eq!(new_uri(uri, pos), "-");

        let uri =
            "http://gms.ahnlab.com/jk?c=62&p=ZK5dQRPRiTcLgwQQBX8wGEPtj_6HIAIjiTff+9BsUcY=&k=1";
        let pos = "http://gms.ahnlab.com/".len() - 1;
        assert_eq!(end_of_hostname_in_uri(uri), pos);
        assert_eq!(
            new_uri(uri, pos),
            "/jk?c=62&p=ZK5dQRPRiTcLgwQQBX8wGEPtj_6HIAIjiTff+9BsUcY=&k=1"
        );
    }
}
