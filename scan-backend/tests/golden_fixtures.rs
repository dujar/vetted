//! The golden fixtures ARE the contract (plan task 4): the rule engine, fed
//! the labeled inputs each fixture describes, reproduces the fixture file
//! byte-exactly — 2-space JSON + trailing newline, the wire's canonical form.
//! This is the Rust-side contract test for the verdict engine; the shared
//! package's round-trip tests pin the types themselves.

#![allow(non_snake_case)] // test names carry the fixture names verbatim

use scan_backend::rules::{
    evaluate, impl_diff_pair, CanonicalEvidence, Probes, ScanInputs, Structure, CHAIN_MAINNET,
};
use vetted_shared::types::{RecordStatus, RegistryRecord};

fn fixture(name: &str) -> String {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../packages/shared/fixtures/verdict")
        .join(format!("{name}.json"));
    std::fs::read_to_string(path).expect("shared verdict fixture readable")
}

fn canonical_json(v: &impl serde::Serialize) -> String {
    let mut s = serde_json::to_string_pretty(v).expect("serializes");
    s.push('\n');
    s
}

fn structure(beacon_proxy: bool, signature_match: bool, impl_addr: Option<&str>) -> Structure {
    Structure {
        beacon_proxy,
        beacon: None,
        impl_addr: impl_addr.map(str::to_string),
        signature_match,
    }
}

#[test]
fn engine_reproduces_VERIFIED_fixture() {
    let mut inputs = ScanInputs::new(CHAIN_MAINNET, "0x9f8c24b7e3d1a05f6c8b4e2d9a7f1035c6b8d2e4");
    // Probed clean: neither the pause nor the blocklist mechanism is present.
    inputs.probes = Some(Probes { paused: false, blocklist: false });
    inputs.structure = Some(structure(true, true, None));
    inputs.record = Some(RegistryRecord {
        status: RecordStatus::Verified,
        risk_flags: "0".into(),
        verified_at: 1788048000,
        impl_: "0x77be0000000000000000000000000000000041af".into(),
        registrar: "0x5e1500000000000000000000000000000000aa11".into(),
        revoked_at: 0,
        reason: "0x0000000000000000000000000000000000000000000000000000000000000000".into(),
    });
    assert_eq!(canonical_json(&evaluate(&inputs)), fixture("VERIFIED"));
}

#[test]
fn engine_reproduces_IMPOSTOR_fixture() {
    let mut inputs = ScanInputs::new(CHAIN_MAINNET, "0x51aa0000000000000000000000000000000090c7");
    // Signature match, no record, canonical metadata mimics NVDA at a
    // different address, and the implementation differs from the canonical's.
    inputs.structure = Some(structure(false, true, Some("0xc30d11111111111111111111111111111111e812")));
    let (live, canonical) = impl_diff_pair(
        "0xc30d11111111111111111111111111111111e812",
        "0x77be0000000000000000000000000000000041af",
    );
    inputs.canonical = Some(CanonicalEvidence {
        listed: false,
        mimics_name: Some("NVIDIA".into()),
        mimics_symbol: Some("NVDA".into()),
        uid_match: None,
        impl_differs: Some((live, canonical)),
    });
    assert_eq!(canonical_json(&evaluate(&inputs)), fixture("IMPOSTOR"));
}

#[test]
fn engine_reproduces_REVOKED_fixture() {
    let mut inputs = ScanInputs::new(CHAIN_MAINNET, "0x51aa0000000000000000000000000000000090c7");
    inputs.structure = Some(structure(true, true, Some("0xc30d11111111111111111111111111111111e812")));
    inputs.record = Some(RegistryRecord {
        status: RecordStatus::Revoked,
        risk_flags: "0".into(),
        verified_at: 1787356800,
        impl_: "0x77be0000000000000000000000000000000041af".into(),
        registrar: "0x5e1500000000000000000000000000000000aa11".into(),
        revoked_at: 1788652800,
        reason: "0x626561636f6e20696d706c206368616e67656400000000000000000000000000".into(),
    });
    inputs.revocation_tx =
        Some("0xab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12".into());
    assert_eq!(canonical_json(&evaluate(&inputs)), fixture("REVOKED"));
}

#[test]
fn engine_reproduces_UNVERIFIED_fixture() {
    let mut inputs = ScanInputs::new(CHAIN_MAINNET, "0x1d4f0000000000000000000000000000000088b3");
    // Issuer list unreachable (degraded): heuristics only, advisory-labeled.
    inputs.degraded = true;
    inputs.structure = Some(structure(true, false, None));
    assert_eq!(canonical_json(&evaluate(&inputs)), fixture("UNVERIFIED"));
}

#[test]
fn engine_reproduces_not_contract_terminal_state() {
    // journeys.md:18 — NOT_CONTRACT is a terminal state, no verdict attempted.
    let mut inputs = ScanInputs::new(CHAIN_MAINNET, "0xdead000000000000000000000000000000000001");
    inputs.has_code = false;
    let out = evaluate(&inputs);
    assert_eq!(out.verdict, None);
    assert_eq!(
        serde_json::to_value(&out).unwrap()["terminalState"],
        "NOT_CONTRACT"
    );
}
