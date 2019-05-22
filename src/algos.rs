use std::error::Error;
use std::io::{Cursor, Read, Write};

use lz4::{Decoder, Encoder, EncoderBuilder};

use crate::constants::*;

pub trait CompressionAlgo {
    fn compress(&self, bytes: &[u8]) -> Result<Vec<u8>, Box<Error>>;
    fn decompress(&self, bytes: &[u8]) -> Result<Vec<u8>, Box<Error>>;
}

/// LZ4 compression algo.
pub struct Lz4 {
    enc_level: u32,
}

/// Trims the signature message fragment only.
pub struct TrimFragment;

/// Trims all the transactions fields. It simply puts delimiter bytes between all fields.
pub struct TrimAll {
    offsets: Vec<(usize, usize)>,
}

impl TrimAll {
    const DELIMITER_BYTE: u8 = 0xFF;

    pub fn new() -> Self {
        let offsets = vec![
            (EXTRA_DATA_DIGEST.4 - 1, SIGNATURE_FRAGMENTS.5),
            (ADDRESS.4 - 1, EXTRA_DATA_DIGEST.5),
            (VALUE.4 - 1, ADDRESS.5),
            (ISSUANCE_TIMESTAMP.4 - 1, VALUE.5),
            (TIMELOCK_LOWER_BOUND.4 - 1, ISSUANCE_TIMESTAMP.5),
            (TIMELOCK_UPPER_BOUND.4 - 1, TIMELOCK_LOWER_BOUND.5),
            (BUNDLE_NONCE.4 - 1, TIMELOCK_UPPER_BOUND.5),
            (TRUNK_HASH.4 - 1, BUNDLE_NONCE.5),
            (BRANCH_HASH.4 - 1, TRUNK_HASH.5),
            (TAG.4 - 1, BRANCH_HASH.5),
            (ATTACHMENT_TIMESTAMP.4 - 1, TAG.5),
            (
                ATTACHMENT_TIMESTAMP_LOWER_BOUND.4 - 1,
                ATTACHMENT_TIMESTAMP.5,
            ),
            (
                ATTACHMENT_TIMESTAMP_UPPER_BOUND.4 - 1,
                ATTACHMENT_TIMESTAMP_LOWER_BOUND.5,
            ),
            (NONCE.4 - 1, ATTACHMENT_TIMESTAMP_UPPER_BOUND.5),
            (TRANSACTION_SIZE_BYTES - 1, NONCE.5),
        ];

        Self { offsets }
    }
}

impl CompressionAlgo for TrimAll {
    fn compress(&self, bytes: &[u8]) -> Result<Vec<u8>, Box<Error>> {
        let mut kept_bytes_list = Vec::with_capacity(self.offsets.len());
        let mut kept_bytes = 0;

        for (last, length) in &self.offsets {
            let (last, length) = (*last, *length);

            let mut is_empty_field = true;

            for i in 0..length {
                if bytes[last - i] != 0 {
                    kept_bytes_list.push((last + 1 - length, length - i));
                    kept_bytes += length - i + 1; // +1 for the delimiter byte
                    is_empty_field = false;
                    break;
                }
            }

            if is_empty_field {
                kept_bytes_list.push((last + 1 - length, 0));
                kept_bytes += 1;
            }
        }

        let mut compressed = vec![0; kept_bytes];

        let mut start = 0;
        for (offset, length) in kept_bytes_list {
            compressed[start..start + length].copy_from_slice(&bytes[offset..offset + length]);

            start += length;

            compressed[start] = TrimAll::DELIMITER_BYTE;
            start += 1;
        }
        Ok(compressed)
    }

    fn decompress(&self, bytes: &[u8]) -> Result<Vec<u8>, Box<Error>> {
        let mut decompressed = vec![0; PACKET_SIZE];

        let mut src = 0;
        let mut dst = 0;
        let mut delta = 0;
        let mut index = 0;

        for i in 0..bytes.len() {
            let is_delimiter = bytes[i] == TrimAll::DELIMITER_BYTE;

            if is_delimiter {
                decompressed[dst..dst + delta].copy_from_slice(&bytes[src..src + delta]);

                index += 1;
                if index >= self.offsets.len() {
                    break;
                }

                dst = self.offsets[index].0 + 1 - self.offsets[index].1;
                src = i + 1;
                delta = 0;
            } else {
                delta += 1;
            }
        }

        Ok(decompressed)
    }
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

impl CompressionAlgo for TrimFragment {
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

