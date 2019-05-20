use crate::algos::*;
use crate::constants::*;
use crate::convert::ascii;
use crate::model::transaction::*;

use std::net::UdpSocket;
use std::time::Instant;

pub fn start(recv_port: u16, algo: Box<dyn CompressionAlgo>) {
    // Bind socket to address
    let recv_addr = &format!("127.0.0.1:{}", recv_port);
    let socket = UdpSocket::bind(recv_addr).expect("Couldn't bind to receiver address");

    // Process incoming UDP packets and print events to terminal
    let mut buf = [0; PACKET_SIZE];
    loop {
        // Block on receiving a new UDP packet
        let (num_bytes, _) = socket.recv_from(&mut buf).unwrap();

        // Measure how long decompression takes
        let start = Instant::now();
        let decompressed = algo
            .decompress(&buf[0..num_bytes])
            .expect("error decompressing transaction");
        let stop = start.elapsed();

        // Print message stored in transaction
        let tx = Transaction::from_tx_bytes(&decompressed);
        let msg = ascii::from_tryte_string(&tx.signature_fragments);
        println!(
            "Received {} bytes ({}) - Decompressed in {} ns",
            num_bytes,
            &msg[..MIN_MESSAGE_LENGTH],
            stop.subsec_nanos()
        );
    }
}
