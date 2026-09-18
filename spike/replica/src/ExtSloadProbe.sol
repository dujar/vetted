// SPDX-License-Identifier: MIT
// Criterion (c) capability probe: does this EVM support the EXTSLOAD opcode
// (0x5c)? Runtime bytecode injected via vm.etch (see test) — solc has no
// portable raw-opcode escape (verbatim_ is unavailable outside the IR
// pipeline), so the raw bytes are hand-assembled:
//   6020 7f<slot=0...0> 5c 52 6020 6000 f3
//   PUSH1 32; PUSH32 0; EXTSLOAD; MSTORE(0,v); RETURN(0,32)
// On a chain without the opcode the CALL reverts (undefined opcode); with it,
// the call returns 32 bytes. Deployment success is not the verdict — the call is.
pragma solidity ^0.8.28;

library ExtSloadProbe {
    /// EXTSLOAD canary runtime (42 bytes, no args needed).
    function runtime() internal pure returns (bytes memory) {
        return hex"60207f00000000000000000000000000000000000000000000000000000000000000005c5260206000f3";
    }
}
