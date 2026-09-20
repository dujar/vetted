// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

/// The genuine Robinhood stock-token forwarder, byte for byte, and the
/// calibrated beacon SHAPE it embeds (plan task 4/5, Revised notes 2/4).
///
/// The mock pattern tokens are NOT Solidity-written proxies: their runtime
/// code IS this 283-byte forwarder (captured live on Robinhood Chain 4663,
/// `spike/evidence/p_proxy.hex`) with only the embedded 20-byte beacon
/// address replaced. That guarantees the guard's shape-based extraction
/// (contracts/core/guard `extract_beacon_from_code`: PUSH32 `0x7f` + 12 zero
/// bytes + 20-byte address, `implementation()` selector 0x5c60da1b within
/// the following 16 bytes) resolves the mock beacon exactly as it resolves
/// the genuine tokens. Shape-based by design - never codehash equality (the
/// step-6 replicas embed a different beacon at the same shape).
///
/// The two scan functions below mirror the guard's extractor algorithm
/// opcode-for-opcode so the mock tests double as a cross-language tripwire.
library ForwarderPatch {
    /// The genuine 283-byte forwarder runtime - byte-identical to
    /// `spike/evidence/p_proxy.hex` (live 4663) and to the guard's native-test
    /// fixture. Structure: STATICCALL implementation() (0x5c60da1b) on the
    /// embedded beacon, DELEGATECALL to the result; NO sstore - the proxy
    /// never writes storage (spike/evidence/calibration_4663.json).
    bytes public constant GENUINE_RUNTIME =
        hex"6080604052600a600c565b005b60186014601a565b609d565b565b5f7f000000000000000000000000e10b6f6b275de231345c20d14ab812db62151b006001600160a01b0316635c60da1b6040518163ffffffff1660e01b8152600401602060405180830381865afa1580156076573d5f5f3e3d5ffd5b505050506040513d601f19601f820116820180604052508101906098919060ba565b905090565b365f5f375f5f365f845af43d5f5f3e80801560b6573d5ff35b3d5ffd5b5f6020828403121560c9575f5ffd5b81516001600160a01b038116811460de575f5ffd5b939250505056fea2646970667358221220e296a217a10765339b402e91ead95bcbcf74e678d7c756705f097d88cc2554b064736f6c63430008210033";

    /// The beacon embedded in the genuine runtime (calibration_4663.json -
    /// both genuine tokens point at this same beacon).
    address public constant GENUINE_BEACON = 0xe10b6f6B275de231345c20D14Ab812db62151b00;

    /// `implementation()` selector bytes - the shape anchor after the
    /// embedded PUSH32 word (guard: SEL_IMPLEMENTATION_BYTES).
    bytes4 private constant SEL_IMPLEMENTATION = 0x5c60da1b;

    error ShapeNotFound();
    error PatchFailed();

    /// Extract the embedded beacon the way the guard does. Returns
    /// address(0) when the shape is absent (the guard's `None` → degrade).
    function tryExtractBeacon(bytes memory code) internal pure returns (address) {
        (uint256 i, bool found) = locateShape(code);
        if (!found) return address(0);
        uint160 raw;
        assembly ("memory-safe") {
            raw := shr(96, mload(add(add(code, 0x20), add(i, 13))))
        }
        return address(raw);
    }

    /// Locate the calibrated shape: first PUSH32 (`0x7f`) whose immediate is
    /// a zero-padded 20-byte address, anchored by the implementation()
    /// selector within the following 16 bytes. Returns the index of the
    /// `0x7f` opcode, or `found == false`.
    function locateShape(bytes memory code) internal pure returns (uint256 i, bool found) {
        uint256 len = code.length;
        while (i + 33 <= len) {
            if (code[i] == 0x7f && _zeroPadded(code, i)) {
                uint256 windowEnd = i + 49;
                if (windowEnd > len) windowEnd = len;
                for (uint256 k = i + 33; k + 4 <= windowEnd; ++k) {
                    if (code[k] == SEL_IMPLEMENTATION[0] && code[k + 1] == SEL_IMPLEMENTATION[1]
                        && code[k + 2] == SEL_IMPLEMENTATION[2] && code[k + 3] == SEL_IMPLEMENTATION[3]) {
                        return (i, true);
                    }
                }
            }
            unchecked {
                ++i;
            }
        }
        return (0, false);
    }

    /// The genuine runtime with the embedded beacon replaced - the mock
    /// tokens' forwarder. Byte-identical to the genuine code except the
    /// 20-byte address span; re-extracted, the result must be exactly
    /// `beacon` (tripwire - never ship a token the guard cannot resolve).
    function patchBeacon(address beacon) internal pure returns (bytes memory patched) {
        patched = GENUINE_RUNTIME;
        (uint256 i, bool found) = locateShape(patched);
        if (!found) revert ShapeNotFound();
        assembly ("memory-safe") {
            // overwrite the 20-byte immediate, preserving the 12 bytes that
            // follow it inside the same 32-byte word
            let p := add(add(patched, 0x20), add(i, 13))
            mstore(p, or(and(mload(p), 0xffffffffffffffffffffffff), shl(96, beacon)))
        }
        if (tryExtractBeacon(patched) != beacon) revert PatchFailed();
    }

    /// `code[i+1 .. i+13]` all zero (the PUSH32 word's 12-byte zero prefix).
    function _zeroPadded(bytes memory code, uint256 i) private pure returns (bool) {
        for (uint256 j = 1; j < 13; ++j) {
            if (code[i + j] != 0) return false;
        }
        return true;
    }
}
