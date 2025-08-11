use bincode::{Decode, decode_from_slice};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::UdpSocket;

#[derive(Decode, Serialize, Deserialize)]
struct SensorData {
    id: u16,
    timestamp: u64,
    x: u32,
    y: u32,
    sequence: u64,
}

struct ReveiverState {
    last_sequence: HashMap<u16, u64>, // sensor id -> last_sequence
    dropped_count: u64,
    out_of_order_count: u64,
}

fn main() {
    let socket = UdpSocket::bind("127.0.0.1:5000").unwrap();
    let mut state = ReveiverState {
        last_sequence: HashMap::new(),
        dropped_count: 0,
        out_of_order_count: 0,
    };

    let mut sum = 0;

    let mut buf = [0u8; 1024];

    loop {
        match socket.recv_from(&mut buf) {
            Ok((len, src)) => {
                match decode_from_slice::<SensorData, _>(&buf[..len], bincode::config::standard()) {
                    Ok((data, _consumed)) => {
                        if let Some(&last_seq) = state.last_sequence.get(&data.id) {
                            if data.sequence != last_seq + 1 {
                                state.out_of_order_count += 1;
                                println!("OUT OF ORDER: got {} after {}", data.sequence, last_seq);
                            } else {
                                let dropped = data.sequence - last_seq - 1;
                                state.dropped_count += 1;
                                println!(
                                    "DROPPED {} packets! ({} -> {})",
                                    dropped, last_seq, data.sequence
                                );
                            }
                        }
                        state.last_sequence.insert(data.id, data.sequence);
                        println!(
                            "From {}: id: {:?} x: {:?}, y: {:?}, seq: {:?}",
                            src, data.id, data.x, data.y, data.sequence
                        );
                        for _ in 0..100 {
                            sum += data.x;
                        }
                    }
                    Err(e) => eprintln!("Failed to decode: {}", e),
                }
            }
            Err(e) => eprintln!("Failed to recieve: {}", e),
        }
    }
}
