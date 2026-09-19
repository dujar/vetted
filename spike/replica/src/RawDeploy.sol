// SPDX-License-Identifier: MIT
// Deploys raw creation code — used to put the EXTSLOAD canary runtime on-chain
// (vm.etch is a test cheatcode, unusable against a live RPC).
pragma solidity ^0.8.28;

contract RawDeploy {
    event Deployed(address addr);
    address public last;

    function deploy(bytes memory initcode) external returns (address a) {
        assembly {
            a := create(0, add(initcode, 32), mload(initcode))
        }
        require(a != address(0), "create failed");
        last = a;
        emit Deployed(a);
    }
}
