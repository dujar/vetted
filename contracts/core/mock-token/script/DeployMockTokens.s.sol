// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import {Script, console2} from "forge-std/Script.sol";
import {RawHull} from "../src/RawHull.sol";
import {ForwarderPatch} from "../src/ForwarderPatch.sol";
import {MockBeacon} from "../src/MockBeacon.sol";
import {MockTokenImpl} from "../src/MockTokenImpl.sol";

interface IRegistry {
    function verify(address token, uint256 riskFlags, address implAddr) external;
}

/// Deploys the mock pattern-token set (plan task 4/5) for the guard
/// integration: ONE shared beacon + ONE shared token impl (the genuine
/// layout - calibration_4663.json: "one shared impl across tokens") behind
/// TWO genuine-shape forwarders (the 283-byte `p_proxy.hex` runtime with the
/// embedded beacon patched), an unwired v2 upgrade target for the
/// GUARD_IMPL_MISMATCH beat, and a plain no-pattern token for the
/// GUARD_NO_RECORD receipt and the degraded (extraction-fails → skip) settle.
///
/// Env:
///   PRIVATE_KEY       broadcast key - is admin (beacon/tokens), registrar
///                     (registry constructor arg) and the buyer.
///   MM_KEY            optional; the project MM counterparty key. Defaults to
///                     PRIVATE_KEY (self-fill receipts).
///   REGISTRY_ADDRESS  optional; token registration is skipped when unset
///                     (mock-only rehearsal on a bare anvil).
///   GUARD_ADDRESS     optional; the MM/buyer allowances are skipped when
///                     unset.
contract DeployMockTokens is Script {
    uint256 constant FUND = 1_000_000e18;

    function run() external {
        uint256 deployerKey = vm.envUint("PRIVATE_KEY");
        uint256 mmKey = vm.envOr("MM_KEY", deployerKey);
        address deployer = vm.addr(deployerKey);
        address mm = vm.addr(mmKey);
        address registry = vm.envOr("REGISTRY_ADDRESS", address(0));
        address guard = vm.envOr("GUARD_ADDRESS", address(0));

        vm.startBroadcast(deployerKey);

        MockTokenImpl implV1 = new MockTokenImpl("Mock Stock (Vetted pattern)", "MSA");
        MockTokenImpl implV2 = new MockTokenImpl("Mock Stock (Vetted pattern)", "MSA");
        MockBeacon beacon = new MockBeacon(address(implV1));

        // the pattern tokens: genuine runtime, only the embedded beacon patched
        address tokenA = address(new RawHull(ForwarderPatch.patchBeacon(address(beacon))));
        address tokenB = address(new RawHull(ForwarderPatch.patchBeacon(address(beacon))));
        MockTokenImpl plain = new MockTokenImpl("Plain Mock Stock (no pattern)", "MSP");

        // per-token identity through the forwarder (proxy-local storage) -
        // distinct names over one shared impl, like the genuine tokens
        MockTokenImpl(tokenA).init("Mock Stock A (Vetted pattern)", "MSA");
        MockTokenImpl(tokenB).init("Mock Stock B (Vetted pattern)", "MSB");

        // Token state lives per-proxy (delegatecall storage) - mint THROUGH
        // the forwarders. The buyer escrows tokenIn; the MM holds tokenOut
        // (and tokenA: the degraded settle's fill is tokenOut = A).
        MockTokenImpl(tokenA).mint(deployer, FUND);
        MockTokenImpl(tokenA).mint(mm, FUND);
        MockTokenImpl(tokenB).mint(mm, FUND);
        plain.mint(deployer, FUND);

        // Standing allowances: the buyer commits (guard pulls tokenIn from
        // buyer), the MM's fill is the guard pulling tokenOut from it.
        if (guard != address(0)) {
            MockTokenImpl(tokenA).approve(guard, type(uint256).max);
            plain.approve(guard, type(uint256).max);
            if (mmKey != deployerKey) {
                vm.stopBroadcast();
                vm.startBroadcast(mmKey);
            }
            MockTokenImpl(tokenA).approve(guard, type(uint256).max);
            MockTokenImpl(tokenB).approve(guard, type(uint256).max);
            if (mmKey != deployerKey) {
                vm.stopBroadcast();
                vm.startBroadcast(deployerKey);
            }
        } else {
            console2.log("NOTE: GUARD_ADDRESS unset - allowances skipped (mock-only rehearsal)");
        }

        // The guard's five red flags ride the registry records: pattern
        // tokens registered with the beacon's CURRENT implementation. The
        // plain token stays unregistered - the receipts script registers it
        // mid-run (that registration is the GUARD_NO_RECORD → degraded-settle
        // beat).
        if (registry != address(0)) {
            IRegistry(registry).verify(tokenA, 0, address(implV1));
            IRegistry(registry).verify(tokenB, 0, address(implV1));
        } else {
            console2.log("NOTE: REGISTRY_ADDRESS unset - registration skipped (mock-only rehearsal)");
        }

        console2.log("MOCK_BEACON", address(beacon));
        console2.log("MOCK_IMPL_V1", address(implV1));
        console2.log("MOCK_IMPL_V2", address(implV2));
        console2.log("MOCK_TOKEN_A", tokenA);
        console2.log("MOCK_TOKEN_B", tokenB);
        console2.log("MOCK_TOKEN_PLAIN", address(plain));

        vm.stopBroadcast();
    }
}
