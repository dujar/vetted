// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import {Script, console2} from "forge-std/Script.sol";
import {MockTokenImpl} from "../src/MockTokenImpl.sol";

interface IRegistry {
    function verify(address token, uint256 riskFlags, address implAddr) external;
    function revoke(address token, string calldata reason) external;
}

interface IGuard {
    function commit(address tokenIn, address tokenOut, uint256 amountIn, uint256 minOut) external;
    function execute() external;
}

interface IMockBeacon {
    function setBlocked(address account, bool blocked) external;
    function upgradeTo(address newImplementation) external;
}

/// The five deterministic red-flag receipts + the clean settle, ON-CHAIN
/// (plan task 5). One buyer key (admin/registrar); MM_KEY optional - with a
/// distinct MM the settle accounting is two-party, without it every fill is a
/// self-fill (logged).
///
/// Every scenario is commit → execute (expected revert, byte-exact string) →
/// restore state → execute again (success) so the escrow always unwinds and
/// the next scenario starts clean. Eleven guard `execute()` txs total: 5
/// reverting + 6 settling - the gas gate (≤ 200,000 each, spec.md:51) is
/// asserted by the deploy script from the broadcast receipts.
///
/// Env (all required except MM_KEY): PRIVATE_KEY, REGISTRY_ADDRESS,
/// GUARD_ADDRESS, BEACON_ADDRESS, TOKEN_A, TOKEN_B, TOKEN_PLAIN, IMPL_V1,
/// IMPL_V2 - as logged by DeployMockTokens.
contract RunReceipts is Script {
    uint256 constant AMOUNT_IN = 1_000e18;
    uint256 constant MIN_OUT = 500e18;

    function run() external {
        uint256 buyerKey = vm.envUint("PRIVATE_KEY");
        uint256 mmKey = vm.envOr("MM_KEY", buyerKey);
        bool oneKey = mmKey == buyerKey;
        address buyer = vm.addr(buyerKey);
        address registry = vm.envAddress("REGISTRY_ADDRESS");
        address guard = vm.envAddress("GUARD_ADDRESS");
        address beacon = vm.envAddress("BEACON_ADDRESS");
        address tokenA = vm.envAddress("TOKEN_A");
        address tokenB = vm.envAddress("TOKEN_B");
        address plain = vm.envAddress("TOKEN_PLAIN");
        address implV1 = vm.envAddress("IMPL_V1");
        address implV2 = vm.envAddress("IMPL_V2");
        if (oneKey) {
            console2.log("NOTE: MM_KEY unset - counterparty == buyer (self-fill receipts)");
        }

        vm.startBroadcast(buyerKey);

        // 1 - GUARD_NO_RECORD: the plain token has no registry record; the
        // guard's record check (fixed order, tokenIn first) fails closed.
        IGuard(guard).commit(plain, tokenA, AMOUNT_IN, MIN_OUT);
        vm.expectRevert(_revert("GUARD_NO_RECORD"));
        IGuard(guard).execute();
        console2.log("RECEIPT 1 GUARD_NO_RECORD (reverted; escrow restored by atomicity)");
        // Restore = register the plain token with ITSELF as impl. Its runtime
        // carries no forwarder shape, so the guard's extraction fails and the
        // beacon-bound probes are SKIPPED (never revert) - this cleanup
        // execute is the degraded-settle receipt.
        IRegistry(registry).verify(plain, 0, plain);
        IGuard(guard).execute();
        console2.log("RECEIPT 2 DEGRADED_SETTLE (ok - patternless token skips probes)");

        // 2 - GUARD_RECORD_REVOKED: the backend's revocation trigger (the
        // demo's upgrade → revocation → refusal beat).
        IRegistry(registry).revoke(tokenA, "beacon impl changed - record stale");
        IGuard(guard).commit(tokenA, tokenB, AMOUNT_IN, MIN_OUT);
        vm.expectRevert(_revert("GUARD_RECORD_REVOKED"));
        IGuard(guard).execute();
        console2.log("RECEIPT 3 GUARD_RECORD_REVOKED (reverted)");
        IRegistry(registry).verify(tokenA, 0, implV1);
        IGuard(guard).execute();
        console2.log("RECEIPT 4 REVOKED_CLEANUP (ok)");

        // 3 - GUARD_PAUSED: pause lives with the TOKEN - `pause()` through
        // the proxy writes the proxy's storage, the guard probes paused()
        // on the proxy.
        MockTokenImpl(tokenA).pause();
        IGuard(guard).commit(tokenA, tokenB, AMOUNT_IN, MIN_OUT);
        vm.expectRevert(_revert("GUARD_PAUSED"));
        IGuard(guard).execute();
        console2.log("RECEIPT 5 GUARD_PAUSED (reverted)");
        MockTokenImpl(tokenA).unpause();
        IGuard(guard).execute();
        console2.log("RECEIPT 6 PAUSED_CLEANUP (ok)");

        // 4 - GUARD_BLOCKLISTED: blocklist state lives on the BEACON; the
        // guard probes isBlocked(buyer) there.
        IMockBeacon(beacon).setBlocked(buyer, true);
        IGuard(guard).commit(tokenA, tokenB, AMOUNT_IN, MIN_OUT);
        vm.expectRevert(_revert("GUARD_BLOCKLISTED"));
        IGuard(guard).execute();
        console2.log("RECEIPT 7 GUARD_BLOCKLISTED (reverted)");
        IMockBeacon(beacon).setBlocked(buyer, false);
        IGuard(guard).execute();
        console2.log("RECEIPT 8 BLOCKLISTED_CLEANUP (ok)");

        // 5 - GUARD_IMPL_MISMATCH: upgrade the beacon - every forwarder now
        // resolves v2 while the records still name v1.
        IMockBeacon(beacon).upgradeTo(implV2);
        IGuard(guard).commit(tokenA, tokenB, AMOUNT_IN, MIN_OUT);
        vm.expectRevert(_revert("GUARD_IMPL_MISMATCH"));
        IGuard(guard).execute();
        console2.log("RECEIPT 9 GUARD_IMPL_MISMATCH (reverted)");
        IMockBeacon(beacon).upgradeTo(implV1);
        IGuard(guard).execute();
        console2.log("RECEIPT 10 MISMATCH_CLEANUP (ok)");

        // 6 - the clean settle with balance assertions. Two-party MM: the
        // buyer pays AMOUNT_IN of A and receives MIN_OUT of B pulled from
        // the MM; self-fill (MM == buyer) round-trips both back.
        uint256 buyerABefore = MockTokenImpl(tokenA).balanceOf(buyer);
        uint256 buyerBBefore = MockTokenImpl(tokenB).balanceOf(buyer);
        IGuard(guard).commit(tokenA, tokenB, AMOUNT_IN, MIN_OUT);
        IGuard(guard).execute();
        if (oneKey) {
            require(MockTokenImpl(tokenA).balanceOf(buyer) == buyerABefore, "settle: tokenIn round-trip");
        } else {
            require(MockTokenImpl(tokenA).balanceOf(buyer) == buyerABefore - AMOUNT_IN, "settle: tokenIn");
            require(MockTokenImpl(tokenB).balanceOf(buyer) == buyerBBefore + MIN_OUT, "settle: tokenOut fill");
        }
        console2.log("RECEIPT 11 SETTLE (ok; balances asserted)");

        vm.stopBroadcast();
    }

    /// The guard's deterministic red-flag reverts arrive as classic
    /// `Error(string)` data - the byte-exact pinned string.
    function _revert(string memory reason) private pure returns (bytes memory) {
        return abi.encodeWithSignature("Error(string)", reason);
    }
}
