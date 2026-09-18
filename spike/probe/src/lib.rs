#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#[macro_use]
extern crate alloc;

use stylus_sdk::{alloy_primitives::U256, prelude::*};

sol_storage! {
    #[entrypoint]
    pub struct Probe {
        uint256 status;
    }
}

#[public]
impl Probe {
    pub fn paused_selector(&self) -> u32 {
        0x5c975abb
    }
}
