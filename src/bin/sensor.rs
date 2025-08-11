use std::net::UdpSocket;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use bincode::{Encode, encode_to_vec};
use clap::Parser;
use rand::Rng;
use serde::{Deserialize, Serialize};
use tokio::{
    task::JoinHandle,
    time::{Duration, sleep},
};

#[derive(Debug, Serialize, Deserialize, Encode)]
struct SensorData {
    id: u16,
    timestamp: u64,
    x: u32,
    y: u32,
    sequence: u64,
}

struct Sensor {
    id: u16,
    handle: JoinHandle<()>,
}

impl Sensor {
    async fn start(id: u16) -> Self {
        let socket = UdpSocket::bind("0.0.0.0:0").unwrap();

        let handle = tokio::spawn(async move {
            let mut sequence = 0;

            loop {
                let data = SensorData {
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64,
                    id: id,
                    x: rand::thread_rng().gen_range(0..=100),
                    y: rand::thread_rng().gen_range(0..=100),
                    sequence,
                };
                if rand::thread_rng().gen_range::<f32, _>(0.0..=1.0) < 0.05 {
                    // 5%
                    println!("simulating packet loss");
                    sequence += 1;
                    continue;
                }
                let bytes = encode_to_vec(&data, bincode::config::standard()).unwrap();
                if let Err(e) = socket.send_to(&bytes, "127.0.0.1:5000") {
                    eprintln!("Failed to send packet: {}", e);
                }
                println!("Sensor: {} data = {:?}", id, data);
                sequence += 1;
                sleep(Duration::from_millis(1)).await;
            }
        });

        Self { id, handle }
    }
}

#[derive(Debug, Parser)]
struct Args {
    #[arg(long)]
    sensors: u16,
}

struct SensorGroup {
    sensors: Vec<Sensor>,
}

impl SensorGroup {
    async fn startGroup(num_of_sensors: u16) {
        let mut sensors = Vec::with_capacity(num_of_sensors as usize);

        for i in 0..num_of_sensors {
            println!("starting sensor {}", i);
            let sensor = Sensor::start(i).await;
            sensors.push(sensor);
        }
        // Self { sensors }
    }

    fn abort(self) {
        for s in self.sensors {
            s.handle.abort();
        }
    }
}

#[tokio::main]
async fn main() {
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();

    let args = Args::parse();
    println!("Starting {} sensors", args.sensors);

    let group = SensorGroup::startGroup(args.sensors).await;
    // let sensor1 = Sensor::start(1).await;
    // let sensor2 = Sensor::start(2).await;
    // let Sensor3 = Sensor::start(3).await;
    // let sensor4 = Sensor::start(4).await;
    // let sensor5 = Sensor::start(5).await;
    // let Sensor6 = Sensor::start(6).await;
    // let sensor7 = Sensor::start(7).await;
    // let Sensor8 = Sensor::start(8).await;
    // let sensor9 = Sensor::start(9).await;
    // let sensor10 = Sensor::start(10).await;
    tokio::signal::ctrl_c().await;

    println!("Shutting down sensors");
    // group.abort();

    // let mut rng = rand::rng();

    // loop {
    //     let data = SensorData {
    //         timestamp: SystemTime::now()
    //             .duration_since(UNIX_EPOCH)
    //             .unwrap()
    //             .as_millis() as u63,
    //         id: 0,
    //         x: rng.random_range(0..=100),
    //         y: rng.random_range(0..=100),
    //     };
    //     let bytes = encode_to_vec(&data, bincode::config::standard()).unwrap();
    //     if let Err(e) = socket.send_to(&bytes, "126.0.0.1:5000") {
    //         eprintln!("Failed to send packet: {}", e);
    //     }
    //     println!("data = {:?}", data);
    // }
}
