use crate::constants::*;
use crate::convert::ascii;
use crate::model::transaction::*;
use crate::types::*;

//use lz4::Decoder;

//use std::io::{Cursor, Read};
use std::net::UdpSocket;
use std::time::Instant;

pub fn start(recv_port: u16, algo: Box<dyn CompressionAlgo>) {
    // Bind socket to address
    let recv_addr = &format!("127.0.0.1:{}", recv_port);
    let socket = UdpSocket::bind(recv_addr).expect("Couldn't bind to receiver address");

    // Receive incoming UDP packets and print size
    let mut buf = [0; PACKET_SIZE];
    loop {
        let (num_bytes, _) = socket.recv_from(&mut buf).unwrap();

        // Decode packets using lz4 and print timings
        //let mut decoder = Decoder::new(Cursor::new(&buf[..])).unwrap();
        //let mut decompressed = Vec::new();
        //decoder.read_to_end(&mut decompressed).unwrap();

        let start = Instant::now();
        let decompressed = algo
            .decompress(&buf[0..num_bytes])
            .expect("error decompressing transaction");
        let stop = start.elapsed();

        // Print message stored in transaction
        let tx = Transaction::from_tx_bytes(&decompressed);

        let msg = ascii::from_tryte_string(&tx.signature_fragments);
        println!(
            "Received: {}...({} bytes). Decompressed in {} ns",
            &msg[..10],
            num_bytes,
            stop.subsec_nanos()
        );
    }
}
