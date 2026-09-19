// SPDX-License-Identifier: MIT
// Minimal OZ-pattern beacon — the upgrade beat's pivot point.
pragma solidity ^0.8.28;

contract UpgradeableBeacon {
    address public implementation; // 0x5c60da1b
    address public owner;

    event Upgraded(address indexed implementation);

    constructor(address impl) {
        implementation = impl;
        owner = msg.sender;
    }

    function upgradeTo(address newImpl) external {
        require(msg.sender == owner, "owner only");
        implementation = newImpl;
        emit Upgraded(newImpl);
    }
}
