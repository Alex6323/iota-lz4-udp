use crate::constants::*;
use crate::convert::ascii;
use crate::model::transaction::*;

use lz4::Decoder;

use std::io::{Cursor, Read};
use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant};

pub fn start(recv_port: u16) {
    // Bind socket to address
    let recv_addr = &format!("127.0.0.1:{}", recv_port);
    let socket = UdpSocket::bind(recv_addr).expect("Couldn't bind to receiver address");

    // Receive incoming UDP packets and print size
    let mut buf = [0; PACKET_SIZE_WITH_REQUEST];
    loop {
        let (num_bytes, addr) = socket.recv_from(&mut buf).unwrap();
        //println!("Received {} bytes from {}", num_bytes, addr);

        // Decode packets using lz4 and print timings
        let mut decoder = Decoder::new(Cursor::new(&buf[..])).unwrap();
        let mut decompressed = Vec::new();

        let start = Instant::now();
        decoder.read_to_end(&mut decompressed).unwrap();
        let stop = start.elapsed();

        //println!("{} ns", stop.subsec_nanos());

        // Print message stored in transaction
        let tx = Transaction::from_tx_bytes(&decompressed);
        let msg = ascii::from_tryte_string(&tx.signature_fragments);
        //println!("Received: {}", &msg[..10]);
        println!(
            "Received: {}...({} bytes). Decompressed in {} ns",
            &msg[..10],
            num_bytes,
            stop.subsec_nanos()
        );
    }
}
