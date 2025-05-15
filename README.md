# LC-3-VM
An implementation of a LC-3 virtual machine using Rust

# Dependencies

rust = "1.86.0"

ctrlc = "3.4.6"

termios = "0.3.3"

timeout-readwrite = "0.4.0"

thiserror = "2.0.12"

# How to use

Start by cloning this repo
This virtual machine runs LC-3 assembled code so you can:
* Run your own assembled code with `make run path=<path_to_your_image>` or `cargo run path_to_your_image`
* Run the 2408 image with `make 2048`
* Run the rogue image with `make rogue`

## Other comands

Use `make doc` to open the documentation
