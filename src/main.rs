use structopt::StructOpt;

mod algos;
mod constants;
mod convert;
mod model;
mod receiver;
mod sender;
mod time;

use crate::algos::*;
use crate::constants::{MAX_MESSAGE_LENGTH, MIN_MESSAGE_LENGTH};

#[derive(Debug, StructOpt)]
struct Args {
    #[structopt(subcommand)]
    mode: EndpointMode,
}

#[derive(Debug, StructOpt)]
enum EndpointMode {
    #[structopt(
        name = "send",
        about = "Send IOTA transactions to a receiver endpoint."
    )]
    Send {
        /// Port of the sender.
        #[structopt(short, default_value = "1337")]
        send_port: u16,

        /// Port of the receiver.
        #[structopt(short, default_value = "1338")]
        recv_port: u16,

        /// Size of the payload stored in the signature message fragment.
        #[structopt(short, default_value = "1458")]
        payload_size: usize,

        /// The compression algorithm.
        #[structopt(subcommand)]
        algo: Algo,
    },

    #[structopt(
        name = "recv",
        about = "Receive IOTA transactions from a sender endpoint."
    )]
    Recv {
        /// Receiving port.
        #[structopt(short, default_value = "1338")]
        recv_port: u16,

        #[structopt(subcommand)]
        algo: Algo,
    },
}

#[derive(Debug, StructOpt)]
enum Algo {
    #[structopt(name = "lz4", about = "Use Lz4 compression algorithm.")]
    Lz4 {
        #[structopt(short, default_value = "0")]
        compression_level: u32,
    },

    #[structopt(name = "trimfrag", about = "Use Trim-Frag compression algorithm.")]
    TrimFrag,

    #[structopt(name = "trimall", about = "Use Trim-All compression algorithm.")]
    TrimAll,
}

fn main() {
    let cli = Args::from_args();
    println!("Running the tool with the following options:");
    println!("{:?}", cli);

    match cli.mode {
        EndpointMode::Recv { recv_port, algo } => {
            //
            let algo: Box<dyn CompressionAlgo> = match algo {
                Algo::Lz4 { compression_level } => Box::new(Lz4::new(compression_level)),
                Algo::TrimFrag => Box::new(TrimFragment),
                Algo::TrimAll => Box::new(TrimAll::new()),
            };

            crate::receiver::start(recv_port, algo);
        }
        EndpointMode::Send {
            send_port,
            recv_port,
            payload_size,
            algo,
        } => {
            //
            let algo: Box<dyn CompressionAlgo> = match algo {
                Algo::Lz4 { compression_level } => Box::new(Lz4::new(compression_level)),
                Algo::TrimFrag => Box::new(TrimFragment),
                Algo::TrimAll => Box::new(TrimAll::new()),
            };

            let payload_size = if payload_size < MIN_MESSAGE_LENGTH {
                MIN_MESSAGE_LENGTH
            } else if payload_size > MAX_MESSAGE_LENGTH {
                MAX_MESSAGE_LENGTH
            } else {
                payload_size
            };

            crate::sender::start(send_port, recv_port, payload_size, algo);
        }
    }
}
