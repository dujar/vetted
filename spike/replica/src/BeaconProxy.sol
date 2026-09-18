// SPDX-License-Identifier: MIT
// Minimal EIP-1967 beacon proxy — reads the beacon through the slot like the
// genuine tokens (their getters revert; this one exposes beacon() so the
// Stylus probe's getter path has a decodable target too).
pragma solidity ^0.8.28;

contract BeaconProxy {
    // keccak256("eip1967.proxy.beacon") - 1
    bytes32 internal constant BEACON_SLOT = 0xa3f0ad74e5423aebfd80d3ef4346578335a9a72aeaee59ff6cb3582b35133d50;

    constructor(address beacon, bytes memory data) {
        assembly {
            sstore(BEACON_SLOT, beacon)
        }
        if (data.length > 0) {
            // resolve via beacon → implementation, then delegatecall the impl
            // (OZ BeaconProxy pattern; delegatecalling the beacon itself would
            // hit UpgradeableBeacon, which has no initialize())
            (bool okImpl, bytes memory ret) = beacon.staticcall(abi.encodeWithSignature("implementation()"));
            require(okImpl && ret.length >= 32, "beacon read failed");
            address impl = abi.decode(ret, (address));
            (bool ok, ) = impl.delegatecall(data);
            require(ok, "init failed");
        }
    }

    function beacon() external view returns (address) {
        address b;
        assembly {
            b := sload(BEACON_SLOT)
        }
        return b;
    }

    function _implementation() internal view returns (address) {
        address b;
        assembly {
            b := sload(BEACON_SLOT)
        }
        (bool ok, bytes memory ret) = b.staticcall(abi.encodeWithSignature("implementation()"));
        require(ok && ret.length >= 32, "beacon read failed");
        return abi.decode(ret, (address));
    }

    fallback() external payable {
        address impl = _implementation();
        assembly {
            calldatacopy(0, 0, calldatasize())
            let ok := delegatecall(gas(), impl, 0, calldatasize(), 0, 0)
            returndatacopy(0, 0, returndatasize())
            switch ok
            case 0 { revert(0, returndatasize()) }
            default { return(0, returndatasize()) }
        }
    }

    receive() external payable {}
}
