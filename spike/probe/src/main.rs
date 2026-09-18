// Template bin target (stylus-tools 0.10.9 templates/contract/src/main.rs) —
// cargo stylus (constructor check + export-abi) requires a runnable bin; the
// real contract logic lives in lib.rs.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]

#[cfg(not(any(test, feature = "export-abi")))]
#[unsafe(no_mangle)]
pub extern "C" fn main() {}

#[cfg(feature = "export-abi")]
fn main() {
    probe::print_from_args();
}
