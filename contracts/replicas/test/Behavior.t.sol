// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import {Cast} from "./Cast.t.sol";
import {DemoBeacon} from "../src/DemoBeacon.sol";
import {DemoStockToken} from "../src/DemoStockToken.sol";
import {IAccessControl} from "@openzeppelin/contracts/access/IAccessControl.sol";

/// @notice Demo-beat behaviors (plan task 6) + the probe-target acceptance
/// test (plan Revised note 2): paused() answers on the proxy, isBlocked(buyer)
/// answers on the BEACON — the step-2 probe semantics must pass verbatim.
contract BehaviorTest is Cast {
    function setUp() public {
        _deployCast();
        vm.startPrank(admin);
        token.mint(alice, 100e18);
        vm.stopPrank();
    }

    // --- transfers settle when unpaused and unblocked ---

    function test_transfer_settles() public {
        vm.prank(alice);
        assertTrue(token.transfer(buyer, 5e18));
        assertEq(token.balanceOf(buyer), 5e18);
    }

    // --- global pause (state lives on the beacon, probe on the proxy) ---

    function test_pause_stops_transfers_and_reads_on_proxy() public {
        vm.prank(admin);
        beacon.pause();
        assertTrue(token.paused()); // proxy answers — beacon state
        vm.prank(alice);
        vm.expectRevert(DemoStockToken.TokenPaused.selector);
        token.transfer(buyer, 1e18);
        vm.prank(admin);
        beacon.unpause();
        assertFalse(token.paused());
        vm.prank(alice);
        assertTrue(token.transfer(buyer, 1e18));
    }

    function test_pause_stops_mint() public {
        vm.prank(admin);
        beacon.pause();
        vm.prank(admin);
        vm.expectRevert(DemoStockToken.TokenPaused.selector);
        token.mint(buyer, 1e18);
    }

    // --- hidden blocklist (state lives on the beacon) ---

    function test_blocklisted_sender_reverts() public {
        vm.prank(admin);
        beacon.setBlocked(alice, true);
        assertTrue(beacon.isBlocked(alice));
        vm.prank(alice);
        vm.expectRevert(DemoStockToken.SenderBlocked.selector);
        token.transfer(buyer, 1e18);
        vm.prank(admin);
        beacon.setBlocked(alice, false);
        vm.prank(alice);
        assertTrue(token.transfer(buyer, 1e18));
    }

    function test_blocklisted_receiver_reverts() public {
        vm.prank(admin);
        beacon.setBlocked(buyer, true);
        vm.prank(alice);
        vm.expectRevert(DemoStockToken.ReceiverBlocked.selector);
        token.transfer(buyer, 1e18);
    }

    // --- fail-closed beacon resolution ---

    // --- the upgrade beat (revocation trigger) ---

    function test_upgrade_swaps_impl_under_fixed_proxy_and_state_survives() public {
        assertEq(beacon.implementation(), address(implV1));
        vm.prank(admin);
        beacon.pause();
        vm.prank(admin);
        beacon.upgradeTo(address(implV2));
        // proxy address fixed, impl swapped
        assertEq(beacon.implementation(), address(implV2));
        assertTrue(token.paused(), "beacon-held state must survive the upgrade");
        assertEq(token.balanceOf(alice), 100e18, "proxy-held state must survive the upgrade");
        // still fully functional on the new impl
        vm.prank(admin);
        beacon.unpause();
        vm.prank(alice);
        assertTrue(token.transfer(buyer, 2e18));
    }

    // --- admin gating everywhere ---

    function test_non_admin_cannot_upgrade_pause_blocklist_mint() public {
        address eve = makeAddr("eve");

        // expectation FIRST: vm.expectRevert is itself a cheatcode call and
        // would consume a pending vm.prank
        vm.expectRevert(
            abi.encodeWithSelector(IAccessControl.AccessControlUnauthorizedAccount.selector, eve, beacon.DEFAULT_ADMIN_ROLE())
        );
        vm.prank(eve);
        beacon.upgradeTo(address(implV2));

        vm.expectRevert(
            abi.encodeWithSelector(IAccessControl.AccessControlUnauthorizedAccount.selector, eve, beacon.PAUSER_ROLE())
        );
        vm.prank(eve);
        beacon.pause();

        vm.expectRevert(
            abi.encodeWithSelector(IAccessControl.AccessControlUnauthorizedAccount.selector, eve, beacon.BLOCKLIST_ROLE())
        );
        vm.prank(eve);
        beacon.setBlocked(eve, true);

        vm.expectRevert(
            abi.encodeWithSelector(IAccessControl.AccessControlUnauthorizedAccount.selector, eve, token.MINTER_ROLE())
        );
        vm.prank(eve);
        token.mint(eve, 1e18);

        // and the beacon never moved
        assertEq(beacon.implementation(), address(implV1));
        assertFalse(beacon.paused());
        assertFalse(beacon.isBlocked(eve));
    }

    // --- probe-target acceptance (plan Revised note 2, step-2 semantics) ---

    function test_calibrated_probe_targets() public view {
        // paused() 0x5c975abb → the token PROXY answers
        (bool okPaused, bytes memory retPaused) = address(proxy).staticcall(abi.encodeWithSelector(SEL_PAUSED));
        assertTrue(okPaused, "proxy must answer paused()");
        assertFalse(abi.decode(retPaused, (bool)));

        // isBlocked(buyer) 0xfbac3951 → the RESOLVED BEACON answers
        (bool okBlocked, bytes memory retBlocked) = address(beacon).staticcall(abi.encodeWithSelector(SEL_BLOCKLIST, buyer));
        assertTrue(okBlocked, "beacon must answer isBlocked()");
        assertFalse(abi.decode(retBlocked, (bool)));

        // …and the same selector reverts through the proxy/impl (live-verified
        // on the genuine tokens — the hidden-blocklist signature)
        (bool okOnProxy,) = address(proxy).staticcall(abi.encodeWithSelector(SEL_BLOCKLIST, buyer));
        assertFalse(okOnProxy, "isBlocked must revert on the proxy");
    }

    // --- fail-closed beacon resolution ---

    function test_impl_without_beacon_is_fail_closed() public {
        // a bare impl (no proxy slot around it) refuses to report pause state
        // or move balances — the beacon is the single source of truth
        DemoStockToken bare = new DemoStockToken();
        vm.expectRevert(DemoStockToken.BeaconUnresolved.selector);
        bare.paused();
        (bool ok, ) = address(bare).staticcall(abi.encodeWithSelector(SEL_PAUSED));
        assertFalse(ok, "bare impl must not answer paused() as unpaused");
    }
}
