// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import {ForwarderPatch} from "../src/ForwarderPatch.sol";
import {RawHull} from "../src/RawHull.sol";
import {MockBeacon} from "../src/MockBeacon.sol";
import {MockTokenImpl} from "../src/MockTokenImpl.sol";

/// The mock pattern tokens must be indistinguishable from the genuine
/// Robinhood stock tokens to the guard's from-contract probes:
/// extraction by SHAPE finds the beacon, `paused()` answers on the PROXY,
/// `isBlocked` answers ONLY on the BEACON (reverts via proxy/impl), and the
/// upgrade beat moves the resolved implementation. Ground truth:
/// `spike/evidence/calibration_4663.json` + the guard's native tests.
contract MockPatternTest is Test {
    // probe selectors - pinned in packages/shared/abi.ts PROBE_SELECTORS
    bytes4 constant SEL_PAUSED = 0x5c975abb;
    bytes4 constant SEL_IS_BLOCKED = 0xfbac3951;
    bytes4 constant SEL_IMPLEMENTATION = 0x5c60da1b;

    address internal implV1;
    address internal implV2;
    address internal beacon;
    address internal tokenA;
    address internal tokenB;
    MockTokenImpl internal plain;

    address internal alice = makeAddr("alice");
    address internal bob = makeAddr("bob");

    function setUp() public {
        implV1 = address(new MockTokenImpl("Mock Stock (Vetted pattern)", "MSA"));
        implV2 = address(new MockTokenImpl("Mock Stock (Vetted pattern)", "MSA"));
        beacon = address(new MockBeacon(implV1));
        tokenA = address(new RawHull(ForwarderPatch.patchBeacon(beacon)));
        tokenB = address(new RawHull(ForwarderPatch.patchBeacon(beacon)));
        plain = new MockTokenImpl("Plain Mock Stock (no pattern)", "MSP");
    }

    // ---- the forwarder runtime is the genuine code, byte for byte ----

    function test_genuine_runtime_matches_spike_evidence() public {
        // tripwire: the shipped constant never drifts from the live-captured
        // bytes (the same fixture the guard's native tests pin)
        bytes memory rawB = bytes(vm.readFile("../../../spike/evidence/p_proxy.hex"));
        // strip the file's trailing newline(s) - parseBytes refuses them
        while (rawB.length > 0 && (rawB[rawB.length - 1] == "\n" || rawB[rawB.length - 1] == "\r")) {
            assembly ("memory-safe") {
                mstore(rawB, sub(mload(rawB), 1))
            }
        }
        bytes memory evidence = vm.parseBytes(string(rawB));
        assertEq(ForwarderPatch.GENUINE_RUNTIME.length, 283, "genuine forwarder is 283 bytes");
        assertEq(evidence.length, 283, "evidence file is 283 bytes");
        assertEq(evidence, ForwarderPatch.GENUINE_RUNTIME, "runtime drift vs spike evidence");
    }

    function test_genuine_beacon_is_what_the_shape_finds() public {
        assertEq(
            ForwarderPatch.tryExtractBeacon(ForwarderPatch.GENUINE_RUNTIME),
            ForwarderPatch.GENUINE_BEACON,
            "the genuine runtime must resolve to the calibrated beacon"
        );
    }

    function test_patch_moves_only_the_embedded_beacon() public {
        bytes memory patched = ForwarderPatch.patchBeacon(bob);
        bytes memory genuine = ForwarderPatch.GENUINE_RUNTIME;
        assertEq(patched.length, genuine.length, "patching must not resize the runtime");
        (uint256 i, bool found) = ForwarderPatch.locateShape(genuine);
        assertTrue(found, "genuine runtime must hold the shape");
        // every byte outside the 20-byte address span [i+13, i+33) is unchanged
        for (uint256 j = 0; j < genuine.length; ++j) {
            if (j >= i + 13 && j < i + 33) continue;
            assertEq(patched[j], genuine[j], "byte changed outside the beacon span");
        }
        assertEq(ForwarderPatch.tryExtractBeacon(patched), bob);
    }

    // ---- what the guard sees on-chain ----

    function test_deployed_forwarder_code_holds_the_shape() public {
        assertEq(tokenA.code.length, 283, "hull must deploy exactly the runtime");
        assertEq(ForwarderPatch.tryExtractBeacon(tokenA.code), beacon, "guard extraction on-chain");
        assertEq(ForwarderPatch.tryExtractBeacon(tokenB.code), beacon, "guard extraction on-chain");
    }

    function test_plain_token_has_no_shape() public {
        assertEq(
            ForwarderPatch.tryExtractBeacon(address(plain).code),
            address(0),
            "patternless token must degrade, not resolve"
        );
    }

    function test_proxy_delegates_erc20_and_state_is_per_proxy() public {
        // mint THROUGH the forwarder (delegatecall storage) - the impl's own
        // storage stays empty, which is the genuine shared-impl semantics
        MockTokenImpl(tokenA).mint(alice, 100e18);
        assertEq(MockTokenImpl(tokenA).balanceOf(alice), 100e18);
        assertEq(MockTokenImpl(tokenB).balanceOf(alice), 0, "storage must be per-proxy");
        vm.prank(alice);
        assertTrue(MockTokenImpl(tokenA).transfer(bob, 1e18));
        assertEq(MockTokenImpl(tokenA).balanceOf(bob), 1e18);
        vm.prank(alice);
        assertTrue(MockTokenImpl(tokenA).approve(bob, 5e18));
        vm.prank(bob);
        assertTrue(MockTokenImpl(tokenA).transferFrom(alice, bob, 2e18));
        assertEq(MockTokenImpl(tokenA).balanceOf(bob), 3e18);
    }

    function test_identity_is_per_proxy_and_init_is_once() public {
        MockTokenImpl(tokenA).init("Mock Stock A", "MSA");
        MockTokenImpl(tokenB).init("Mock Stock B", "MSB");
        assertEq(MockTokenImpl(tokenA).name(), "Mock Stock A", "identity through proxy A");
        assertEq(MockTokenImpl(tokenB).name(), "Mock Stock B", "identity through proxy B");
        assertEq(plain.name(), "Plain Mock Stock (no pattern)", "standalone identity");
        vm.expectRevert(abi.encodeWithSelector(MockTokenImpl.AlreadyInitialized.selector));
        MockTokenImpl(tokenA).init("again", "AGAIN");
    }

    function test_isBlocked_answers_on_the_beacon_only() public {
        (bool okBeacon, bool blocked) = _probe(beacon, SEL_IS_BLOCKED, alice);
        assertTrue(okBeacon, "isBlocked must answer on the beacon");
        assertFalse(blocked, "a fresh account is unblocked");
        MockBeacon(beacon).setBlocked(alice, true);
        (, blocked) = _probe(beacon, SEL_IS_BLOCKED, alice);
        assertTrue(blocked, "the guard's GUARD_BLOCKLISTED input");
        // the same selector REVERTS through the proxy and on the impl - the
        // calibrated behavior that forces the guard to resolve the beacon
        (bool okProxy,) = _probe(tokenA, SEL_IS_BLOCKED, alice);
        assertFalse(okProxy, "isBlocked must revert on the proxy");
        (bool okImpl,) = _probe(implV1, SEL_IS_BLOCKED, alice);
        assertFalse(okImpl, "isBlocked must revert on the impl");
        (bool okPlain,) = _probe(address(plain), SEL_IS_BLOCKED, alice);
        assertFalse(okPlain, "isBlocked must revert on the patternless token");
    }

    function test_paused_answers_on_the_proxy_per_token() public {
        (bool ok, bool paused) = _probe(tokenA, SEL_PAUSED, address(0));
        assertTrue(ok, "paused() must answer on the proxy");
        assertFalse(paused);
        MockTokenImpl(tokenA).pause(); // through the proxy: per-token storage
        (, paused) = _probe(tokenA, SEL_PAUSED, address(0));
        assertTrue(paused, "the guard's GUARD_PAUSED input");
        (, bool pausedB) = _probe(tokenB, SEL_PAUSED, address(0));
        assertFalse(pausedB, "pause state is per-proxy, not shared");
    }

    function test_upgrade_moves_resolution_and_keeps_state() public {
        MockTokenImpl(tokenA).mint(alice, 100e18);
        MockBeacon(beacon).upgradeTo(implV2);
        assertEq(MockBeacon(beacon).implementation(), implV2);
        // the guard's resolution: STATICCALL implementation() on the beacon
        (bool ok, bytes memory ret) = beacon.staticcall(abi.encodeWithSelector(SEL_IMPLEMENTATION));
        assertTrue(ok);
        assertEq(abi.decode(ret, (address)), implV2);
        // balances live in the proxy's storage - an impl swap preserves them
        assertEq(MockTokenImpl(tokenA).balanceOf(alice), 100e18, "upgrade must not touch proxy state");
    }

    function test_transfer_reverts_while_paused() public {
        MockTokenImpl(tokenA).pause();
        vm.expectRevert(abi.encodeWithSelector(MockTokenImpl.Paused.selector));
        MockTokenImpl(tokenA).transfer(bob, 1);
    }

    function test_erc20_returns_true_bool() public {
        MockTokenImpl(tokenA).mint(alice, 10e18);
        vm.prank(alice);
        assertTrue(MockTokenImpl(tokenA).transfer(bob, 1e18), "the guard accepts ret[31]==1");
    }

    /// The exact probe sequence the guard runs for ONE token (record aside -
    /// that is the stylus registry, exercised by the native tests): paused on
    /// the proxy, beacon from code shape, isBlocked + implementation on the
    /// beacon. On a fresh system every reading must pass the decision table.
    function test_probe_sequence_mirrors_the_guard() public {
        (bool pausedOk, bool paused) = _probe(tokenA, SEL_PAUSED, address(0));
        assertTrue(pausedOk && !paused);
        address resolved = ForwarderPatch.tryExtractBeacon(tokenA.code);
        assertEq(resolved, beacon);
        (bool blockedOk, bool blocked) = _probe(resolved, SEL_IS_BLOCKED, alice);
        assertTrue(blockedOk && !blocked);
        (bool implOk, bytes memory ret) = resolved.staticcall(abi.encodeWithSelector(SEL_IMPLEMENTATION));
        assertTrue(implOk);
        assertEq(abi.decode(ret, (address)), implV1);
    }

    function _probe(address target, bytes4 sel, address who)
        internal
        view
        returns (bool ok, bool value)
    {
        bytes memory ret;
        (ok, ret) = target.staticcall(abi.encodeWithSelector(sel, who));
        value = ok && ret.length == 32 && ret[31] != 0;
    }
}
