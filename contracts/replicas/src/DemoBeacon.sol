// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import {AccessControlUpgradeable} from "@openzeppelin/contracts-upgradeable/access/AccessControlUpgradeable.sol";

/// @notice Faithful replica of the GENUINE Robinhood stock-token beacon
/// (spike/evidence/calibration_4663.json "beacon"): an AccessControl-holding
/// upgrade beacon that ALSO carries the global pause and the per-address
/// blocklist. THIS is where the hidden demo state lives — the token
/// impl/forwarder revert on isBlocked(address) (0xfbac3951); the beacon
/// answers (plan Revised note 1; step-2 findings "beacon-not-proxy").
/// The demo's revocation trigger is upgradeTo(): swapping implementation()
/// under a FIXED proxy address is what the registry keys drift on.
contract DemoBeacon is AccessControlUpgradeable {
    bytes32 public constant PAUSER_ROLE = keccak256("PAUSER_ROLE");
    bytes32 public constant BLOCKLIST_ROLE = keccak256("BLOCKLIST_ROLE");

    /// @dev 0x5c60da1b — selector pinned by the step-2 calibration.
    address public implementation;

    /// @dev 0x5c975abb — answers here, and on the token proxy via the impl's
    /// forwarding view (calibrated probe target: proxy). State lives HERE.
    bool public paused;

    /// @dev 0xfbac3951 — answers ONLY here (calibrated probe target: the
    /// resolved beacon). The hidden per-address blocklist.
    mapping(address account => bool) public isBlocked;

    event Upgraded(address indexed implementation);
    event BlocklistSet(address indexed account, bool blocked);

    error ZeroImplementation();

    constructor(address initialImplementation, address admin) {
        if (initialImplementation == address(0)) revert ZeroImplementation();
        __AccessControl_init();
        _grantRole(DEFAULT_ADMIN_ROLE, admin);
        _grantRole(PAUSER_ROLE, admin);
        _grantRole(BLOCKLIST_ROLE, admin);
        implementation = initialImplementation;
        emit Upgraded(initialImplementation);
    }

    /// @notice the revocation trigger beat: a new implementation under the
    /// same proxy address, with the beacon-held pause/blocklist state intact.
    function upgradeTo(address newImplementation) external onlyRole(DEFAULT_ADMIN_ROLE) {
        if (newImplementation == address(0)) revert ZeroImplementation();
        implementation = newImplementation;
        emit Upgraded(newImplementation);
    }

    function pause() external onlyRole(PAUSER_ROLE) {
        paused = true;
    }

    function unpause() external onlyRole(PAUSER_ROLE) {
        paused = false;
    }

    /// @notice the non-obvious setter of the hidden-modifier pattern — the
    /// blocklist is not discoverable from the token surface at all.
    function setBlocked(address account, bool blocked) external onlyRole(BLOCKLIST_ROLE) {
        isBlocked[account] = blocked;
        emit BlocklistSet(account, blocked);
    }
}
