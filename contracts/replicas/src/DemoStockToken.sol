// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import {ERC20Upgradeable} from "@openzeppelin/contracts-upgradeable/token/ERC20/ERC20Upgradeable.sol";
import {AccessControlUpgradeable} from "@openzeppelin/contracts-upgradeable/access/AccessControlUpgradeable.sol";
import {DemoBeacon} from "./DemoBeacon.sol";

/// @notice The genuine-pattern stock-token IMPLEMENTATION (demo cast: v1 and
/// the upgrade target are two instances of this same code — the registry
/// records the impl as an ADDRESS, so the upgrade beat drifts the record
/// without any logic change; step-6 verify loose end 2).
///
/// GENUINE LAYOUT (plan Revised note 1 — not the spike's anti-pattern):
/// pause and blocklist state live on the BEACON. This impl only reads them,
/// by resolving its own proxy's EIP-1967 beacon slot and staticalling the
/// beacon — the token surface stays silent about the blocklist (isBlocked
/// reverts through the proxy, live-verified on the genuine tokens).
contract DemoStockToken is Initializable, ERC20Upgradeable, AccessControlUpgradeable {
    bytes32 public constant MINTER_ROLE = keccak256("MINTER_ROLE");

    // keccak256("eip1967.proxy.beacon") - 1 — read-only here; written by the
    // proxy constructor.
    bytes32 private constant BEACON_SLOT = 0xa3f0ad74e5423aebfd80d3ef4346578335a9a72aeaee59ff6cb3582b35133d50;

    error BeaconUnresolved();
    error TokenPaused();
    error SenderBlocked();
    error ReceiverBlocked();

    constructor() {
        _disableInitializers();
    }

    /// @param name_ cast name (CastNames.NAME)
    /// @param symbol_ cast ticker (CastNames.SYMBOL)
    /// @param admin_ the demo operator (also initial minter — the AP stand-in;
    /// genuine mints are restricted to Authorised Participants).
    function initialize(string calldata name_, string calldata symbol_, address admin_) public initializer {
        __ERC20_init(name_, symbol_);
        __AccessControl_init();
        _grantRole(DEFAULT_ADMIN_ROLE, admin_);
        _grantRole(MINTER_ROLE, admin_);
    }

    function mint(address to, uint256 amount) external onlyRole(MINTER_ROLE) {
        _mint(to, amount);
    }

    /// @dev Answers on the PROXY (calibrated probe 0x5c975abb, F4) by
    /// forwarding to the beacon, where the state actually lives.
    function paused() public view returns (bool) {
        return _beacon().paused();
    }

    /// @dev Fail-closed: an impl with no resolvable beacon transfers nothing.
    /// (In the deployed cast the slot is always set by the proxy constructor.)
    function _beacon() internal view returns (DemoBeacon) {
        address b;
        assembly {
            b := sload(BEACON_SLOT)
        }
        if (b == address(0)) revert BeaconUnresolved();
        return DemoBeacon(b);
    }

    /// @dev The hidden-modifier equivalent: the transfer hook enforces the
    /// beacon-held global pause and per-address blocklist on every move.
    function _update(address from, address to, uint256 value) internal override {
        DemoBeacon beacon = _beacon();
        if (beacon.paused()) revert TokenPaused();
        if (from != address(0) && beacon.isBlocked(from)) revert SenderBlocked();
        if (to != address(0) && beacon.isBlocked(to)) revert ReceiverBlocked();
        super._update(from, to, value);
    }
}
