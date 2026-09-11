//! Rust round-trip: every golden fixture deserializes into the wire types and
//! re-serializes to the same `serde_json::Value` (structural identity). The TS
//! side (roundtrip.test.ts) enforces byte-identical canonical form — together
//! the two implementations cannot drift apart.

use std::fs;

use serde_json::Value;
use vetted_shared::types::{
    GuardRevert, GuardRevertReason, RecordStatus, ScanResponse, TerminalState, Verdict,
};

fn read_json(path: &std::path::Path) -> (String, Value) {
    let raw = fs::read_to_string(path).expect("fixture readable");
    let value: Value = serde_json::from_str(&raw).expect("fixture parses");
    (raw, value)
}

#[test]
fn verdict_fixtures_round_trip() {
    let mut dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dir.push("fixtures/verdict");

    let mut files: Vec<_> = fs::read_dir(&dir)
        .expect("verdict fixture dir")
        .map(|e| e.unwrap().path())
        .collect();
    files.sort();

    assert_eq!(files.len(), 4, "one fixture per verdict");
    for file in files {
        let (raw, value) = read_json(&file);
        let scan: ScanResponse = serde_json::from_str(&raw).expect("deserializes as ScanResponse");
        let re: Value = serde_json::to_value(&scan).expect("reserializes");
        assert_eq!(re, value, "{}: structural round-trip", file.display());

        let name = file.file_stem().unwrap().to_str().unwrap();
        let expected = match name {
            "VERIFIED" => Some(Verdict::Verified),
            "IMPOSTOR" => Some(Verdict::Impostor),
            "UNVERIFIED" => Some(Verdict::Unverified),
            "REVOKED" => Some(Verdict::Revoked),
            _ => panic!("unexpected fixture {name}"),
        };
        assert_eq!(scan.verdict, expected, "{}: verdict matches filename", name);
        assert!(
            scan.verdict.is_some() || scan.terminal_state.is_some(),
            "no verdict attempted must set a terminal state"
        );
        for row in &scan.power_report {
            assert!(
                row.evidence_url.starts_with("http://") || row.evidence_url.starts_with("https://"),
                "every power-report row carries an evidence link"
            );
        }
    }
}

#[test]
fn guard_fixtures_round_trip_and_cover_all_reasons() {
    let mut dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dir.push("fixtures/guard");

    let mut files: Vec<_> = fs::read_dir(&dir)
        .expect("guard fixture dir")
        .map(|e| e.unwrap().path())
        .collect();
    files.sort();

    assert_eq!(files.len(), 5, "one fixture per guard revert reason");
    let mut seen = std::collections::HashSet::new();
    for file in files {
        let (raw, value) = read_json(&file);
        let guard: GuardRevert = serde_json::from_str(&raw).expect("deserializes as GuardRevert");
        let re: Value = serde_json::to_value(&guard).expect("reserializes");
        assert_eq!(re, value, "{}: structural round-trip", file.display());
        assert!(!guard.description.is_empty());
        seen.insert(guard.reason);
    }
    assert_eq!(
        seen,
        [
            GuardRevertReason::NoRecord,
            GuardRevertReason::RecordRevoked,
            GuardRevertReason::Paused,
            GuardRevertReason::Blocklisted,
            GuardRevertReason::ImplMismatch,
        ]
        .into_iter()
        .collect(),
        "all five reasons covered verbatim"
    );
}

#[test]
fn revoked_fixture_carries_record_and_revocation_evidence() {
    let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("fixtures/verdict/REVOKED.json");
    let (raw, _) = read_json(&path);
    let scan: ScanResponse = serde_json::from_str(&raw).unwrap();
    assert_eq!(scan.verdict, Some(Verdict::Revoked));
    let record = scan.record.expect("REVOKED carries its record");
    assert_eq!(record.status, RecordStatus::Revoked);
    assert!(record.revoked_at > 0);
    let tx = scan.revocation_tx.expect("REVOKED links the revocation tx");
    assert_eq!(tx.len(), 66, "0x + 32 bytes");
    let _ = TerminalState::NotContract; // type existence guard
}
