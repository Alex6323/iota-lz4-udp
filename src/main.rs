use std::env;

mod constants;
mod convert;
mod crypto;
mod model;
mod receiver;
mod sender;
mod time;

//=====================================================
// YOU CAN PLAY WITH THOSE VALUES TO SIMULATE DIFFERENT
// MORE OR LESS REDUNDANT DATA AND TO SEE EFFECTS ON
// TIMINGS WHEN CHANGING THE ENCODER LEVEL.
//=====================================================
pub const ENCODER_LEVEL: u32 = 0;
pub const MESSAGE_LENGTH: usize = 1000; //1458 = max number of bytes for signature/message fragment
                                        //=====================================================
const SENDER_ARG: &str = "sender";
const SENDER_PORT: u16 = 1337;
const RECVER_ARG: &str = "receiver";
const RECVER_PORT: u16 = 1338;

fn main() {
    let args = env::args().collect::<Vec<String>>();
    let app_name = &args[0];

    match args.len() {
        2 => match &args[1][..] {
            SENDER_ARG => {
                crate::sender::start(SENDER_PORT, RECVER_PORT, MESSAGE_LENGTH, ENCODER_LEVEL)
            }
            RECVER_ARG => crate::receiver::start(RECVER_PORT),
            _ => panic!("Unknown argument: {}", args[1]),
        },
        _ => panic!(
            "Wrong number of arguments.  usage: {} sender|receiver",
            app_name
        ),
    }
    println!("Hello, world!");
}
