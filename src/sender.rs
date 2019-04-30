use lz4::{Encoder, EncoderBuilder};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

use std::io::Write;
use std::iter;
use std::net::UdpSocket;
use std::thread;
use std::time::{Duration, Instant};

use crate::constants::*;
use crate::model::transaction::*;

// MODIFY THIS VALUE TO CHANGE PAUSE BETWEEN SENDS
const SLEEP_MS: u64 = 3000;

pub fn start(send_port: u16, recv_port: u16, msg_length: usize, enc_level: u32) {
    let send_addr = &format!("127.0.0.1:{}", send_port);
    let recv_addr = &format!("127.0.0.1:{}", recv_port);

    // create UDP socket
    let socket = UdpSocket::bind(send_addr).expect("Couldn't bind to sender address");
    let mut rng = thread_rng();

    // create transactions
    loop {
        // Create a random message from alphanumberic chars
        let msg: String = iter::repeat(())
            .map(|()| rng.sample(Alphanumeric))
            .take(msg_length)
            .collect();

        // Create a transaction from that message
        let tx = Transaction::default().message(&msg);

        // convert it to bytes
        let tx_bytes = tx.as_bytes();

        let mut encoder = EncoderBuilder::new()
            .level(enc_level)
            .build(Vec::new())
            .expect("couldn't create lz4 encoder");

        let start = Instant::now();
        // compress it using lz4
        encoder
            //.write(&tx_bytes)
            .write_all(&tx_bytes)
            .expect("couldn't compress transaction");

        let (buf, result) = encoder.finish();
        let stop = start.elapsed();

        println!(
            "Sent ascii msg/tx: {}... Compressed in {} ns",
            &msg[..10],
            stop.subsec_nanos()
        );
        result.unwrap();

        // send it to the receiver
        socket
            .send_to(&buf, recv_addr)
            .expect("Couldn't send packet to receiver");

        sleep(SLEEP_MS);
    }
}

fn sleep(ms: u64) {
    thread::sleep(Duration::from_millis(ms));
}
