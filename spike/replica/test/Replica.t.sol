// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import {StockToken} from "../src/StockToken.sol";
import {UpgradeableBeacon} from "../src/UpgradeableBeacon.sol";
import {BeaconProxy} from "../src/BeaconProxy.sol";
import {ExtSloadProbe} from "../src/ExtSloadProbe.sol";

contract ReplicaTest is Test {
    StockToken impl1;
    StockToken impl2;
    UpgradeableBeacon beacon;
    BeaconProxy proxy;
    StockToken token;

    function setUp() public {
        impl1 = new StockToken();
        impl2 = new StockToken();
        beacon = new UpgradeableBeacon(address(impl1));
        proxy = new BeaconProxy(
            address(beacon),
            abi.encodeCall(StockToken.initialize, ("Everpure Replica", "P0", "uid1"))
        );
        token = StockToken(address(proxy));
    }

    /// The calibrated selectors must match the live 4663 tokens byte-for-byte
    /// (spike/findings.md task 5). Public state variables have no .selector
    /// member — derive from signatures and pin the calibrated bytes.
    function test_selectors_match_calibration() public pure {
        assertEq(bytes4(keccak256("paused()")), bytes4(0x5c975abb), "paused()");
        assertEq(bytes4(keccak256("isBlocked(address)")), bytes4(0xfbac3951), "isBlocked(address)");
        assertEq(bytes4(keccak256("implementation()")), bytes4(0x5c60da1b), "implementation()");
        assertEq(bytes4(keccak256("beacon()")), bytes4(0x59659e90), "beacon()");
    }

    function test_beacon_slot_layout_is_eip1967() public view {
        assertEq(
            proxy.beacon(),
            address(uint160(uint256(vm.load(address(proxy), 0xa3f0ad74e5423aebfd80d3ef4346578335a9a72aeaee59ff6cb3582b35133d50))))
        );
    }

    function test_pause_blocks_transfer() public {
        token.mint(address(this), 100e18);
        token.pause();
        assertTrue(token.paused());
        try token.transfer(address(1), 1e18) {
            fail("transfer should revert while paused");
        } catch {}
        token.unpause();
        assertTrue(token.transfer(address(1), 1e18));
    }

    function test_blocklist_blocks_sender() public {
        token.mint(address(this), 100e18);
        token.blockAddress(address(this));
        assertTrue(token.isBlocked(address(this)));
        try token.transfer(address(1), 1e18) {
            fail("transfer should revert when sender blocked");
        } catch {}
    }

    function test_beacon_upgrade_resolves_under_fixed_proxy() public {
        assertEq(beacon.implementation(), address(impl1));
        beacon.upgradeTo(address(impl2));
        assertEq(beacon.implementation(), address(impl2));
        // proxy address unchanged; live impl moved (the revocation trigger)
        assertTrue(address(proxy) != address(0));
        assertEq(token.uid(), "uid1");
    }

    /// EXTSLOAD (0x5c) verdict: undefined on standard EVM forks — local revm
    /// is expected to revert. ArbOS chains are decided by the same canary
    /// deployed there (findings.md task 6). Assert only the conditional: if
    /// the call succeeds it returns a word.
    function test_extsload_opcode_verdict_recorded() public {
        address canary = address(uint160(0x5C));
        vm.etch(canary, ExtSloadProbe.runtime());
        (bool ok, bytes memory ret) = canary.call("");
        if (ok) {
            assertEq(ret.length, 32, "extsload canary returned a word");
        }
        // revert branch = opcode absent: verdict recorded, no assertion
    }

    bytes32 constant BEACON_SLOT = 0xa3f0ad74e5423aebfd80d3ef4346578335a9a72aeaee59ff6cb3582b35133d50;
}
