// SPDX-License-Identifier: MIT
// EXTSLOAD (0x5c) verdict canary — deploys the 42-byte ExtSloadProbe runtime
// and CALLS it; only the call decides (deployment always succeeds).
//   call succeeds + 32 bytes → the chain ships EIP-2330: criterion (c) works
//                              from-contract; deploy Extsload-style helper.
//   call reverts             → opcode absent (expected on stock EVM forks):
//                              criterion (c) is node-side only for EVERY
//                              language, Stylus included — not a gate FAIL.
pragma solidity ^0.8.28;

import {Script, console2} from "forge-std/Script.sol";
import {ExtSloadProbe} from "../src/ExtSloadProbe.sol";
import {RawDeploy} from "../src/RawDeploy.sol";

contract DeployCanary is Script {
    function run() external {
        uint256 pk = vm.envUint("DEPLOY_KEY");
        vm.startBroadcast(pk);

        RawDeploy factory = new RawDeploy();
        bytes memory canaryInit = abi.encodePacked(
            hex"602a80600c39602a6000f3",
            ExtSloadProbe.runtime()
        );
        address canary = factory.deploy{gas: 3_000_000}(canaryInit);

        (bool ok, bytes memory ret) = canary.call("");
        console2.log("EXTSLOAD_CANARY", address(canary));
        if (ok) {
            console2.log("VERDICT: opcode 0x5c PRESENT, ret bytes:", ret.length);
        } else {
            console2.log("VERDICT: opcode 0x5c ABSENT (node-side reads only)");
        }

        vm.stopBroadcast();
    }
}
