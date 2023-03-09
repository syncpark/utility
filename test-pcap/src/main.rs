use anyhow::Result;
use chrono::Utc;
use libc::timeval;
use pcap::{Capture, Linktype, Packet, PacketCodec, PacketHeader};
use std::process::exit;

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

    if let Err(e) = run(&packets) {
        eprintln!("{e:?}");
    }
}

fn read_packets(path: &str) -> Result<Vec<Vec<u8>>> {
    let cap = pcap::Capture::from_file(path)?;
    let mut packets = Vec::new();
    for (seq, packet) in cap.iter(Codec).enumerate() {
        if let Ok(p) = packet {
            println!("{p:?}");
            packets.push(p.data);
        }
        if seq == 5 {
            break;
        }
    }

    Ok(packets)
}

const A_BILLION: i64 = 1_000_000_000;
fn run(packets: &[Vec<u8>]) -> Result<()> {
    let new_pcap = Capture::dead_with_precision(Linktype::ETHERNET, pcap::Precision::Nano)?;
    let mut file = new_pcap.savefile("new_file.pcap")?;
    let now = Utc::now().timestamp_nanos();
    for packet in packets {
        let len = u32::try_from(packet.len()).unwrap_or_default();
        let header = PacketHeader {
            ts: timeval {
                tv_sec: now / A_BILLION,
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

    Ok(())
}
