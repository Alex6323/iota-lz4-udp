use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

use std::iter;
use std::net::UdpSocket;
use std::thread;
use std::time::{Duration, Instant};

use crate::algos::CompressionAlgo;
use crate::constants::MIN_MESSAGE_LENGTH;
use crate::model::transaction::*;

// MODIFY THIS VALUE TO CHANGE PAUSE BETWEEN SENDS
const SLEEP_MS: u64 = 3000;

pub fn start(send_port: u16, recv_port: u16, msg_length: usize, algo: Box<dyn CompressionAlgo>) {
    //
    let recv_addr = &format!("127.0.0.1:{}", recv_port);

    // Create a UDP socket
    let send_addr = &format!("127.0.0.1:{}", send_port);
    let socket = UdpSocket::bind(send_addr).expect("Couldn't bind to sender address");
    let mut rng = thread_rng();

    // Send compressed UDP packets and print events to terminal
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
            "Sent {} bytes ({}) - Compressed {} bytes in {} ns ({:.2}).",
            compressed.len(),
            &msg[..MIN_MESSAGE_LENGTH],
            tx_bytes.len(),
            stop.subsec_nanos(),
            tx_bytes.len() as f64 / compressed.len() as f64,
        );

        sleep(SLEEP_MS);
    }
}

fn sleep(ms: u64) {
    thread::sleep(Duration::from_millis(ms));
}
