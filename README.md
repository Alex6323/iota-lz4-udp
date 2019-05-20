# About
This tool creates IOTA transactions, compresses them using the fast compression algorithm [lz4](https://en.wikipedia.org/wiki/LZ4_(compression_algorithm)) and then sends them via UDP to a receiver which decompresses and decodes them. Both ends measure and print the timings for compressing/decompressing the transactions. 

# How to build it
1. If you haven't already install Rust using `rustup`, Rust's toolchain manager. You can download it [here](https://www.rust-lang.org/tools/install).
2. Clone this repository, cd into it, and build the project using `cargo build --release`.

# How to run it
Open two terminals and change into the *release* directory respectively. Currently the following compression algorithms are supported:
* Lz4
* TrimFrag (trims the empty space in the signature message fragment)

Running two endpoints sending lz4 compressed IOTA transactions can be as simple as typing: 
```Bash
./itxc recv lz4 
```
in one terminal, and 
```Bash
./itxc send lz4
```
in the other terminal. You can however, customize your test by adjusting the ports, changing the compression level for lz4, and choose a different payload size. If you want to see all options of a subcommand simply type:
```Bash
./itxc [SUBCOMMAND] --help
```

# Contact 
Feel free to contact me on the IOTA Discord server. My handle is /alex/#6323. Have fun :)
