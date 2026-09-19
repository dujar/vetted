// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

/// @notice Faithful replica of the GENUINE stock-token forwarder
/// (spike/evidence/p_proxy.hex, calibration "proxy" field): runtime bytecode
/// EMBEDS the beacon address as a zero-padded immediate, STATICCALLs its
/// implementation() getter (selector 0x5c60da1b), DELEGATECALLs the result.
/// No beacon getter, no runtime slot read — resolution from this contract is
/// by storage (EIP-1967 beacon slot) or by bytecode SHAPE, which is exactly
/// how the guard (step 3) and the step-4 fingerprint find it. Never codehash
/// equality: the embedded address differs per deploy (FINGERPRINT.md F3).
contract DemoBeaconProxy {
    // keccak256("eip1967.proxy.beacon") - 1 — set once here (on the genuine
    // tokens, by deployment tooling); the impl slot stays empty (F2).
    bytes32 internal constant BEACON_SLOT = 0xa3f0ad74e5423aebfd80d3ef4346578335a9a72aeaee59ff6cb3582b35133d50;

    // Immutable → embedded in runtime bytecode as an immediate (genuine shape).
    address private immutable beacon;

    constructor(address beacon_) {
        beacon = beacon_;
        assembly {
            sstore(BEACON_SLOT, beacon_)
        }
    }

    fallback() external payable {
        (bool ok, bytes memory ret) = beacon.staticcall(abi.encodeWithSignature("implementation()"));
        require(ok && ret.length >= 32, "DemoBeaconProxy: beacon");
        address impl = abi.decode(ret, (address));
        require(impl != address(0), "DemoBeaconProxy: impl");
        assembly {
            calldatacopy(0, 0, calldatasize())
            let success := delegatecall(gas(), impl, 0, calldatasize(), 0, 0)
            returndatacopy(0, 0, returndatasize())
            switch success
            case 0 { revert(0, returndatasize()) }
            default { return(0, returndatasize()) }
        }
    }

    receive() external payable {}
}
