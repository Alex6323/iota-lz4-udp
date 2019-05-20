use std::env;

use structopt::StructOpt;

mod constants;
mod convert;
mod crypto;
mod model;
mod receiver;
mod sender;
mod time;
mod types;

use crate::types::*;

#[derive(StructOpt)]
struct CliArgs {
    #[structopt(
        name = "message length",
        help = "Sets the size of the message.",
        default_value = "280"
    )]
    message_length: usize,

    #[structopt(
        name = "sender port",
        help = "Sets the sender UDP port.",
        default_value = "1337"
    )]
    sender_port: u16,

    //#[structopt(name = "sender", raw(required = "true"))]
    #[structopt(name = "sender", help = "Toggles whether this is the sender.")]
    sender: bool,

    // dependent on 'sender'
    #[structopt(
        name = "receiver port",
        help = "Sets the receivers UDP port.",
        default_value = "1338"
    )]
    receiver_port: u16,

    #[structopt(subcommand)]
    algo: CliSubcommands,
}

#[derive(StructOpt)]
#[structopt(
    name = "compr-poc",
    about = "Various compression algos for Ict transactions."
)]
enum CliSubcommands {
    #[structopt(name = "lz4", about = "Use the lz4 compression algorithm.")]
    Lz4 {
        #[structopt(short = "e", default_value = "0")]
        encoder_level: u32,
    },

    #[structopt(
        name = "trim-frag",
        about = "Trim the signature message fragment if possible."
    )]
    TrimFlag,

    #[structopt(name = "trim-all", about = "Trim all transaction fields if possible.")]
    TrimAll,
}

const MIN_MESSAGE_LENGTH: usize = 10;
const MAX_MESSAGE_LENGTH: usize = 1458;

//=====================================================
// YOU CAN PLAY WITH THOSE VALUES
// * try increasing the encoder level
// * try to change the message length
//=====================================================
/*
pub const ENCODER_LEVEL: u32 = 0;
pub const MESSAGE_LENGTH: usize = 10; //1458 = max number of bytes for signature/message fragment
                                      //=====================================================
const SENDER_ARG: &str = "sender";
const SENDER_PORT: u16 = 1337;
const RECVER_ARG: &str = "receiver";
const RECVER_PORT: u16 = 1338;
*/

fn main() {
    let cli = CliArgs::from_args();

    let algo: Box<dyn CompressionAlgo> = match cli.algo {
        CliSubcommands::Lz4 { encoder_level } => Box::new(Lz4::new(encoder_level)),
        CliSubcommands::TrimFlag => Box::new(TrimFrag),
        CliSubcommands::TrimAll => panic!("Not supported yet"),
    };

    let message_length = if cli.message_length < MIN_MESSAGE_LENGTH {
        MIN_MESSAGE_LENGTH
    } else if cli.message_length > MAX_MESSAGE_LENGTH {
        MAX_MESSAGE_LENGTH
    } else {
        cli.message_length
    };

    if cli.sender {
        crate::sender::start(cli.sender_port, cli.receiver_port, message_length, algo);
    } else {
        crate::receiver::start(cli.receiver_port, algo);
    }
}
