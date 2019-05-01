# About
This tool creates IOTA transactions, compresses them using the fast compression algorithm [lz4](https://en.wikipedia.org/wiki/LZ4_(compression_algorithm)) and then sends them via UDP to a receiver which decompresses and decodes them. Both ends measure and print the timings for compressing/decompressing the transactions. 

# How to build it
1. If you haven't already install Rust using `rustup`, Rust's toolchain manager. You can download it [here](https://www.rust-lang.org/tools/install).
2. Clone this repository, cd into it, and build the project using `cargo build --release`.

# How to run it
Open two terminals and change into the *release* directory respectively. Then in one terminal start the receiver and in the other terminal start the sender by typing:
```Bash
   ./iota_lz4_udp receiver 
   ./iota_lz4_udp sender 
```
Per default this tool uses the ports `1337` for the sender and `1338` for the receiver. Change those in `main.rs` if you get error messages saying the port is already in use. Play with the constants in `main.rs` to see the effects when changing the compression level or when changing the payload inside of the signature/message fragment.

# Contact 
Feel free to contact me on the IOTA Discord server. My handle is /alex/#6323. Have fun :)
