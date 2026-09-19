// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import {Script, console2} from "forge-std/Script.sol";
import {CastNames} from "../src/CastNames.sol";
import {DemoStockToken} from "../src/DemoStockToken.sol";
import {DemoBeacon} from "../src/DemoBeacon.sol";
import {DemoBeaconProxy} from "../src/DemoBeaconProxy.sol";
import {ImpostorPlain} from "../src/ImpostorPlain.sol";
import {ImpostorSelfProxy, ImpostorSelfProxyImpl} from "../src/ImpostorSelfProxy.sol";

/// @notice Deploys the full demo cast on one chain (runbook: demo/assets.md):
/// the genuine-pattern replica — two impls (v1 + UPGRADE TARGET, verify loose
/// end 2: the registry records the impl as an ADDRESS, so the demo's
/// upgrade beat drifts the record without a code change), beacon, proxy —
/// plus the two impostor twins sharing name/symbol.
///
/// Env: DEPLOY_KEY (pk, the throwaway spike key for rehearsals — never
/// committed), ADMIN (optional, defaults to the deployer).
/// The cast ships CLEAN: unpaused, nobody blocked — step 9's runbook drives
/// the beats (pause / blocklist / upgrade) live.
contract DeployReplicas is Script {
    function run() external {
        uint256 pk = vm.envUint("DEPLOY_KEY");
        address admin = vm.envOr("ADMIN", vm.addr(pk));
        vm.startBroadcast(pk);

        DemoStockToken implV1 = new DemoStockToken();
        DemoStockToken implV2 = new DemoStockToken(); // upgrade target
        DemoBeacon beacon = new DemoBeacon(address(implV1), admin);
        DemoBeaconProxy proxy = new DemoBeaconProxy(address(beacon));
        DemoStockToken(address(proxy)).initialize(CastNames.NAME, CastNames.SYMBOL, admin);
        DemoStockToken(address(proxy)).mint(admin, 1_000_000e18); // AP stand-in float

        ImpostorPlain twin1 = new ImpostorPlain(CastNames.NAME, CastNames.SYMBOL, bytes32(uint256(0xa0a1a2a3)));
        twin1.mint(admin, 1_000_000e18);
        ImpostorSelfProxyImpl twin2Impl = new ImpostorSelfProxyImpl();
        ImpostorSelfProxy twin2 = new ImpostorSelfProxy(address(twin2Impl));
        (bool okInit,) = address(twin2).call(
            abi.encodeWithSelector(ImpostorSelfProxyImpl.init.selector, CastNames.NAME, CastNames.SYMBOL)
        );
        require(okInit, "twin2 init");
        (bool okMint,) = address(twin2).call(abi.encodeWithSelector(ImpostorSelfProxyImpl.mint.selector, admin, 1_000_000e18));
        require(okMint, "twin2 mint");

        vm.stopBroadcast();

        console2.log("REPLICA_IMPL_V1", address(implV1));
        console2.log("REPLICA_IMPL_V2_UPGRADE_TARGET", address(implV2));
        console2.log("REPLICA_BEACON", address(beacon));
        console2.log("REPLICA_PROXY (canonical demo token)", address(proxy));
        console2.log("TWIN1_IMPOSTOR_PLAIN", address(twin1));
        console2.log("TWIN2_IMPOSTOR_SELFPROXY_PROXY", address(twin2));
        console2.log("TWIN2_IMPOSTOR_SELFPROXY_IMPL", address(twin2Impl));
        console2.log("ADMIN", admin);
    }
}
