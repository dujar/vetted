// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

/// The BEACON side of the genuine Robinhood stock-token pattern, mirroring
/// `spike/evidence/calibration_4663.json`: the upgradeable implementation
/// pointer (`implementation()` / `upgradeTo`) AND the per-address blocklist
/// (`isBlocked`) live HERE — "the blocklist STATE lives on the beacon", not
/// on the token. Pause does NOT live here: the guard probes `paused()`
/// (0x5c975abb) on the token PROXY and `isBlocked(address)` (0xfbac3951) on
/// this beacon — the same selector reverts on the proxy/impl (live-verified,
/// which is exactly what the guard's beacon-resolution path expects).
///
/// Minimal on purpose: the guard only ever STATICCALLs 0xfbac3951 and
/// 0x5c60da1b on it, and the receipts script drives `upgradeTo` (the demo's
/// upgrade → revocation beat) and `setBlocked` (the GUARD_BLOCKLISTED
/// receipt). Plain admin gating instead of AccessControl — roles are not part
/// of the probed surface.
contract MockBeacon {
    address public immutable admin;
    address internal _implementation;
    mapping(address account => bool) internal _blocked;

    /// Emitted on `upgradeTo` — the drift event the backend's watchdog
    /// (step 4) would key on in the real flow.
    event Upgraded(address indexed implementation);
    event BlocklistSet(address indexed account, bool blocked);

    error Unauthorized(address caller);

    constructor(address initialImplementation) {
        admin = msg.sender;
        _implementation = initialImplementation;
    }

    modifier onlyAdmin() {
        if (msg.sender != admin) revert Unauthorized(msg.sender);
        _;
    }

    /// `implementation()` — selector 0x5c60da1b. The guard resolves the
    /// token's implementation with a STATICCALL to this beacon.
    function implementation() external view returns (address) {
        return _implementation;
    }

    /// The upgrade beat: moving the pointer changes what every forwarder
    /// resolves — GUARD_IMPL_MISMATCH once a record still names the old impl.
    function upgradeTo(address newImplementation) external onlyAdmin {
        _implementation = newImplementation;
        emit Upgraded(newImplementation);
    }

    /// `isBlocked(address)` — selector 0xfbac3951. A fresh account reads
    /// false (calibration: "returned false for a fresh EOA").
    function isBlocked(address account) external view returns (bool) {
        return _blocked[account];
    }

    /// Blocklist write — the GUARD_BLOCKLISTED receipt flips the buyer here.
    function setBlocked(address account, bool blocked) external onlyAdmin {
        _blocked[account] = blocked;
        emit BlocklistSet(account, blocked);
    }
}