    // first we need to convert mainnet trytes to ict trytes
    // NOTE: length is already 2754 (instead of 2673 on the mainnet)
    const MAINNET_TRYTES: &str = "SEGQSWYCJHRLJYEGZLRYQAZPLVRAYIWGWJUMFFX99UZUKBQNFYAOQLOFARIKNEBKDRHJJWDJARXTNPHPAODJRSGJBVVYBVJHZALJWDCJHZRSACOVCVVAVHZVTPFTAJWVGFSVLSYXHNNXEGSMJHDBZKGFQNYJJJBAPDHFFGZ9POSOMWTDPGXI9KQRLMUVWNEQDANMXROVORJVALWVGDDJAFOOBXUKVCCIVXSSHZUCZV9XVBASLWX9NXPWGMGYCRD9ILQMKIGPBGGMKAIJKNALBLABATYFVIRBKTXTWNUZAUXRASB9EEIQHWBD9ZYUDBUPBSWXVYXQXECRCHQAYH9ZBUZBASPOIGBSGWJYFKFRITUBVMCYGCMAPTXOIWEVTUXSUOUPTUQOPMMPUTHXMOP9CW9THAZXEPMOMNEOBLUBPOAIOBEBERRZCIKHSTDWUSUPUWNJOCLNZDCEKWWAAJDPJXJEHHSYFN9MH9BGUDQ9CSZBIHRC9PSQJPGKH9ILZDWUWLEKWFKUFFFIMOQKRMKOYXEJHXLCEGCGGKHGJUHOXINSWCKRNMUNAJDCVLZGEBII9ASTYFTDYDZIZSNHIWHSQ9HODQMVNDKMKHCFDXIIGDIVJSBOOE9GRIXCD9ZUTWCUDKFTETSYSRBQABXCXZFOWQMQFXHYZWD9JZXUWHILMRNWXSGUMIIXZYCTWWHCWMSSTCNSQXQXMQPTM9MOQMIVDYNNARDCVNQEDTBKWOIOSKPKPOZHJGJJGNYWQWUWAZMBZJ9XEJMRVRYFQPJ9NOIIXEGIKMMN9DXYQUILRSCSJDIDN9DCTFGQIYWROZQIEQTKMRVLGGDGA9UVZPNRGSVTZYAPMWFUWDEUULSEEGAGITPJQ9DBEYEN9NVJPUWZTOTJHEQIXAPDOICBNNCJVDNM9YRNXMMPCOYHJDUFNCYTZGRCBZKOLHHUK9VOZWHEYQND9WUHDNGFTAS99MRCAU9QOYVUZKTIBDNAAPNEZBQPIRUFUMAWVTCXSXQQIYQPRFDUXCLJNMEIKVAINVCCZROEWEX9XVRM9IHLHQCKC9VLK9ZZWFBJUZKGJCSOPQPFVVAUDLKFJIJKMLZXFBMXLMWRSNDXRMMDLE9VBPUZB9SVLTMHA9DDDANOKIPY9ULDWAKOUDFEDHZDKMU9VMHUSFG9HRGZAZULEJJTEH9SLQDOMZTLVMBCXVNQPNKXRLBOUCCSBZRJCZIUFTFBKFVLKRBPDKLRLZSMMIQNMOZYFBGQFKUJYIJULGMVNFYJWPKPTSMYUHSUEXIPPPPPJTMDQLFFSFJFEPNUBDEDDBPGAOEJGQTHIWISLRDAABO9H9CSIAXPPJYCRFRCIH9TVBZKTCK9SPQZUYMUOKMZYOMPRHRGF9UAKZTZZG9VVVTIHMSNDREUOUOSLKUHTNFXTNSJVPVWCQXUDIMJIAMBPXUGBNDTBYPKYQYJJCDJSCTTWHOJKORLHGKRJMDCMRHSXHHMQBFJWZWHNUHZLYOAFQTRZFXDBYASYKWEVHKYDTJIAUKNCCEPSW9RITZXBOFKBAQOWHKTALQSCHARLUUGXISDMBVEUKOVXTKTEVKLGYVYHPNYWKNLCVETWIHHVTBWT9UPMTQWBZPRPRSISUBIBECVDNIZQULAGLONGVFLVZPBMHJND9CEVIXSYGFZAGGN9MQYOAKMENSEOGCUNKEJTDLEDCD9LGKYANHMZFSSDDZJKTKUJSFL9GYFDICTPJEPDSBXDQTARJQEWUVWDWSQPKIHPJONKHESSQH9FNQEO9WUCFDWPPPTIQPWCVDYTTWPLCJJVYNKE9ZEJNQBEJBMDBLNJKQDOQOHVS9VY9UPSU9KZVDFOESHNRRWBK9EZCYALAUYFGPCEWJQDXFENSNQEAUWDXJGOMCLQUQWMCPHOBZZ9SZJ9KZXSHDLPHPNYMVUJQSQETTN9SG9SIANJHWUYQXZXAJLYHCZYRGITZYQLAAYDVQVNKCDIYWAYBAFBMAYEAEAGMTJGJRSNHBHCEVIQRXEFVWJWOPU9FPDOWIFL9EWGHICRBNRITJDZNYACOGTUDBZYIYZZWAOCDBQFFNTTSTGKECWTVWZSPHX9HNRUYEAEWXENEIDLVVFMZFVPUNHMQPAIOKVIBDIHQIHFGRJOHHONPLGBSJUD9HHDTQQUZN9NVJYOAUMXMMOCNUFLZ9BAJSZMDMPQHPWSFVWOJQDPHV9DYSQPIBL9LYZHQKKOVF9TFVTTXQEUWFQSLGLVTGK99VSUEDXIBIWCQHDQQSQLDHZ9999999999999999999TRINITY99999999999999999999TNXSQ9D99A99999999B99999999MXKZAGDGKVADXOVCAXEQYZGOGQKDLKIUPYXIL9PXYBQXGYDEGNXTFURSWQYLJDFKEV9VVBBQLTLHIBTFYOGBHPUUHS9CKWSAPIMDIRNSUJ9CFPGKTUFAGQYVMFKOZSVAHIFJXWCFBZLICUWF9GNDZWCOWDUIIZ9999OXNRVXLBKJXEZMVABR9UQBVSTBDFSAJVRRNFEJRL9UFTOFPJHQMQKAJHDBIQAETS9OUVTQ9DSPAOZ9999TRINITY99999999999999999999LPZYMWQME999999999MMMMMMMMMDTIZE9999999999999999999999";

