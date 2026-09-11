//! Placeholder contract proving the harness end to end: `sol_storage!` +
//! `#[entrypoint]` compile to wasm32, and `stylus_sdk::testing`'s native TestVM
//! runs unit tests in CI without an EVM. No deploy — real contracts land in
//! step 3 (`contracts/core/registry`, `contracts/core/guard`).

// Allow `cargo stylus export-abi` to generate a main function (template shape).
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

// The SDK's storage macros expand to `alloc` paths.
#[macro_use]
extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use stylus_sdk::{alloy_primitives::U256, prelude::*};

sol_storage! {
    #[entrypoint]
    pub struct Hello {
        string greeting;
        uint256 greeted;
    }
}

#[public]
impl Hello {
    /// Current greeting; never empty (defaults to "hello").
    pub fn greet(&self) -> String {
        let g = self.greeting.get_string();
        if g.is_empty() {
            "hello".to_string()
        } else {
            g
        }
    }

    /// Set the greeting; returns how many times the greeting has been set.
    pub fn set_greeting(&mut self, value: String) -> U256 {
        self.greeting.set_str(&value);
        let n = self.greeted.get() + U256::from(1);
        self.greeted.set(n);
        n
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use stylus_sdk::testing::*;

    #[test]
    fn default_greeting_is_hello() {
        let vm = TestVM::default();
        let hello = Hello::from(&vm);
        assert_eq!(hello.greet(), "hello");
    }

    #[test]
    fn set_greeting_updates_and_counts() {
        let vm = TestVM::default();
        let mut hello = Hello::from(&vm);
        assert_eq!(hello.set_greeting("vetted".into()), U256::from(1));
        assert_eq!(hello.greet(), "vetted");
        assert_eq!(hello.set_greeting("again".into()), U256::from(2));
        assert_eq!(hello.greet(), "again");
    }
}
