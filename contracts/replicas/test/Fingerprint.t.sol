// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import {Cast} from "./Cast.t.sol";
import {CastNames} from "../src/CastNames.sol";
import {DemoBeacon} from "../src/DemoBeacon.sol";
import {ImpostorPlain} from "../src/ImpostorPlain.sol";
import {ImpostorSelfProxy, ImpostorSelfProxyImpl} from "../src/ImpostorSelfProxy.sol";

/// @notice The composed pattern fingerprint (FINGERPRINT.md — step-4 joint
/// check list): the replica must PASS every check; each twin must FAIL on its
/// named checks while keeping the name/symbol mimicry. Mirrors the step-4
/// matcher — divergence is a blocking defect for whichever side drifts.
contract FingerprintTest is Cast {
    // ERC-20 / pattern selector surface required by F5
    bytes4[] internal tokenSelectors;
    bytes4[] internal beaconSelectors;

    constructor() {
        // zero-arg or arg-carrying forms exercised via _selectorKnown
        // token (via proxy)
        tokenSelectors.push(0x06fdde03); // name()
        tokenSelectors.push(0x95d89b41); // symbol()
        tokenSelectors.push(0x313ce567); // decimals()
        tokenSelectors.push(0x18160ddd); // totalSupply()
        tokenSelectors.push(0x70a08231); // balanceOf(address)
        tokenSelectors.push(0xa9059cbb); // transfer(address,uint256)
        tokenSelectors.push(0x23b872dd); // transferFrom(address,address,uint256)
        tokenSelectors.push(0x095ea7b3); // approve(address,uint256)
        tokenSelectors.push(0xdd62ed3e); // allowance(address,address)
        tokenSelectors.push(SEL_PAUSED); // paused() — on the PROXY
        // beacon
        beaconSelectors.push(SEL_IMPLEMENTATION); // implementation()
        beaconSelectors.push(0x3659cfe6); // upgradeTo(address)
        beaconSelectors.push(SEL_PAUSED); // paused()
        beaconSelectors.push(0x8456cb59); // pause()
        beaconSelectors.push(0x3f4ba83a); // unpause()
        beaconSelectors.push(SEL_BLOCKLIST); // isBlocked(address)
        beaconSelectors.push(0x91d14854); // hasRole(bytes32,address)
        beaconSelectors.push(0x2f2ff15d); // grantRole(bytes32,address)
        beaconSelectors.push(0xa217fddf); // DEFAULT_ADMIN_ROLE()
        beaconSelectors.push(0x01ffc9a7); // supportsInterface(bytes4)
    }

    function _checkSelectors(address target, bytes4[] storage sels) internal {
        for (uint256 i = 0; i < sels.length; i++) {
            bytes memory args;
            if (sels[i] == 0x70a08231 || sels[i] == 0xfbac3951 || sels[i] == 0x3659cfe6) {
                args = abi.encode(address(1));
            } else if (sels[i] == 0xa9059cbb || sels[i] == 0x095ea7b3) {
                args = abi.encode(address(1), uint256(0));
            } else if (sels[i] == 0x23b872dd || sels[i] == 0xdd62ed3e) {
                args = abi.encode(address(1), address(2), uint256(0));
            } else if (sels[i] == 0x91d14854) {
                args = abi.encode(bytes32(0), address(1));
            } else if (sels[i] == 0x2f2ff15d) {
                args = abi.encode(bytes32(0), address(1));
            } else if (sels[i] == 0x01ffc9a7) {
                args = abi.encode(bytes4(0x01ffc9a7));
            }
            assertTrue(_selectorKnown(target, sels[i], args), "missing selector");
        }
    }

    // ------------------------------------------------------------------
    // Replica: passes EVERY check
    // ------------------------------------------------------------------

    function test_replica_passes_full_fingerprint() public {
        _deployCast();

        // F1 — solc 0.8.x metadata
        (uint8 major, uint8 minor) = _solcVersion(address(proxy).code);
        assertEq(major, 0, "F1: solc major");
        assertEq(minor, 8, "F1: solc minor");

        // F2 — beacon slot SET, impl slot EMPTY
        assertTrue(_slot(address(proxy), BEACON_SLOT) != bytes32(0), "F2: beacon slot empty");
        assertEq(_slot(address(proxy), IMPL_SLOT), bytes32(0), "F2: impl slot set");

        // F3 — beacon resolvable from forwarder bytecode shape
        (bool found, address resolved) = _findEmbeddedBeacon(address(proxy));
        assertTrue(found, "F3: no implementation()-answering embedded address");
        assertEq(resolved, address(beacon), "F3: shape resolution != deployed beacon");
        (bool ok, bytes memory ret) = resolved.staticcall(abi.encodeWithSelector(SEL_IMPLEMENTATION));
        assertTrue(ok, "F3: implementation() failed");
        assertEq(abi.decode(ret, (address)), address(implV1), "F3: resolved impl mismatch");

        // F4 — calibrated probe targets: paused() on the PROXY,
        // isBlocked(buyer) on the RESOLVED BEACON, and the negative:
        // isBlocked reverts through the proxy (hidden-modifier semantics)
        assertTrue(_answeredFalse(address(proxy), abi.encodeWithSelector(SEL_PAUSED)), "F4: paused() on proxy");
        assertTrue(
            _answeredFalse(address(beacon), abi.encodeWithSelector(SEL_BLOCKLIST, buyer)), "F4: isBlocked on beacon"
        );
        (bool blockedOnProxy,) = _call(address(proxy), abi.encodeWithSelector(SEL_BLOCKLIST, buyer));
        assertFalse(blockedOnProxy, "F4: isBlocked must revert on the proxy");

        // F5 — required selector surface
        _checkSelectors(address(proxy), tokenSelectors);
        _checkSelectors(address(beacon), beaconSelectors);

        // mimicry ground truth: name/symbol match the cast (shared with twins)
        assertEq(token.name(), CastNames.NAME);
        assertEq(token.symbol(), CastNames.SYMBOL);
        assertEq(token.decimals(), 18);
    }

    /// @dev guard rehearsal (step-3 decision 1): the guard's forwarder-bytecode
    /// extraction + implementation() staticcall path, end to end.
    function test_guard_extraction_path() public {
        _deployCast();
        (bool found, address resolved) = _findEmbeddedBeacon(address(proxy));
        assertTrue(found, "F3: guard could not extract beacon");
        (bool ok, bytes memory ret) = resolved.staticcall(abi.encodeWithSelector(SEL_IMPLEMENTATION));
        assertTrue(ok);
        assertEq(abi.decode(ret, (address)), address(implV1));
    }

    // ------------------------------------------------------------------
    // Twin 1 — ImpostorPlain: passes mimicry, fails F2/F3/F4
    // ------------------------------------------------------------------

    function test_impostor_plain_fails_fingerprint() public {
        ImpostorPlain twin = new ImpostorPlain(CastNames.NAME, CastNames.SYMBOL, bytes32(uint256(0xdeadbeef)));

        // mimicry holds (the trap)
        assertEq(twin.name(), CastNames.NAME);
        assertEq(twin.symbol(), CastNames.SYMBOL);
        assertEq(twin.decimals(), 18);

        // F2 — no EIP-1967 beacon layout at all
        assertEq(_slot(address(twin), BEACON_SLOT), bytes32(0), "F2: beacon slot unexpectedly set");
        assertEq(_slot(address(twin), IMPL_SLOT), bytes32(0));

        // F3 — no embedded implementation()-answering forwarder
        (bool found,) = _findEmbeddedBeacon(address(twin));
        assertFalse(found, "F3: twin must not embed an implementation()-answering beacon");

        // F4 — blocklist probe has NO resolved beacon to answer; the decoy
        // isBlocked lives on the token itself (wrong target)
        (bool okDecoy,) = _call(address(twin), abi.encodeWithSelector(SEL_BLOCKLIST, buyer));
        assertTrue(okDecoy, "decoy isBlocked should answer on the token (the trap)");
    }

    // ------------------------------------------------------------------
    // Twin 2 — ImpostorSelfProxy: passes mimicry + proxy-paused probe,
    // fails F2/F3/F4 on slot layout
    // ------------------------------------------------------------------

    function test_impostor_selfproxy_fails_fingerprint() public {
        ImpostorSelfProxyImpl twinImpl = new ImpostorSelfProxyImpl();
        ImpostorSelfProxy twinProxy = new ImpostorSelfProxy(address(twinImpl));
        (bool okInit,) = address(twinProxy).call(abi.encodeWithSelector(ImpostorSelfProxyImpl.init.selector, CastNames.NAME, CastNames.SYMBOL));
        assertTrue(okInit, "twin init through proxy");

        // mimicry: paused() answers on its proxy just like the genuine shape
        (bool okPaused, bytes memory retPaused) = _call(address(twinProxy), abi.encodeWithSelector(SEL_PAUSED));
        assertTrue(okPaused, "twin paused() should answer on its proxy (the trap)");
        assertTrue(retPaused.length == 32);

        // F2 — the WRONG slot: impl slot SET, beacon slot EMPTY
        assertTrue(_slot(address(twinProxy), IMPL_SLOT) != bytes32(0), "F2: impl slot unexpectedly empty");
        assertEq(_slot(address(twinProxy), BEACON_SLOT), bytes32(0), "F2: beacon slot unexpectedly set");

        // F3 — nothing in its bytecode answers implementation()
        (bool found,) = _findEmbeddedBeacon(address(twinProxy));
        assertFalse(found, "F3: twin must not embed an implementation()-answering beacon");

        // F4 — no resolved beacon ⇒ blocklist probe unanswerable
        assertFalse(_answeredFalse(address(twinProxy), abi.encodeWithSelector(SEL_BLOCKLIST, buyer)));
    }

    // ------------------------------------------------------------------
    // Excluded-on-purpose checks (documented negatives): the twins' fake
    // uid() and codehash differences must not be what rejects them — and
    // the replica must not be rejected by the impossible checks either.
    // ------------------------------------------------------------------

    function test_excluded_checks_are_not_load_bearing() public {
        _deployCast();
        ImpostorPlain twin = new ImpostorPlain(CastNames.NAME, CastNames.SYMBOL, bytes32(uint256(0xc0ffee)));

        // codehash byte-equality is impossible for ANY replica (embedded
        // per-deploy addresses) — FINGERPRINT.md excludes it explicitly
        assertFalse(
            keccak256(address(proxy).code) == 0x6c1fdd40002dcb440c7fff6a84171404d279ccb057803b65826f7546acd65630,
            "genuine codehash equality is not a composed check"
        );

        // uid() value is NOT a composed check: the replica has none, the twin
        // fabricates one — neither fact moves the verdict
        (bool okUid,) = _call(address(twin), abi.encodeWithSignature("uid()"));
        assertTrue(okUid, "twin fake uid() answers (and is ignored)");
        (bool okUidReplica,) = _call(address(proxy), abi.encodeWithSignature("uid()"));
        assertFalse(okUidReplica, "replica deliberately has no uid()");
    }
}
