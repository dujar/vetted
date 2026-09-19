// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import {CastNames} from "../src/CastNames.sol";
import {DemoStockToken} from "../src/DemoStockToken.sol";
import {DemoBeacon} from "../src/DemoBeacon.sol";
import {DemoBeaconProxy} from "../src/DemoBeaconProxy.sol";

/// @notice Shared fixture: deploys the replica cast exactly as the deploy
/// script does, and pins the calibrated constants (slots + probe selectors,
/// packages/shared/abi.ts + calibration_4663.json) used by both test suites.
abstract contract Cast is Test {
    bytes32 constant IMPL_SLOT = 0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc;
    bytes32 constant BEACON_SLOT = 0xa3f0ad74e5423aebfd80d3ef4346578335a9a72aeaee59ff6cb3582b35133d50;

    bytes4 constant SEL_PAUSED = 0x5c975abb; // paused()
    bytes4 constant SEL_BLOCKLIST = 0xfbac3951; // isBlocked(address)
    bytes4 constant SEL_IMPLEMENTATION = 0x5c60da1b; // implementation()

    DemoStockToken internal implV1;
    DemoStockToken internal implV2; // upgrade target (verify loose end 2)
    DemoBeacon internal beacon;
    DemoBeaconProxy internal proxy;
    DemoStockToken internal token; // address(proxy)

    address internal admin = makeAddr("admin");
    address internal alice = makeAddr("alice");
    address internal buyer = makeAddr("buyer");

    function _deployCast() internal {
        implV1 = new DemoStockToken();
        implV2 = new DemoStockToken();
        beacon = new DemoBeacon(address(implV1), admin);
        proxy = new DemoBeaconProxy(address(beacon));
        DemoStockToken(address(proxy)).initialize(CastNames.NAME, CastNames.SYMBOL, admin);
        token = DemoStockToken(address(proxy));
    }

    // --- raw-call helpers (probe fidelity: calibrated selectors, no ABI sugar) ---

    function _call(address target, bytes memory data) internal view returns (bool ok, bytes memory ret) {
        (ok, ret) = target.staticcall(data);
    }

    /// @dev true when the selector DISPATCHES (body may still revert with
    /// data); a missing selector reverts with EMPTY returndata. Uses a plain
    /// `call` — a staticcall would make write-paths (transfer/approve) revert
    /// EMPTY mid-body, indistinguishable from a missing selector.
    function _selectorKnown(address target, bytes4 sel, bytes memory args) internal returns (bool) {
        if (target.code.length == 0) return false;
        (bool ok, bytes memory ret) = target.call(bytes.concat(sel, args));
        return ok || ret.length > 0;
    }

    function _answeredFalse(address target, bytes memory data) internal view returns (bool) {
        (bool ok, bytes memory ret) = _call(target, data);
        return ok && ret.length == 32 && abi.decode(ret, (uint256)) == 0;
    }

    // --- fingerprint check primitives (FINGERPRINT.md) ---

    function _slot(address target, bytes32 slot) internal view returns (bytes32) {
        return vm.load(target, slot);
    }

    /// @dev F3 — resolve the beacon from forwarder bytecode SHAPE: find a
    /// 20-byte embedded address whose staticcall implementation() returns an
    /// address with code. Never codehash equality. Returns found=false when
    /// nothing answers (twins) — no revert, so callers can assert either way.
    function _findEmbeddedBeacon(address forwarder) internal view returns (bool found, address resolved) {
        bytes memory code = forwarder.code;
        for (uint256 i = 0; i + 20 <= code.length; i++) {
            address candidate;
            assembly {
                candidate := shr(96, mload(add(add(code, 32), i)))
            }
            if (candidate.code.length == 0) continue;
            (bool ok, bytes memory ret) = candidate.staticcall(abi.encodeWithSelector(SEL_IMPLEMENTATION));
            if (ok && ret.length >= 32) {
                address impl = abi.decode(ret, (address));
                if (impl.code.length > 0) return (true, candidate);
            }
        }
        return (false, address(0));
    }

    /// @dev F1 — read the solc version out of the CBOR metadata tail:
    /// ... 0x64 "solc" 0x43 <major> <minor> <patch>. Returns (major, minor).
    function _solcVersion(bytes memory code) internal pure returns (uint8 major, uint8 minor) {
        uint256 n = code.length;
        require(n >= 16, "no metadata");
        for (uint256 i = n - 16; i + 6 < n; i++) {
            if (
                code[i] == 0x73 && code[i + 1] == 0x6f && code[i + 2] == 0x6c && code[i + 3] == 0x63
                    && code[i + 4] == 0x43
            ) {
                return (uint8(code[i + 5]), uint8(code[i + 6]));
            }
        }
        revert("fingerprint: no solc metadata tail");
    }
}
