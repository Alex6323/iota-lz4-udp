use std::error::Error;
use std::io::{Cursor, Read, Write};

use lz4::{Decoder, Encoder, EncoderBuilder};

use crate::constants::*;

pub trait CompressionAlgo {
    fn compress(&self, bytes: &[u8]) -> Result<Vec<u8>, Box<Error>>;
    fn decompress(&self, bytes: &[u8]) -> Result<Vec<u8>, Box<Error>>;
}

pub struct TrimFrag;

pub struct Lz4 {
    enc_level: u32,
}

impl Lz4 {
    pub fn new(enc_level: u32) -> Self {
        Lz4 { enc_level }
    }
}

impl CompressionAlgo for Lz4 {
    fn compress(&self, bytes: &[u8]) -> Result<Vec<u8>, Box<Error>> {
        let mut encoder = EncoderBuilder::new()
            .level(self.enc_level)
            .build(Vec::new())?;

        encoder.write_all(bytes)?;

        let (buf, result) = encoder.finish();
        result?;

        Ok(buf)
    }

    fn decompress(&self, bytes: &[u8]) -> Result<Vec<u8>, Box<Error>> {
        let mut decoder = Decoder::new(Cursor::new(bytes))?;
        let mut decompr = Vec::new();

        decoder.read_to_end(&mut decompr)?;

        Ok(decompr)
    }
}
const NOT_SIGNATURE_FRAGMENTS: usize = 324;

impl CompressionAlgo for TrimFrag {
    fn compress(&self, bytes: &[u8]) -> Result<Vec<u8>, Box<Error>> {
        // Count 0 bytes with the sig/msg fragment
        let compressed_sigfrag_size = {
            let mut size = SIGNATURE_FRAGMENTS.5;
            for i in (0..SIGNATURE_FRAGMENTS.5).rev() {
                if bytes[i] != 0 {
                    break;
                }
                size -= 1;
            }
            size
        };

        let mut compressed = vec![0; compressed_sigfrag_size + NOT_SIGNATURE_FRAGMENTS];
        compressed[0..compressed_sigfrag_size].copy_from_slice(&bytes[0..compressed_sigfrag_size]);
        compressed[compressed_sigfrag_size..compressed_sigfrag_size + NOT_SIGNATURE_FRAGMENTS]
            .copy_from_slice(&bytes[SIGNATURE_FRAGMENTS.5..PACKET_SIZE]);

        Ok(compressed)
    }

    fn decompress(&self, bytes: &[u8]) -> Result<Vec<u8>, Box<Error>> {
        let compressed_sigfrag_size = bytes.len() - NOT_SIGNATURE_FRAGMENTS;

        let mut decompressed = vec![0; PACKET_SIZE];
        decompressed[0..compressed_sigfrag_size]
            .copy_from_slice(&bytes[0..compressed_sigfrag_size]);
        decompressed[SIGNATURE_FRAGMENTS.5..PACKET_SIZE]
            .copy_from_slice(&bytes[compressed_sigfrag_size..bytes.len()]);

        Ok(decompressed)
    }
}

#[cfg(test)]
mod tests {
    use super::super::model::transaction::*;
    use super::*;

    #[test]
    fn lz4_compression_works() {
        let lz4 = Lz4::new(0);
        let tx = TransactionBuilder::default().message("Hello").build();
        let bytes = tx.as_bytes();

        let compressed = lz4
            .compress(&bytes)
            .expect("error compressing transaction bytes");

        let decompressed = lz4
            .decompress(&compressed)
            .expect("error decompressing transactions bytes");

        let tx2 = Transaction::from_tx_bytes(&decompressed);

        assert_eq!(tx.as_tryte_string(), tx2.as_tryte_string());
    }

    #[test]
    fn trim_frag_compression_works() {
        let trim_frag = TrimFrag;
        let tx = TransactionBuilder::default().message("Hello").build();
        let bytes = tx.as_bytes();

        let compressed = trim_frag
            .compress(&bytes)
            .expect("error compressing transaction bytes");

        println!("compressed = {}", compressed.len());

        let decompressed = trim_frag
            .decompress(&compressed)
            .expect("error decompressing transactions bytes");

        let tx2 = Transaction::from_tx_bytes(&decompressed);

        assert_eq!(tx.as_tryte_string(), tx2.as_tryte_string());

        use super::super::convert::ascii;
        let msg = ascii::from_tryte_string(&tx2.signature_fragments);
        println!("{}", msg);
    }

    #[test]
    fn ascii_converter_works() {
        use super::super::convert::ascii;

        let tx = TransactionBuilder::default().message("y3s7bxyrwS").build();
        let tx_bytes = tx.as_bytes();
        let trim_frag = TrimFrag;
        //trim_frag
        let msg = ascii::from_tryte_string(&tx.signature_fragments);
        println!("{}", msg);
    }
}
