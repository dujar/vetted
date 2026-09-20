// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

/// Deploys a contract whose runtime code is EXACTLY the given bytes
/// (EIP-5202-style "hull": the constructor returns the payload as the
/// account's code). The mock pattern tokens' forwarders are the genuine
/// 283-byte runtime with a patched beacon — `vm.etch` is a test-only
/// cheatcode, so live deploys go through here.
///
/// Provenance: same deploy trick as the spike's `RawDeploy` + `DeployCanary`
/// (spike/replica/src), reduced to a hull for a single deployment.
contract RawHull {
    constructor(bytes memory runtime) payable {
        assembly ("memory-safe") {
            return(add(runtime, 0x20), mload(runtime))
        }
    }
}