    fn get_example_trytes() -> String {
        let sig_msg_frag = MAINNET_TRYTES.get(0..2187).unwrap();
        let extra_data_digest = MAINNET_TRYTES.get((2187 + 162)..(2187 + 162 + 81)).unwrap(); //copied bundle hash
        let addr_value_tag_timestamps = MAINNET_TRYTES.get(2187..(2187 + 162)).unwrap();
        let rest = MAINNET_TRYTES.get((2187 + 162 + 81)..).unwrap();

        format!(
            "{}{}{}{}",
            sig_msg_frag, extra_data_digest, addr_value_tag_timestamps, rest
        )
    }

    #[test]
    fn lz4_compression_works() {
        let lz4 = Lz4::new(0);
        //let tx = TransactionBuilder::default().message("Hello").build();
        let tx = Transaction::from_tryte_string(&get_example_trytes());
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
    fn trim_fragment_compression_works() {
        let trim_frag = TrimFragment;
        let tx = Transaction::from_tryte_string(&get_example_trytes());
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
    }

    #[test]
    fn trim_all_compression_works() {
        let trim_all = TrimAll::new();
        let tx = Transaction::from_tryte_string(&get_example_trytes());
        let bytes = tx.as_bytes();

        let compressed = trim_all
            .compress(&bytes)
            .expect("error compressing transaction bytes");

        println!("compressed = {}", compressed.len());

        let decompressed = trim_all
            .decompress(&compressed)
            .expect("error decompressing transactions bytes");

        println!("decompressed = {}", decompressed.len());

        let tx2 = Transaction::from_tx_bytes(&decompressed);

        assert_eq!(tx.as_tryte_string(), tx2.as_tryte_string());

        //use super::super::convert::ascii;
        //let msg = ascii::from_tryte_string(&tx2.signature_fragments);
        //println!("{}", msg);
    }

    #[test]
    fn bench_create_1000_compressions() {
        /*
        let start = Instant::now();
        for i in 0..1000 {
            let hash = Transaction::default().message(&i.to_string()).get_hash();
        }
        let stop = start.elapsed();

        println!(
            "{} ms",
            stop.as_secs() * 1000 + u64::from(stop.subsec_millis())
        );
        */
    }
}
