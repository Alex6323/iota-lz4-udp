use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

//use std::io::Write;
use std::iter;
use std::net::UdpSocket;
use std::thread;
use std::time::{Duration, Instant};

use crate::model::transaction::*;
use crate::types::CompressionAlgo;

// MODIFY THIS VALUE TO CHANGE PAUSE BETWEEN SENDS
const SLEEP_MS: u64 = 3000;

pub fn start(send_port: u16, recv_port: u16, msg_length: usize, algo: Box<dyn CompressionAlgo>) {
    let send_addr = &format!("127.0.0.1:{}", send_port);
    let recv_addr = &format!("127.0.0.1:{}", recv_port);

    // Create UDP socket
    let socket = UdpSocket::bind(send_addr).expect("Couldn't bind to sender address");

    // Create an RNG
    let mut rng = thread_rng();

    loop {
        // Create a random message from alphanumberic chars
        let msg: String = iter::repeat(())
            .map(|()| rng.sample(Alphanumeric))
            .take(msg_length)
            .collect();

        // Create a transaction from that message
        let tx = Transaction::default().message(&msg);

        // Convert it to bytes
        let tx_bytes = tx.as_bytes();

        // Compress bytes
        let start = Instant::now();
        let compressed = algo
            .compress(&tx_bytes[..])
            .expect("error compressing transaction");
        let stop = start.elapsed();

        // Send it to the receiver
        socket
            .send_to(&compressed, recv_addr)
            .expect("Couldn't send packet to receiver");

        println!(
            "Sent ascii msg/tx: {}...({:.2}) Compressed in {} ns",
            &msg[..10],
            tx_bytes.len() as f64 / compressed.len() as f64,
            stop.subsec_nanos()
        );

        sleep(SLEEP_MS);
    }
}

fn sleep(ms: u64) {
    thread::sleep(Duration::from_millis(ms));
}
