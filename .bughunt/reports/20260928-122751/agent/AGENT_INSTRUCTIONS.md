# BugHunt Agent Instructions

You are fixing a BugHunt report. Treat `queue.json` as the machine-readable source of
truth.

Work in this order:

1. Start with execution errors because an analyzer that did not run leaves a blind spot.
2. Then process `queue.json` in order. Fix one finding or coherent repeated-signal
cluster at a time.
3. Inspect the surrounding code before editing. Do not blindly obey analyzer text.
4. Fix root causes. Do not add `noqa`, type ignores, Semgrep suppressions, or
broad exclusions unless the finding is proven false-positive and the suppression
is narrowly justified.
5. Run each item's `verification_command` after the fix, then the narrowest
relevant tests.
6. Re-run BugHunt after a cluster is cleared; line movement can make stale
report locations inaccurate.
7. If a finding reveals a genuine bug that escaped existing defenses, add it
to Bug Corpus / create a permanent detector or property test.
8. Never interpret a missing/crashed analyzer as clean.
9. If CI, a tool, or a test is hanging, flaky, environment-dependent, or failing
infrastructure reasons, use the `ci-fix-dont-freeze` skill before weakening, skipping,
quarantining, or disabling that defense. Preserve the failure evidence and
root-cause it.
10. Use `RISK_MAP.md` and cross-tool correlations to prioritize code with
uncovered branches plus independent analyzer agreement.

Useful files:

- `queue.json`: one item per finding, sorted for repair, including deterministic autofix
metadata.
- `AUTOFIX.md`: exactly which findings have safe or review-required deterministic fixes.
- `FIX_QUEUE.md`: human/agent readable queue.
- `BLIND_SPOTS.md`: analyzers that errored or never ran.
- `RISK_MAP.md` / `risk-map.json`: coverage-weighted file priority for repair and V2
exploration.
- `tasks/`: repeated-signal clusters with all known locations.
- `../findings.jsonl`: streaming-friendly one-record-per-finding representation.
- `../report.json`: full scan facts, commands, raw outputs, and statuses.
