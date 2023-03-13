use anyhow::{anyhow, Result};
use chrono::Utc;
use libc::timeval;
use pcap::{Capture, Linktype, Packet, PacketCodec, PacketHeader, Precision};
use std::{
    io::{Read, Seek, SeekFrom, Write},
    os::fd::AsRawFd,
    process::{exit, Command, Stdio},
};
use tempfile::tempfile;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketOwned {
    header: PacketHeader,
    data: Vec<u8>,
}

pub struct Codec;
impl PacketCodec for Codec {
    type Item = PacketOwned;

    fn decode(&mut self, packet: Packet) -> Self::Item {
        PacketOwned {
            header: *packet.header,
            data: packet.data.to_vec(),
        }
    }
}

fn main() {
    let packets = match read_packets("sample.pcap") {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{e:?}");
            exit(1);
        }
    };

    if let Ok(pcap) = make_pcap(&packets) {
        match run_tcpdump(&pcap) {
            Ok(s) => println!("{s}"),
            Err(e) => eprintln!("{e:?}"),
        }
    }
}

fn read_packets(path: &str) -> Result<Vec<Vec<u8>>> {
    let cap = pcap::Capture::from_file(path)?;
    let mut packets = Vec::new();
    for (seq, packet) in cap.iter(Codec).enumerate() {
        if let Ok(p) = packet {
            packets.push(p.data);
        }
        if seq == 5 {
            break;
        }
    }

    Ok(packets)
}

const A_BILLION: i64 = 1_000_000_000;
fn make_pcap(packets: &[Vec<u8>]) -> Result<Vec<u8>> {
    let mut pcapfile = tempfile()?;
    let fd = pcapfile.as_raw_fd();
    let cap = Capture::dead_with_precision(Linktype::ETHERNET, Precision::Nano)?;
    let mut file = unsafe { cap.savefile_raw_fd(fd)? };

    let now = Utc::now().timestamp_nanos();
    for packet in packets {
        let len = u32::try_from(packet.len()).unwrap_or_default();
        let header = PacketHeader {
            ts: timeval {
                tv_sec: now / A_BILLION,
                #[cfg(target_os = "linux")]
                tv_usec: now % A_BILLION,
                #[cfg(target_os = "macos")]
                tv_usec: i32::try_from(now % A_BILLION).unwrap_or_default(),
            },
            caplen: len,
            len,
        };
        let p = Packet {
            header: &header,
            data: packet,
        };
        file.write(&p);
    }
    file.flush()?;
    let mut buf = Vec::new();
    let _ = pcapfile.seek(SeekFrom::Start(0))?;
    let _ = pcapfile.read_to_end(&mut buf)?;
    Ok(buf)
}

fn run_tcpdump(pcap: &[u8]) -> Result<String> {
    let cmd = "/usr/sbin/tcpdump";
    let args = [
        "-n",
        "-X",
        "-tttt",
        "-v",
        "--time-stamp-precision",
        "nano",
        "-r",
        "-",
    ];
    let mut child = Command::new(cmd)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    if let Some(mut child_stdin) = child.stdin.take() {
        #[cfg(target_os = "macos")]
        child_stdin.write_all(&[0, 0, 0, 0])?;
        child_stdin.write_all(pcap)?;
        // std::thread::spawn(move || child_stdin.write(pcap));
    } else {
        return Err(anyhow!("failed to execute tcpdump"));
    }

    let output = child.wait_with_output()?;
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
    }
    Err(anyhow!("failed to run tcpdump"))
}
