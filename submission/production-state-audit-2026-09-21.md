# Production state audit — step-10 task 2, recorded 2026-09-21 (SGT; 21:36 UTC)

Unfunded regime at step-10 start: the operator key reads 0 wei on
4663/46630/421614, so every funded-run artifact is PENDING-FUNDS and the
audit's live checks cover what exists without chain state. Re-run with
`bash scripts/release/production-check.sh` after the go-list — every line
must then read PASS or a justified N/A
(`submission/post-funding-checklist.md` step 2). The WARN on the secrets
audit is a checkout limitation (no wrangler deps in this worktree), not a
worker finding: REGISTRAR_KEY/DRIFT_ADMIN_SECRET ship deliberately unset
until the registrar handoff (plan Revised 2026-09-20 note 1).

---

# raw script output — 2026-09-20 21:36:10 UTC
# key 0x151e9f57F31310aFeBBB60c222c14badCf938E4C · regime-aware (PENDING-FUNDS = the funded run hasn't executed)

PENDING-FUNDS | deployments/4663.json absent — go-list (submission/post-funding-checklist.md §1) not executed yet
PENDING-FUNDS | deployments/421614.json absent — first funded run anywhere writes it (end-to-end deploy.sh on 421614)
PENDING-FUNDS | chain 4663: operator key reads 0 wei
PENDING-FUNDS | chain 421614: operator key reads 0 wei
PASS          | Pages https://vetted-1un.pages.dev -> 200
PASS          | Worker /health ok
PASS          | watchdog degrade sentinel exact (runs:0 + provenanceUrl — the sentinel, never a measured zero)
PENDING-FUNDS | P scan: record null (worker REGISTRY_ADDRESS_4663 unset — go-list step 6) -> UNVERIFIED
PASS          | spike canonical-fetch mirror alive (fallback)
PASS          | 4663 explorer resolves (robinhoodchain.blockscout.com)
PASS          | 421614 mirror explorer resolves (sepolia.arbiscan.io)
WARN          | worker secrets audit skipped (no scan-backend/node_modules) — run from a checkout with deps installed
MANUAL        | Pages env contract (go-list + checklist step 5): VITE_API_MODE=live, VITE_API_URL=https://vetted-scan-backend.dujar-coding.workers.dev,
MANUAL        |   VITE_REGISTRY_ADDRESS/VITE_GUARD_ADDRESS <- deployments/4663.json, VITE_WALLETCONNECT_PROJECT_ID

# summary: 6 pass, 1 warn, 0 FAIL, 5 pending-funds, 0 n/a
# AUDIT: no failures — remaining PENDING-FUNDS lines are the funded run's work
