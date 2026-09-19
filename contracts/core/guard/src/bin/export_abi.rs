//! Generates the Solidity interface — `cargo stylus export-abi` runs this
//! bin with the `export-abi` feature (template shape).
use guard::Guard;
use stylus_sdk::abi::export::print_from_args;

fn main() {
    print_from_args::<Guard>();
}
