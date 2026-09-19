// SPDX-License-Identifier: MIT
// Spike replica deploy (step 2 task 6): impl1 + beacon + proxy, pause +
// blocklist states, then THE BEAT: beacon.upgradeTo(impl2) — the live proof
// that a beacon upgrade changes the resolved implementation under a fixed
// proxy address (what step 6 replicates and what registry revocation keys on).
pragma solidity ^0.8.28;

import {Script, console2} from "forge-std/Script.sol";
import {StockToken} from "../src/StockToken.sol";
import {UpgradeableBeacon} from "../src/UpgradeableBeacon.sol";
import {BeaconProxy} from "../src/BeaconProxy.sol";
import {ExtSloadProbe} from "../src/ExtSloadProbe.sol";
import {RawDeploy} from "../src/RawDeploy.sol";

contract Deploy is Script {

    function run() external {
        uint256 pk = vm.envUint("DEPLOY_KEY");
        vm.startBroadcast(pk);

        StockToken impl1 = new StockToken();
        StockToken impl2 = new StockToken();
        UpgradeableBeacon beacon = new UpgradeableBeacon(address(impl1));
        BeaconProxy proxy = new BeaconProxy(
            address(beacon),
            abi.encodeCall(
                StockToken.initialize,
                ("Everpure Replica", "P0", "0xreplica-uid-1")
            )
        );

        StockToken token = StockToken(address(proxy));
        token.mint(vm.addr(pk), 1000e18);
        token.pause();
        token.blockAddress(0x000000000000000000000000000000000000B10C);

        // the beat: upgrade under a fixed proxy address
        beacon.upgradeTo(address(impl2));

        // EXTSLOAD canary on-chain: initcode wraps the 42-byte runtime
        // (602a80600c39602a6000f3 + 60207f...5c5260206000f3)
        RawDeploy factory = new RawDeploy();
        bytes memory canaryInit = abi.encodePacked(
            hex"602a80600c39602a6000f3",
            ExtSloadProbe.runtime()
        );
        address canary = factory.deploy{gas: 3_000_000}(canaryInit);

        vm.stopBroadcast();

        console2.log("REPLICA_PROXY", address(proxy));
        console2.log("REPLICA_BEACON", address(beacon));
        console2.log("REPLICA_IMPL1", address(impl1));
        console2.log("REPLICA_IMPL2", address(impl2));
    }
}
