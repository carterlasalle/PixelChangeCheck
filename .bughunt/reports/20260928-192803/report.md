# BugHunt Report

- Generated: `2026-09-28T19:28:03.383799-04:00`
- Profile: `all`
- Defense health: **67/100** (execution/defense health, not probability of bug-freedom)
- Elapsed: **9.0s**
- Raw normalized findings: **134**
- Logical issue clusters: **97** (non-destructive dedup view)
- Distinct repeated signals: **34**
- Cross-tool correlated locations: **0**
- Deterministic auto-fixes: **0** total (**0 safe**, 0 unsafe, 0 review-required)
- Agent repair queue: [`agent/FIX_QUEUE.md`](agent/FIX_QUEUE.md)
- Auto-fix inventory: [`agent/AUTOFIX.md`](agent/AUTOFIX.md)
- Coverage-weighted risk map: [`agent/RISK_MAP.md`](agent/RISK_MAP.md)
- Deduplicated repair queue: [`agent/DEDUPLICATED_QUEUE.md`](agent/DEDUPLICATED_QUEUE.md)
- Repair checklist: [`agent/CHECKLIST.md`](agent/CHECKLIST.md)

## ODC-style defect taxonomy

- **63** — `checking`
- **38** — `function`
- **30** — `algorithm`
- **2** — `timing/serialization`
- **1** — `build/package/merge`

## Highest-priority repeated signals

| Count | Tool / rule | Representative message |
|---:|---|---|
| 17 | `eslint:no-undef` | 'document' is not defined. |
| 4 | `oxlint:unicorn(prefer-add-event-listener)` | Prefer `addEventListener()` over their `on`-function counterparts. |
| 1 | `actionlint:[{"message":"label <path> is unknown. available labels are <path> <path> <pa…` | [{"message":"label \"macos-13\" is unknown. available labels are \"windows-latest\", \"windows-latest-8-cores\", \"windows-2025\", \"windows-2025-vs2026\", \"windows-2022\", \"windows-11-arm\", \"ubun |
| 1 | `eslint:no-unused-vars` | 'FMT_PNG' is assigned a value but never used. |
| 1 | `oxlint:eslint(no-unused-vars)` | Variable 'FMT_PNG' is declared but never used. Unused variables should start with a '_'. |
| 1 | `oxlint:typescript(no-extraneous-class)` | Unexpected class with only a constructor. |
| 1 | `oxlint:unicorn(no-new-array)` | Do not use `new Array(singleArgument)`. |
| 1 | `scc:BHGRAPH001` | scc index failed (exit 1): error: index: store: sqlite: database is locked  |
| 1 | `shellcheck:SC1036` | '(' is invalid here. Did you forget to escape it? |
| 1 | `shellcheck:SC1072` | Expected end of $(..) expression. Fix any mentioned problems and try again. |
| 1 | `shellcheck:SC1073` | Couldn't parse this command expansion. Fix to allow more checks. |
| 1 | `yamllint:<n>:<n> error wrong indentation: expected <n> but found <n> (indentation)` | 4:3       error    wrong indentation: expected 4 but found 2  (indentation) |
| 30 | `lizard:LIZARD_CCN` | receive_once has 177 NLOC, 33 CCN, 1139 token, 3 PARAM, 204 length, 0 ND |
| 17 | `oxlint:eslint(no-undef)` | 'location' is not defined. |
| 12 | `oxlint:eslint(no-plusplus)` | Unary operator '++' used. |
| 11 | `oxlint:eslint(no-implicit-globals)` | Unexpected function declaration in the global scope. |
| 6 | `oxlint:eslint(no-use-before-define)` | 'join' was used before it was defined. |
| 4 | `oxlint:eslint(no-bitwise)` | Unexpected use of `">>"`. |
| 3 | `oxlint:eslint(sort-vars)` | Variable declarations should be sorted |
| 2 | `hadolint:DL3008` | Pin versions in apt get install. Instead of `apt-get install <package>` use `apt-get install <package>=<version>` |
| 2 | `oxlint:eslint(max-lines-per-function)` | The function `parseMessage` has too many lines (70). Maximum allowed is 50. |
| 2 | `oxlint:eslint(no-undefined)` | Unexpected use of `undefined` |
| 2 | `oxlint:oxc(no-async-await)` | async is not allowed |
| 2 | `oxlint:unicorn(prefer-query-selector)` | Prefer `.querySelector()` over `.getElementById()`. |
| 1 | `hadolint:DL3025` | Use arguments JSON notation for CMD and ENTRYPOINT arguments |

## Hot files

- **96** findings — `src/server/renderer/client.js`
- **4** findings — `src/app/share.rs`
- **4** findings — `deploy/relay/provision-oracle.sh`
- **4** findings — `Dockerfile`
- **3** findings — `src/network/protocol.rs`
- **3** findings — `src/pcc/planner.rs`
- **2** findings — `src/pcc/detector.rs`
- **2** findings — `src/pcc/compositor.rs`
- **2** findings — `src/reach/mod.rs`
- **2** findings — `src/relay.rs`
- **1** findings — `src/app/view.rs`
- **1** findings — `src/encoder/mod.rs`
- **1** findings — `src/network/wire.rs`
- **1** findings — `src/server/renderer/web.rs`
- **1** findings — `src/audio/playout.rs`
- **1** findings — `src/pcc/types.rs`
- **1** findings — `src/telemetry/mod.rs`
- **1** findings — `src/main.rs`
- **1** findings — `scripts/smoke.sh`

## Defense results

| Status | Tool | Class | Findings | Time | Note |
|---|---|---|---:|---:|---|
| PASS | `complexity` | complexity-budgets | 0 | 0.1s |  |
| FINDINGS | `lizard` | cross-language-complexity | 30 | 0.4s |  |
| PASS | `semgrep` | semantic-static | 0 | 8.2s |  |
| FINDINGS | `system-ir` | structural-graph | 1 | 0.8s |  |
| PASS | `verify-gaps` | direct-verification | 0 | 1.8s |  |
| PASS | `protocol` | protocol-correctness | 0 | 0.1s |  |
| PASS | `data` | data-invariants | 0 | 0.1s |  |
| FINDINGS | `actionlint` | ci-correctness | 1 | 0.4s |  |
| FINDINGS | `shellcheck` | shell-correctness | 5 | 0.5s |  |
| PASS | `dotenv-linter` | config-correctness | 0 | 0.0s |  |
| FINDINGS | `hadolint` | container-correctness | 4 | 0.2s |  |
| PASS | `clippy` | rust-correctness | 0 | 6.2s |  |
| FINDINGS | `oxlint` | js-ts-correctness | 74 | 0.1s |  |
| FINDINGS | `eslint` | js-ts-correctness | 18 | 5.7s |  |
| PASS | `madge` | js-ts-architecture | 0 | 1.8s |  |
| PASS | `taplo` | config-correctness | 0 | 0.1s |  |
| FINDINGS | `yamllint` | config-correctness | 1 | 0.3s |  |
| N/A | `codeql` | whole-program | 0 | 0.0s | not applicable: no first-party Python capability detected |
| N/A | `pysa` | taint | 0 | 0.0s | not applicable: no first-party Python capability detected |
| SKIPPED | `mutmut` | mutation | 0 | 0.0s | explicitly skipped by user |
| N/A | `compile` | syntax | 0 | 0.0s | not applicable: no first-party Python capability detected |
| N/A | `ruff` | lint | 0 | 0.0s | not applicable: no first-party Python capability detected |
| N/A | `basedpyright` | types | 0 | 0.0s | not applicable: no first-party Python capability detected |
| N/A | `mypy` | types | 0 | 0.0s | not applicable: no first-party Python capability detected |
| N/A | `ty` | types | 0 | 0.0s | not applicable: no first-party Python capability detected |
| N/A | `pyrefly` | types | 0 | 0.0s | not applicable: no first-party Python capability detected |
| N/A | `pylint` | lint | 0 | 0.0s | not applicable: no first-party Python capability detected |
| N/A | `pylint-tests` | lint | 0 | 0.0s | not applicable: no first-party Python capability detected |
| N/A | `policy` | repository-policy | 0 | 0.0s | not applicable: no first-party Python capability detected |
| N/A | `complexipy` | cognitive-complexity | 0 | 0.0s | not applicable: no first-party Python capability detected |
| N/A | `radon` | maintainability | 0 | 0.0s | not applicable: no first-party Python capability detected |
| N/A | `vulture` | dead-code | 0 | 0.0s | not applicable: no first-party Python capability detected |
| N/A | `bandit` | security | 0 | 0.0s | not applicable: no first-party Python capability detected |
| N/A | `deptry` | dependencies | 0 | 0.0s | not applicable: no first-party Python capability detected |
| N/A | `import-linter` | architecture | 0 | 0.0s | not applicable: no first-party Python capability detected |
| N/A | `ast-grep` | structural | 0 | 0.0s | not applicable: no first-party Python capability detected |
| N/A | `deal` | contracts | 0 | 0.0s | not applicable: no first-party Python capability detected |
| N/A | `crosshair` | symbolic | 0 | 0.0s | not applicable: no first-party Python capability detected |
| N/A | `pytest` | tests/property/state | 0 | 0.0s | not applicable: no first-party Python capability detected |
| N/A | `bugcorpus` | historical-bugs | 0 | 0.0s | not applicable: no .bugcorpus corpus in this repository |
| N/A | `tracelayer` | direct-verification | 0 | 0.0s | not applicable: no .trace workspace in this repository |
| SKIPPED | `schemathesis` | api-fuzz | 0 | 0.0s | no safe runnable API target discovered/configured |
| N/A | `atheris` | coverage-fuzz | 0 | 0.0s | not applicable: no first-party Python capability detected |
| SKIPPED | `custom` | custom | 0 | 0.0s | no high-confidence repository-specific semantic campaign could be inferred |
| N/A | `oasdiff` | api-compatibility | 0 | 0.0s | not applicable: no openapi capability detected |
| N/A | `buf` | schema-compatibility | 0 | 0.0s | not applicable: no protobuf capability detected |
| N/A | `sqlfluff` | sql-correctness | 0 | 0.0s | not applicable: no sql capability detected |
| N/A | `squawk` | migration-correctness | 0 | 0.0s | not applicable: no postgres-migrations capability detected |
| N/A | `tflint` | terraform-correctness | 0 | 0.0s | not applicable: no terraform capability detected |
| N/A | `golangci-lint` | go-correctness | 0 | 0.0s | not applicable: no go capability detected |
| N/A | `cppcheck` | cpp-correctness | 0 | 0.0s | not applicable: no cpp capability detected |
| N/A | `clang-tidy` | cpp-correctness | 0 | 0.0s | not applicable: no cpp-compile-db capability detected |
| N/A | `infer` | whole-program-native | 0 | 0.0s | not applicable: no cpp-compile-db capability detected |
| N/A | `phpstan` | php-types | 0 | 0.0s | not applicable: no php capability detected |
| N/A | `react-doctor` | react-correctness | 0 | 0.0s | not applicable: no react capability detected |
| SKIPPED | `tsc` | ts-types | 0 | 0.0s | TypeScript detected but no root tsconfig.json project exists |
| SKIPPED | `knip` | js-ts-dead-contract | 0 | 0.0s | no root package.json: knip requires a project manifest at the scan root |
| SKIPPED | `publint` | package-correctness | 0 | 0.0s | publint is installed but package.json is missing name/version; add both fields to make the package publishable |
| N/A | `check-jsonschema` | schema-correctness | 0 | 0.0s | not applicable: no schema-ref capability detected |
| N/A | `alembic-check` | migration-drift | 0 | 0.0s | not applicable: no alembic capability detected |
| N/A | `django-migrations` | migration-drift | 0 | 0.0s | not applicable: no django capability detected |
| N/A | `pact-contracts` | service-contract | 0 | 0.0s | not applicable: no pact capability detected |

## Findings

### BH-E691EB215F2994FF — actionlint

- Location: `<unknown>:?:?`
- Severity: `error`
- Signal: `actionlint:[{"message":"label <path> is unknown. available labels are <path> <path> <path> <path> <path> <path> <path> <path> <path> <path> <path> <path> <path> <path> <path> <path> <path> <p`
- Fingerprint: `e691eb215f2994ff`

[{"message":"label \"macos-13\" is unknown. available labels are \"windows-latest\", \"windows-latest-8-cores\", \"windows-2025\", \"windows-2025-vs2026\", \"windows-2022\", \"windows-11-arm\", \"ubuntu-slim\", \"ubuntu-latest\", \"ubuntu-latest-4-cores\", \"ubuntu-latest-8-cores\", \"ubuntu-latest-16-cores\", \"ubuntu-24.04\", \"ubuntu-24.04-arm\", \"ubuntu-22.04\", \"ubuntu-22.04-arm\", \"macos-latest\", \"macos-latest-xlarge\", \"macos-latest-large\", \"macos-26-intel\", \"macos-26-xlarge\", \"macos-26-large\", \"macos-26\", \"macos-15-intel\", \"macos-15-xlarge\", \"macos-15-large\", \"macos-15\", \"macos-14-xlarge\", \"macos-14-large\", \"macos-14\", \"self-hosted\", \"x64\", \"arm\", \"arm64\", \"linux\", \"macos\", \"windows\". if it is a custom label for self-hosted runner, set list of labels in actionlint.yaml config file","filepath":".github/workflows/release.yml","line":37,"column":43,"kind":"runner-label","snippet":"        os: [ubuntu-latest, macos-latest, macos-13, window

### BH-C5AC5B4CB3B55E5D — scc / BHGRAPH001

- Location: `<unknown>:?:?`
- Severity: `error`
- Signal: `scc:BHGRAPH001`
- Fingerprint: `c5ac5b4cb3b55e5d`

scc index failed (exit 1): error: index: store: sqlite: database is locked


### BH-F02F2F421D4184BE — yamllint

- Location: `<unknown>:?:?`
- Severity: `error`
- Signal: `yamllint:<n>:<n> error wrong indentation: expected <n> but found <n> (indentation)`
- Fingerprint: `f02f2f421d4184be`

4:3       error    wrong indentation: expected 4 but found 2  (indentation)

### BH-B9BDACF1820C06C9 — shellcheck / SC1073

- Location: `deploy/relay/provision-oracle.sh:182:72`
- Severity: `error`
- Signal: `shellcheck:SC1073`
- Fingerprint: `b9bdacf1820c06c9`

Couldn't parse this command expansion. Fix to allow more checks.

### BH-13EFABB6E6F20A4F — shellcheck / SC1036

- Location: `deploy/relay/provision-oracle.sh:182:94`
- Severity: `error`
- Signal: `shellcheck:SC1036`
- Fingerprint: `13efabb6e6f20a4f`

'(' is invalid here. Did you forget to escape it?

### BH-73679292A497882F — shellcheck / SC1072

- Location: `deploy/relay/provision-oracle.sh:182:94`
- Severity: `error`
- Signal: `shellcheck:SC1072`
- Fingerprint: `73679292a497882f`

Expected end of $(..) expression. Fix any mentioned problems and try again.

### BH-F01ED09DE398FD59 — oxlint / eslint(no-unused-vars)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `error`
- Signal: `oxlint:eslint(no-unused-vars)`
- Fingerprint: `f01ed09de398fd59`

Variable 'FMT_PNG' is declared but never used. Unused variables should start with a '_'.

### BH-B21015E7721FFA3E — oxlint / typescript(no-extraneous-class)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `error`
- Signal: `oxlint:typescript(no-extraneous-class)`
- Fingerprint: `b21015e7721ffa3e`

Unexpected class with only a constructor.

### BH-E105D5995218A028 — oxlint / unicorn(no-new-array)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `error`
- Signal: `oxlint:unicorn(no-new-array)`
- Fingerprint: `e105d5995218a028`

Do not use `new Array(singleArgument)`.

### BH-3FE9863C39C16172 — oxlint / unicorn(prefer-add-event-listener)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `error`
- Signal: `oxlint:unicorn(prefer-add-event-listener)`
- Fingerprint: `3fe9863c39c16172`

Prefer `addEventListener()` over their `on`-function counterparts.

### BH-3FE9863C39C16172 — oxlint / unicorn(prefer-add-event-listener)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `error`
- Signal: `oxlint:unicorn(prefer-add-event-listener)`
- Fingerprint: `3fe9863c39c16172`

Prefer `addEventListener()` over their `on`-function counterparts.

### BH-3FE9863C39C16172 — oxlint / unicorn(prefer-add-event-listener)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `error`
- Signal: `oxlint:unicorn(prefer-add-event-listener)`
- Fingerprint: `3fe9863c39c16172`

Prefer `addEventListener()` over their `on`-function counterparts.

### BH-3FE9863C39C16172 — oxlint / unicorn(prefer-add-event-listener)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `error`
- Signal: `oxlint:unicorn(prefer-add-event-listener)`
- Fingerprint: `3fe9863c39c16172`

Prefer `addEventListener()` over their `on`-function counterparts.

### BH-F9EE3C2A97D02E90 — eslint / no-unused-vars

- Location: `src/server/renderer/client.js:33:7`
- Severity: `error`
- Signal: `eslint:no-unused-vars`
- Fingerprint: `f9ee3c2a97d02e90`

'FMT_PNG' is assigned a value but never used.

### BH-442844C5E0F800CC — eslint / no-undef

- Location: `src/server/renderer/client.js:137:21`
- Severity: `error`
- Signal: `eslint:no-undef`
- Fingerprint: `442844c5e0f800cc`

'document' is not defined.

### BH-E978965F5F67E680 — eslint / no-undef

- Location: `src/server/renderer/client.js:188:24`
- Severity: `error`
- Signal: `eslint:no-undef`
- Fingerprint: `e978965f5f67e680`

'Blob' is not defined.

### BH-DEBE143C15DED943 — eslint / no-undef

- Location: `src/server/renderer/client.js:189:28`
- Severity: `error`
- Signal: `eslint:no-undef`
- Fingerprint: `debe143c15ded943`

'createImageBitmap' is not defined.

### BH-0EC0BD628C3EB543 — eslint / no-undef

- Location: `src/server/renderer/client.js:193:23`
- Severity: `error`
- Signal: `eslint:no-undef`
- Fingerprint: `0ec0bd628c3eb543`

'OffscreenCanvas' is not defined.

### BH-0B6A9C340807CC3A — eslint / no-undef

- Location: `src/server/renderer/client.js:348:32`
- Severity: `error`
- Signal: `eslint:no-undef`
- Fingerprint: `0b6a9c340807cc3a`

'TextDecoder' is not defined.

### BH-A8E315B9664ED7F1 — eslint / no-undef

- Location: `src/server/renderer/client.js:358:19`
- Severity: `error`
- Signal: `eslint:no-undef`
- Fingerprint: `a8e315b9664ed7f1`

'TextEncoder' is not defined.

### BH-442844C5E0F800CC — eslint / no-undef

- Location: `src/server/renderer/client.js:383:19`
- Severity: `error`
- Signal: `eslint:no-undef`
- Fingerprint: `442844c5e0f800cc`

'document' is not defined.

### BH-D7D0A91F9B426F2E — eslint / no-undef

- Location: `src/server/renderer/client.js:391:5`
- Severity: `error`
- Signal: `eslint:no-undef`
- Fingerprint: `d7d0a91f9b426f2e`

'requestAnimationFrame' is not defined.

### BH-7CFA34A2865BBB94 — eslint / no-undef

- Location: `src/server/renderer/client.js:404:31`
- Severity: `error`
- Signal: `eslint:no-undef`
- Fingerprint: `7cfa34a2865bbb94`

'ImageData' is not defined.

### BH-442844C5E0F800CC — eslint / no-undef

- Location: `src/server/renderer/client.js:412:16`
- Severity: `error`
- Signal: `eslint:no-undef`
- Fingerprint: `442844c5e0f800cc`

'document' is not defined.

### BH-D10B23A3D6562ADE — eslint / no-undef

- Location: `src/server/renderer/client.js:417:19`
- Severity: `error`
- Signal: `eslint:no-undef`
- Fingerprint: `d10b23a3d6562ade`

'location' is not defined.

### BH-D10B23A3D6562ADE — eslint / no-undef

- Location: `src/server/renderer/client.js:418:31`
- Severity: `error`
- Signal: `eslint:no-undef`
- Fingerprint: `d10b23a3d6562ade`

'location' is not defined.

### BH-01E3C22BBBE38E0B — eslint / no-undef

- Location: `src/server/renderer/client.js:419:24`
- Severity: `error`
- Signal: `eslint:no-undef`
- Fingerprint: `01e3c22bbbe38e0b`

'WebSocket' is not defined.

### BH-779EAA82D656D60C — eslint / no-undef

- Location: `src/server/renderer/client.js:435:7`
- Severity: `error`
- Signal: `eslint:no-undef`
- Fingerprint: `779eaa82d656d60c`

'setTimeout' is not defined.

### BH-01E3C22BBBE38E0B — eslint / no-undef

- Location: `src/server/renderer/client.js:441:51`
- Severity: `error`
- Signal: `eslint:no-undef`
- Fingerprint: `01e3c22bbbe38e0b`

'WebSocket' is not defined.

### BH-01E3C22BBBE38E0B — eslint / no-undef

- Location: `src/server/renderer/client.js:500:51`
- Severity: `error`
- Signal: `eslint:no-undef`
- Fingerprint: `01e3c22bbbe38e0b`

'WebSocket' is not defined.

### BH-229054FF4267132E — eslint / no-undef

- Location: `src/server/renderer/client.js:526:1`
- Severity: `error`
- Signal: `eslint:no-undef`
- Fingerprint: `229054ff4267132e`

'window' is not defined.

### BH-69C8D25F56F669A2 — hadolint / DL3008

- Location: `Dockerfile:5:1`
- Severity: `warning`
- Signal: `hadolint:DL3008`
- Fingerprint: `69c8d25f56f669a2`

Pin versions in apt get install. Instead of `apt-get install <package>` use `apt-get install <package>=<version>`

### BH-69C8D25F56F669A2 — hadolint / DL3008

- Location: `Dockerfile:15:1`
- Severity: `warning`
- Signal: `hadolint:DL3008`
- Fingerprint: `69c8d25f56f669a2`

Pin versions in apt get install. Instead of `apt-get install <package>` use `apt-get install <package>=<version>`

### BH-74DD16E6C8721C49 — hadolint / DL3025

- Location: `Dockerfile:23:1`
- Severity: `warning`
- Signal: `hadolint:DL3025`
- Fingerprint: `74dd16e6c8721c49`

Use arguments JSON notation for CMD and ENTRYPOINT arguments

### BH-161A4B78F1E08411 — lizard / LIZARD_CCN

- Location: `src/app/share.rs:148:?`
- Severity: `warning`
- Signal: `lizard:LIZARD_CCN`
- Fingerprint: `161a4b78f1e08411`

run_share has 137 NLOC, 23 CCN, 905 token, 2 PARAM, 170 length, 0 ND

### BH-2039419E579FD5E6 — lizard / LIZARD_CCN

- Location: `src/app/share.rs:319:?`
- Severity: `warning`
- Signal: `lizard:LIZARD_CCN`
- Fingerprint: `2039419e579fd5e6`

start_web has 56 NLOC, 15 CCN, 387 token, 7 PARAM, 60 length, 0 ND

### BH-FFEFB93B834E9D10 — lizard / LIZARD_CCN

- Location: `src/app/share.rs:490:?`
- Severity: `warning`
- Signal: `lizard:LIZARD_CCN`
- Fingerprint: `ffefb93b834e9d10`

serve_viewer has 151 NLOC, 17 CCN, 980 token, 8 PARAM, 168 length, 0 ND

### BH-B0598EFCA2F69C30 — lizard / LIZARD_CCN

- Location: `src/app/share.rs:761:?`
- Severity: `warning`
- Signal: `lizard:LIZARD_CCN`
- Fingerprint: `b0598efca2f69c30`

capture_loop has 184 NLOC, 36 CCN, 1406 token, 12 PARAM, 224 length, 0 ND

### BH-93903C0364AF2C55 — lizard / LIZARD_CCN

- Location: `src/app/view.rs:185:?`
- Severity: `warning`
- Signal: `lizard:LIZARD_CCN`
- Fingerprint: `93903c0364af2c55`

receive_once has 177 NLOC, 33 CCN, 1139 token, 3 PARAM, 204 length, 0 ND

### BH-204E753A08C85D13 — lizard / LIZARD_CCN

- Location: `src/audio/playout.rs:116:?`
- Severity: `warning`
- Signal: `lizard:LIZARD_CCN`
- Fingerprint: `204e753a08c85d13`

start_default_output has 38 NLOC, 15 CCN, 264 token, 1 PARAM, 38 length, 0 ND

### BH-75B6EA5BF08776C7 — lizard / LIZARD_CCN

- Location: `src/encoder/mod.rs:81:?`
- Severity: `warning`
- Signal: `lizard:LIZARD_CCN`
- Fingerprint: `75b6ea5bf08776c7`

decode_snapshot has 45 NLOC, 14 CCN, 277 token, 4 PARAM, 49 length, 0 ND

### BH-26ACBEAC796CD5C6 — lizard / LIZARD_CCN

- Location: `src/main.rs:184:?`
- Severity: `warning`
- Signal: `lizard:LIZARD_CCN`
- Fingerprint: `26acbeac796cd5c6`

main has 130 NLOC, 24 CCN, 803 token, 0 PARAM, 138 length, 0 ND

### BH-284D4343F6622EC7 — lizard / LIZARD_CCN

- Location: `src/network/protocol.rs:185:?`
- Severity: `warning`
- Signal: `lizard:LIZARD_CCN`
- Fingerprint: `284d4343f6622ec7`

encode_body has 82 NLOC, 3 CCN, 647 token, 2 PARAM, 82 length, 0 ND

### BH-A19266937FFA9FB5 — lizard / LIZARD_CCN

- Location: `src/network/protocol.rs:413:?`
- Severity: `warning`
- Signal: `lizard:LIZARD_CCN`
- Fingerprint: `a19266937ffa9fb5`

message has 105 NLOC, 43 CCN, 715 token, 1 PARAM, 108 length, 0 ND

### BH-F7A620F417529B9B — lizard / LIZARD_CCN

- Location: `src/network/protocol.rs:522:?`
- Severity: `warning`
- Signal: `lizard:LIZARD_CCN`
- Fingerprint: `f7a620f417529b9b`

op has 56 NLOC, 15 CCN, 309 token, 1 PARAM, 59 length, 0 ND

### BH-A11AF4AAD82EFCFA — lizard / LIZARD_CCN

- Location: `src/network/wire.rs:129:?`
- Severity: `warning`
- Signal: `lizard:LIZARD_CCN`
- Fingerprint: `a11af4aad82efcfa`

validate has 47 NLOC, 18 CCN, 281 token, 3 PARAM, 49 length, 0 ND

### BH-D1A116DCF78BD907 — lizard / LIZARD_CCN

- Location: `src/pcc/compositor.rs:212:?`
- Severity: `warning`
- Signal: `lizard:LIZARD_CCN`
- Fingerprint: `d1a116dcf78bd907`

apply_ops has 77 NLOC, 12 CCN, 573 token, 4 PARAM, 88 length, 0 ND

### BH-15A5192DAD0A0BE6 — lizard / LIZARD_CCN

- Location: `src/pcc/compositor.rs:379:?`
- Severity: `warning`
- Signal: `lizard:LIZARD_CCN`
- Fingerprint: `15a5192dad0a0be6`

blit_region has 18 NLOC, 2 CCN, 139 token, 9 PARAM, 18 length, 0 ND

### BH-2CCFDA0D0D8F2B1A — lizard / LIZARD_CCN

- Location: `src/pcc/detector.rs:147:?`
- Severity: `warning`
- Signal: `lizard:LIZARD_CCN`
- Fingerprint: `2ccfda0d0d8f2b1a`

detect has 119 NLOC, 28 CCN, 902 token, 5 PARAM, 139 length, 0 ND

### BH-B45136FA8482B143 — lizard / LIZARD_CCN

- Location: `src/pcc/detector.rs:465:?`
- Severity: `warning`
- Signal: `lizard:LIZARD_CCN`
- Fingerprint: `b45136fa8482b143`

verify_displacement_of has 34 NLOC, 11 CCN, 274 token, 10 PARAM, 36 length, 0 ND

### BH-4B47EFEB3FCEB2C1 — lizard / LIZARD_CCN

- Location: `src/pcc/planner.rs:132:?`
- Severity: `warning`
- Signal: `lizard:LIZARD_CCN`
- Fingerprint: `4b47efeb3fceb2c1`

plan has 63 NLOC, 12 CCN, 416 token, 6 PARAM, 77 length, 0 ND

### BH-AC2617C20C0C131C — lizard / LIZARD_CCN

- Location: `src/pcc/planner.rs:289:?`
- Severity: `warning`
- Signal: `lizard:LIZARD_CCN`
- Fingerprint: `ac2617c20c0c131c`

check_shift has 51 NLOC, 12 CCN, 414 token, 10 PARAM, 60 length, 0 ND

### BH-FC3B755BB0255BB4 — lizard / LIZARD_CCN

- Location: `src/pcc/planner.rs:365:?`
- Severity: `warning`
- Signal: `lizard:LIZARD_CCN`
- Fingerprint: `fc3b755bb0255bb4`

subtract_region has 44 NLOC, 15 CCN, 358 token, 5 PARAM, 51 length, 0 ND

### BH-7F5239537BBBF2EE — lizard / LIZARD_CCN

- Location: `src/pcc/types.rs:203:?`
- Severity: `warning`
- Signal: `lizard:LIZARD_CCN`
- Fingerprint: `7f5239537bbbf2ee`

capture_frame has 20 NLOC, 1 CCN, 48 token, 1 PARAM, 94 length, 0 ND

### BH-11B7A076EA0F0E09 — lizard / LIZARD_CCN

- Location: `src/reach/mod.rs:138:?`
- Severity: `warning`
- Signal: `lizard:LIZARD_CCN`
- Fingerprint: `11b7a076ea0f0e09`

classify has 24 NLOC, 17 CCN, 282 token, 1 PARAM, 27 length, 0 ND

### BH-60F986CB594AE8C9 — lizard / LIZARD_CCN

- Location: `src/reach/mod.rs:327:?`
- Severity: `warning`
- Signal: `lizard:LIZARD_CCN`
- Fingerprint: `60f986cb594ae8c9`

diagnose has 65 NLOC, 12 CCN, 334 token, 1 PARAM, 77 length, 0 ND

### BH-817E2399756FFDE2 — lizard / LIZARD_CCN

- Location: `src/relay.rs:119:?`
- Severity: `warning`
- Signal: `lizard:LIZARD_CCN`
- Fingerprint: `817e2399756ffde2`

connect has 45 NLOC, 14 CCN, 311 token, 6 PARAM, 51 length, 0 ND

### BH-F375F7D38217BE9C — lizard / LIZARD_CCN

- Location: `src/relay.rs:293:?`
- Severity: `warning`
- Signal: `lizard:LIZARD_CCN`
- Fingerprint: `f375f7d38217be9c`

handle_client has 147 NLOC, 27 CCN, 981 token, 8 PARAM, 174 length, 0 ND

### BH-9881F18300301C12 — oxlint / import(unambiguous)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:import(unambiguous)`
- Fingerprint: `9881f18300301c12`

This module could be mistakenly parsed as script instead of module

### BH-BA6B7D909DB906F7 — oxlint / eslint(max-classes-per-file)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(max-classes-per-file)`
- Fingerprint: `ba6b7d909db906f7`

File has too many classes (5). Maximum allowed is 1

### BH-058B7E6479AE1D87 — oxlint / eslint(max-lines)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(max-lines)`
- Fingerprint: `058b7e6479ae1d87`

File has too many lines (538).

### BH-F365632C92257DD2 — oxlint / eslint(no-implicit-globals)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-implicit-globals)`
- Fingerprint: `f365632c92257dd2`

Unexpected function declaration in the global scope.

### BH-F365632C92257DD2 — oxlint / eslint(no-implicit-globals)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-implicit-globals)`
- Fingerprint: `f365632c92257dd2`

Unexpected function declaration in the global scope.

### BH-F365632C92257DD2 — oxlint / eslint(no-implicit-globals)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-implicit-globals)`
- Fingerprint: `f365632c92257dd2`

Unexpected function declaration in the global scope.

### BH-F365632C92257DD2 — oxlint / eslint(no-implicit-globals)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-implicit-globals)`
- Fingerprint: `f365632c92257dd2`

Unexpected function declaration in the global scope.

### BH-F365632C92257DD2 — oxlint / eslint(no-implicit-globals)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-implicit-globals)`
- Fingerprint: `f365632c92257dd2`

Unexpected function declaration in the global scope.

### BH-F365632C92257DD2 — oxlint / eslint(no-implicit-globals)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-implicit-globals)`
- Fingerprint: `f365632c92257dd2`

Unexpected function declaration in the global scope.

### BH-F365632C92257DD2 — oxlint / eslint(no-implicit-globals)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-implicit-globals)`
- Fingerprint: `f365632c92257dd2`

Unexpected function declaration in the global scope.

### BH-F365632C92257DD2 — oxlint / eslint(no-implicit-globals)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-implicit-globals)`
- Fingerprint: `f365632c92257dd2`

Unexpected function declaration in the global scope.

### BH-F365632C92257DD2 — oxlint / eslint(no-implicit-globals)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-implicit-globals)`
- Fingerprint: `f365632c92257dd2`

Unexpected function declaration in the global scope.

### BH-F365632C92257DD2 — oxlint / eslint(no-implicit-globals)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-implicit-globals)`
- Fingerprint: `f365632c92257dd2`

Unexpected function declaration in the global scope.

### BH-F365632C92257DD2 — oxlint / eslint(no-implicit-globals)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-implicit-globals)`
- Fingerprint: `f365632c92257dd2`

Unexpected function declaration in the global scope.

### BH-221968370A461DA2 — oxlint / eslint(no-inline-comments)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-inline-comments)`
- Fingerprint: `221968370a461da2`

Unexpected comment inline with code

### BH-9F840E446F36BA51 — oxlint / eslint(no-undef)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-undef)`
- Fingerprint: `9f840e446f36ba51`

'location' is not defined.

### BH-9F840E446F36BA51 — oxlint / eslint(no-undef)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-undef)`
- Fingerprint: `9f840e446f36ba51`

'location' is not defined.

### BH-DF140AD0A3B9B32F — oxlint / eslint(no-undef)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-undef)`
- Fingerprint: `df140ad0a3b9b32f`

'TextEncoder' is not defined.

### BH-E006C52C8E0C3D1A — oxlint / eslint(no-undef)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-undef)`
- Fingerprint: `e006c52c8e0c3d1a`

'TextDecoder' is not defined.

### BH-18F1BD3D731C1FDF — oxlint / eslint(no-undef)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-undef)`
- Fingerprint: `18f1bd3d731c1fdf`

'WebSocket' is not defined.

### BH-18F1BD3D731C1FDF — oxlint / eslint(no-undef)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-undef)`
- Fingerprint: `18f1bd3d731c1fdf`

'WebSocket' is not defined.

### BH-18F1BD3D731C1FDF — oxlint / eslint(no-undef)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-undef)`
- Fingerprint: `18f1bd3d731c1fdf`

'WebSocket' is not defined.

### BH-2A7DAD091CACD33F — oxlint / eslint(no-undef)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-undef)`
- Fingerprint: `2a7dad091cacd33f`

'requestAnimationFrame' is not defined.

### BH-069DF221D199C422 — oxlint / eslint(no-undef)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-undef)`
- Fingerprint: `069df221d199c422`

'createImageBitmap' is not defined.

### BH-334C06DB21A13A74 — oxlint / eslint(no-undef)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-undef)`
- Fingerprint: `334c06db21a13a74`

'OffscreenCanvas' is not defined.

### BH-52AD747F4C7B0B19 — oxlint / eslint(no-undef)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-undef)`
- Fingerprint: `52ad747f4c7b0b19`

'document' is not defined.

### BH-52AD747F4C7B0B19 — oxlint / eslint(no-undef)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-undef)`
- Fingerprint: `52ad747f4c7b0b19`

'document' is not defined.

### BH-52AD747F4C7B0B19 — oxlint / eslint(no-undef)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-undef)`
- Fingerprint: `52ad747f4c7b0b19`

'document' is not defined.

### BH-473942BA7A693751 — oxlint / eslint(no-undef)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-undef)`
- Fingerprint: `473942ba7a693751`

'Blob' is not defined.

### BH-7B36C35CD86B21B7 — oxlint / eslint(no-undef)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-undef)`
- Fingerprint: `7b36c35cd86b21b7`

'ImageData' is not defined.

### BH-28BB56463A161668 — oxlint / eslint(no-undef)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-undef)`
- Fingerprint: `28bb56463a161668`

'setTimeout' is not defined.

### BH-FF29A73AB7B354DB — oxlint / eslint(no-undef)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-undef)`
- Fingerprint: `ff29a73ab7b354db`

'window' is not defined.

### BH-9D54B0E8E4846BB4 — oxlint / eslint(no-plusplus)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-plusplus)`
- Fingerprint: `9d54b0e8e4846bb4`

Unary operator '++' used.

### BH-775675C146C7E0DA — oxlint / eslint(sort-vars)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(sort-vars)`
- Fingerprint: `775675c146c7e0da`

Variable declarations should be sorted

### BH-9D54B0E8E4846BB4 — oxlint / eslint(no-plusplus)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-plusplus)`
- Fingerprint: `9d54b0e8e4846bb4`

Unary operator '++' used.

### BH-EF9A13050E6C505A — oxlint / eslint(no-bitwise)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-bitwise)`
- Fingerprint: `ef9a13050e6c505a`

Unexpected use of `">>"`.

### BH-9D54B0E8E4846BB4 — oxlint / eslint(no-plusplus)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-plusplus)`
- Fingerprint: `9d54b0e8e4846bb4`

Unary operator '++' used.

### BH-A1DB7EF4C1919AFC — oxlint / eslint(no-bitwise)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-bitwise)`
- Fingerprint: `a1db7ef4c1919afc`

Unexpected use of `"|"`.

### BH-D4004F3E52B67ACB — oxlint / eslint(no-bitwise)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-bitwise)`
- Fingerprint: `d4004f3e52b67acb`

Unexpected use of `"<<"`.

### BH-820E23BFB99D04B8 — oxlint / eslint(no-bitwise)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-bitwise)`
- Fingerprint: `820e23bfb99d04b8`

Unexpected use of `"&"`.

### BH-9D54B0E8E4846BB4 — oxlint / eslint(no-plusplus)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-plusplus)`
- Fingerprint: `9d54b0e8e4846bb4`

Unary operator '++' used.

### BH-9D54B0E8E4846BB4 — oxlint / eslint(no-plusplus)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-plusplus)`
- Fingerprint: `9d54b0e8e4846bb4`

Unary operator '++' used.

### BH-9D54B0E8E4846BB4 — oxlint / eslint(no-plusplus)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-plusplus)`
- Fingerprint: `9d54b0e8e4846bb4`

Unary operator '++' used.

### BH-9D54B0E8E4846BB4 — oxlint / eslint(no-plusplus)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-plusplus)`
- Fingerprint: `9d54b0e8e4846bb4`

Unary operator '++' used.

### BH-2B475FC7B6315C2D — oxlint / eslint(no-undefined)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-undefined)`
- Fingerprint: `2b475fc7b6315c2d`

Unexpected use of `undefined`

### BH-348F358994D72353 — oxlint / oxc(no-async-await)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:oxc(no-async-await)`
- Fingerprint: `348f358994d72353`

async is not allowed

### BH-2B475FC7B6315C2D — oxlint / eslint(no-undefined)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-undefined)`
- Fingerprint: `2b475fc7b6315c2d`

Unexpected use of `undefined`

### BH-32E00442ABED73B1 — oxlint / eslint(no-use-before-define)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-use-before-define)`
- Fingerprint: `32e00442abed73b1`

'join' was used before it was defined.

### BH-A4B4604200E06339 — oxlint / oxc(no-rest-spread-properties)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:oxc(no-rest-spread-properties)`
- Fingerprint: `a4b4604200e06339`

object spread property are not allowed. 

### BH-CA0C4D415D6C58E7 — oxlint / eslint(no-use-before-define)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-use-before-define)`
- Fingerprint: `ca0c4d415d6c58e7`

'blit' was used before it was defined.

### BH-607297B9C7FE47B5 — oxlint / eslint(no-use-before-define)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-use-before-define)`
- Fingerprint: `607297b9c7fe47b5`

'fill' was used before it was defined.

### BH-4C323958A0C748BB — oxlint / eslint(no-use-before-define)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-use-before-define)`
- Fingerprint: `4c323958a0c748bb`

'blitRegion' was used before it was defined.

### BH-9D54B0E8E4846BB4 — oxlint / eslint(no-plusplus)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-plusplus)`
- Fingerprint: `9d54b0e8e4846bb4`

Unary operator '++' used.

### BH-9D54B0E8E4846BB4 — oxlint / eslint(no-plusplus)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-plusplus)`
- Fingerprint: `9d54b0e8e4846bb4`

Unary operator '++' used.

### BH-9D54B0E8E4846BB4 — oxlint / eslint(no-plusplus)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-plusplus)`
- Fingerprint: `9d54b0e8e4846bb4`

Unary operator '++' used.

### BH-9D54B0E8E4846BB4 — oxlint / eslint(no-plusplus)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-plusplus)`
- Fingerprint: `9d54b0e8e4846bb4`

Unary operator '++' used.

### BH-E8C2F1A6730C5798 — oxlint / eslint(max-lines-per-function)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(max-lines-per-function)`
- Fingerprint: `e8c2f1a6730c5798`

The function `parseMessage` has too many lines (70). Maximum allowed is 50.

### BH-9D54B0E8E4846BB4 — oxlint / eslint(no-plusplus)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-plusplus)`
- Fingerprint: `9d54b0e8e4846bb4`

Unary operator '++' used.

### BH-775675C146C7E0DA — oxlint / eslint(sort-vars)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(sort-vars)`
- Fingerprint: `775675c146c7e0da`

Variable declarations should be sorted

### BH-775675C146C7E0DA — oxlint / eslint(sort-vars)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(sort-vars)`
- Fingerprint: `775675c146c7e0da`

Variable declarations should be sorted

### BH-534A1CFE8B2CE037 — oxlint / unicorn(prefer-query-selector)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:unicorn(prefer-query-selector)`
- Fingerprint: `534a1cfe8b2ce037`

Prefer `.querySelector()` over `.getElementById()`.

### BH-2752E00BE3332DF2 — oxlint / eslint(class-methods-use-this)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(class-methods-use-this)`
- Fingerprint: `2752e00be3332df2`

Expected method `status` to have this.

### BH-534A1CFE8B2CE037 — oxlint / unicorn(prefer-query-selector)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:unicorn(prefer-query-selector)`
- Fingerprint: `534a1cfe8b2ce037`

Prefer `.querySelector()` over `.getElementById()`.

### BH-F3D8F5765AD53525 — oxlint / eslint(no-use-before-define)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-use-before-define)`
- Fingerprint: `f3d8f5765ad53525`

'encodeAck' was used before it was defined.

### BH-8A1B62A4EC71713B — oxlint / eslint(max-lines-per-function)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(max-lines-per-function)`
- Fingerprint: `8a1b62a4ec71713b`

The async method `onMessage` has too many lines (52). Maximum allowed is 50.

### BH-348F358994D72353 — oxlint / oxc(no-async-await)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:oxc(no-async-await)`
- Fingerprint: `348f358994d72353`

async is not allowed

### BH-FDB09C23C5592E0B — oxlint / eslint(no-use-before-define)

- Location: `src/server/renderer/client.js:?:?`
- Severity: `warning`
- Signal: `oxlint:eslint(no-use-before-define)`
- Fingerprint: `fdb09c23c5592e0b`

'encodeRequestKeyframe' was used before it was defined.

### BH-7D5EB110F217DE1D — lizard / LIZARD_CCN

- Location: `src/server/renderer/client.js:79:?`
- Severity: `warning`
- Signal: `lizard:LIZARD_CCN`
- Fingerprint: `7d5eb110f217de1d`

lz4Decode has 31 NLOC, 14 CCN, 315 token, 2 PARAM, 32 length, 0 ND

### BH-857556D0A10D53F4 — lizard / LIZARD_CCN

- Location: `src/server/renderer/client.js:212:?`
- Severity: `warning`
- Signal: `lizard:LIZARD_CCN`
- Fingerprint: `857556d0a10d53f4`

applyOps has 31 NLOC, 18 CCN, 456 token, 3 PARAM, 35 length, 0 ND

### BH-8D59C9E0CB94EC5A — lizard / LIZARD_CCN

- Location: `src/server/renderer/client.js:267:?`
- Severity: `warning`
- Signal: `lizard:LIZARD_CCN`
- Fingerprint: `8d59c9e0cb94ec5a`

blitRegion has 5 NLOC, 2 CCN, 61 token, 9 PARAM, 5 length, 0 ND

### BH-D61193B1CADB1F8D — lizard / LIZARD_CCN

- Location: `src/server/renderer/client.js:286:?`
- Severity: `warning`
- Signal: `lizard:LIZARD_CCN`
- Fingerprint: `d61193b1cadb1f8d`

parseMessage has 68 NLOC, 18 CCN, 715 token, 1 PARAM, 68 length, 0 ND

### BH-FED69D0D537CCE8A — lizard / LIZARD_CCN

- Location: `src/server/renderer/web.rs:456:?`
- Severity: `warning`
- Signal: `lizard:LIZARD_CCN`
- Fingerprint: `fed69d0d537cce8a`

read_ws_frame has 39 NLOC, 12 CCN, 302 token, 1 PARAM, 39 length, 0 ND

### BH-DBC7B4C3909800A3 — lizard / LIZARD_CCN

- Location: `src/telemetry/mod.rs:160:?`
- Severity: `warning`
- Signal: `lizard:LIZARD_CCN`
- Fingerprint: `dbc7b4c3909800a3`

render_prometheus has 161 NLOC, 1 CCN, 603 token, 2 PARAM, 170 length, 0 ND

### BH-0D65D4FE005D8915 — hadolint / DL3066

- Location: `Dockerfile:21:1`
- Severity: `note`
- Signal: `hadolint:DL3066`
- Fingerprint: `0d65d4fe005d8915`

Non-numeric user-id may not be resolvable by host system

### BH-9869B909107784CB — shellcheck / SC1009

- Location: `deploy/relay/provision-oracle.sh:182:15`
- Severity: `note`
- Signal: `shellcheck:SC1009`
- Fingerprint: `9869b909107784cb`

The mentioned syntax error was in this double quoted string.

### BH-9BF8BD064FC3605A — shellcheck / SC2086

- Location: `scripts/smoke.sh:182:23`
- Severity: `note`
- Signal: `shellcheck:SC2086`
- Fingerprint: `9bf8bd064fc3605a`

Double quote to prevent globbing and word splitting.

## Accepted debt

_No accepted debt recorded in debt.toml._

## Execution failures / unavailable defenses

- **mutmut** — SKIPPED: explicitly skipped by user
- **schemathesis** — SKIPPED: no safe runnable API target discovered/configured
- **custom** — SKIPPED: no high-confidence repository-specific semantic campaign could be inferred
- **tsc** — SKIPPED: TypeScript detected but no root tsconfig.json project exists
- **knip** — SKIPPED: no root package.json: knip requires a project manifest at the scan root
- **publint** — SKIPPED: publint is installed but package.json is missing name/version; add both fields to make the package publishable

## Raw output

### complexity

```text
[]
```

### lizard

```text
src/app/view.rs:185: warning: receive_once has 177 NLOC, 33 CCN, 1139 token, 3 PARAM, 204 length, 0 ND
src/encoder/mod.rs:81: warning: decode_snapshot has 45 NLOC, 14 CCN, 277 token, 4 PARAM, 49 length, 0 ND
src/network/wire.rs:129: warning: validate has 47 NLOC, 18 CCN, 281 token, 3 PARAM, 49 length, 0 ND
src/app/share.rs:148: warning: run_share has 137 NLOC, 23 CCN, 905 token, 2 PARAM, 170 length, 0 ND
src/app/share.rs:319: warning: start_web has 56 NLOC, 15 CCN, 387 token, 7 PARAM, 60 length, 0 ND
src/app/share.rs:490: warning: serve_viewer has 151 NLOC, 17 CCN, 980 token, 8 PARAM, 168 length, 0 ND
src/app/share.rs:761: warning: capture_loop has 184 NLOC, 36 CCN, 1406 token, 12 PARAM, 224 length, 0 ND
src/network/protocol.rs:185: warning: encode_body has 82 NLOC, 3 CCN, 647 token, 2 PARAM, 82 length, 0 ND
src/network/protocol.rs:413: warning: message has 105 NLOC, 43 CCN, 715 token, 1 PARAM, 108 length, 0 ND
src/network/protocol.rs:522: warning: op has 56 NLOC, 15 CCN, 309 token, 1 PARAM, 59 length, 0 ND
src/server/renderer/web.rs:456: warning: read_ws_frame has 39 NLOC, 12 CCN, 302 token, 1 PARAM, 39 length, 0 ND
src/audio/playout.rs:116: warning: start_default_output has 38 NLOC, 15 CCN, 264 token, 1 PARAM, 38 length, 0 ND
src/pcc/types.rs:203: warning: capture_frame has 20 NLOC, 1 CCN, 48 token, 1 PARAM, 94 length, 0 ND
src/server/renderer/client.js:79: warning: lz4Decode has 31 NLOC, 14 CCN, 315 token, 2 PARAM, 32 length, 0 ND
src/server/renderer/client.js:212: warning: applyOps has 31 NLOC, 18 CCN, 456 token, 3 PARAM, 35 length, 0 ND
src/server/renderer/client.js:267: warning: blitRegion has 5 NLOC, 2 CCN, 61 token, 9 PARAM, 5 length, 0 ND
src/server/renderer/client.js:286: warning: parseMessage has 68 NLOC, 18 CCN, 715 token, 1 PARAM, 68 length, 0 ND
src/telemetry/mod.rs:160: warning: render_prometheus has 161 NLOC, 1 CCN, 603 token, 2 PARAM, 170 length, 0 ND
src/pcc/detector.rs:147: warning: detect has 119 NLOC, 28 CCN, 902 token, 5 PARAM, 139 length, 0 ND
src/pcc/detector.rs:465: warning: verify_displacement_of has 34 NLOC, 11 CCN, 274 token, 10 PARAM, 36 length, 0 ND
src/pcc/compositor.rs:212: warning: apply_ops has 77 NLOC, 12 CCN, 573 token, 4 PARAM, 88 length, 0 ND
src/pcc/compositor.rs:379: warning: blit_region has 18 NLOC, 2 CCN, 139 token, 9 PARAM, 18 length, 0 ND
src/pcc/planner.rs:132: warning: plan has 63 NLOC, 12 CCN, 416 token, 6 PARAM, 77 length, 0 ND
src/pcc/planner.rs:289: warning: check_shift has 51 NLOC, 12 CCN, 414 token, 10 PARAM, 60 length, 0 ND
src/pcc/planner.rs:365: warning: subtract_region has 44 NLOC, 15 CCN, 358 token, 5 PARAM, 51 length, 0 ND
src/main.rs:184: warning: main has 130 NLOC, 24 CCN, 803 token, 0 PARAM, 138 length, 0 ND
src/reach/mod.rs:138: warning: classify has 24 NLOC, 17 CCN, 282 token, 1 PARAM, 27 length, 0 ND
src/reach/mod.rs:327: warning: diagnose has 65 NLOC, 12 CCN, 334 token, 1 PARAM, 77 length, 0 ND
src/relay.rs:119: warning: connect has 45 NLOC, 14 CCN, 311 token, 6 PARAM, 51 length, 0 ND
src/relay.rs:293: warning: handle_client has 147 NLOC, 27 CCN, 981 token, 8 PARAM, 174 length, 0 ND
```

### semgrep

```text
{"version":"1.177.0","results":[],"errors":[],"paths":{"scanned":["src/app/mod.rs","src/app/share.rs","src/app/view.rs","src/audio/capture.rs","src/audio/codec.rs","src/audio/mod.rs","src/audio/playout.rs","src/audio/sync.rs","src/capture/mod.rs","src/codec/mod.rs","src/encoder/mod.rs","src/lib.rs","src/main.rs","src/network/config.rs","src/network/e2e.rs","src/network/mod.rs","src/network/protocol.rs","src/network/resilience.rs","src/network/transport.rs","src/network/wire.rs","src/pcc/compositor.rs","src/pcc/detector.rs","src/pcc/mod.rs","src/pcc/planner.rs","src/pcc/types.rs","src/reach/mod.rs","src/relay.rs","src/server/mod.rs","src/server/renderer/buffer.rs","src/server/renderer/client.js","src/server/renderer/mod.rs","src/server/renderer/web.rs","src/telemetry/logging.rs","src/telemetry/metrics.rs","src/telemetry/mod.rs"]},"time":{"rules":[],"rules_parse_time":0.3504960536956787,"profiling_times":{"config_time":2.3142249584198,"core_time":3.2583439350128174,"ignores_time":0.0011789798736572266,"total_time":5.582930088043213},"parsing_time":{"total_time":0.0,"per_file_time":{"mean":0.0,"std_dev":0.0},"very_slow_stats":{"time_ratio":0.0,"count_ratio":0.0},"very_slow_files":[]},"scanning_time":{"total_time":0.5176820755004883,"per_file_time":{"mean":0.004930305480957031,"std_dev":0.0007494768160794204},"very_slow_stats":{"time_ratio":0.0,"count_ratio":0.0},"very_slow_files":[]},"matching_time":{"total_time":0.0,"per_file_and_rule_time":{"mean":0.0,"std_dev":0.0},"very_slow_stats":{"time_ratio":0.0,"count_ratio":0.0},"very_slow_rules_on_files":[]},"tainting_time":{"total_time":0.0,"per_def_and_rule_time":{"mean":0.0,"std_dev":0.0},"very_slow_stats":{"time_ratio":0.0,"count_ratio":0.0},"very_slow_rules_on_defs":[]},"fixpoint_timeouts":[],"prefiltering":{"project_level_time":0.0,"file_level_time":0.0,"rules_with_project_prefilters_ratio":0.0,"rules_with_file_prefilters_ratio":0.9989658738366081,"rules_selected_ratio":0.013960703205791106,"rules_matched_ratio":0.013960703205791106},"targets":[],"total_bytes":0,"max_memory_bytes":827292864},"engine_requested":"OSS","skipped_rules":[],"profiling_results":[]}

[stderr]
Rule bughunt.configs.semgrep.rules.bughunt.assertion-free-test contains an include pattern 'tests/**' that will soon be interpreted as '/tests/**' to comply with the Semgrepignore v2 and Gitignore specifications. To make this pattern permanently unanchored, edit rule bughunt.configs.semgrep.rules.bughunt.assertion-free-test and change it to '**/tests/**'. To confirm the anchored behavior and avoid this warning, change it to '/tests/**'.
               
               
┌─────────────┐
│ Scan Status │
└─────────────┘
  Scanning 35 files tracked by git with 1087 Code rules:
                                                                                                                        
  Language      Rules   Files          Origin      Rules                                                                
 ─────────────────────────────        ───────────────────                                                               
  <multilang>      47      35          Community    1074                                                                
  rust              4      34          Custom         13                                                                
  js              153       1                                                                                           
                                                                                                                        
Rule bughunt.configs.semgrep.rules.bughunt.assertion-free-test contains an include pattern 'tests/**' that will soon be interpreted as '/tests/**' to comply with the Semgrepignore v2 and Gitignore specifications. To make this pattern permanently unanchored, edit rule bughunt.configs.semgrep.rules.bughunt.assertion-free-test and change it to '**/tests/**'. To confirm the anchored behavior and avoid this warning, change it to '/tests/**'.
                
                
┌──────────────┐
│ Scan Summary │
└──────────────┘
✅ Scan completed successfully.
 • Findings: 0 (0 blocking)
 • Rules run: 203
 • Targets scanned: 35
 • Parsed lines: ~100.0%
 • Scan skipped: 
   ◦ Files matching .semgrepignore patterns: 2
 • Scan was limited to files tracked by git
 • For a detailed list of skipped files and lines, run semgrep with the --verbose flag
Ran 203 rules on 35 files: 0 findings.
(need more rules? `semgrep login` for additional free Semgrep Registry rules)
```

### system-ir

```text
{"findings": [{"tool": "scc", "code": "BHGRAPH001", "message": "scc index failed (exit 1): error: index: store: sqlite: database is locked\n", "severity": "error"}], "cache": null}
```

### verify-gaps

```text
{"findings": []}
```

### protocol

```text
{"findings": []}
```

### data

```text
{"findings": []}
```

### actionlint

```text
[{"message":"label \"macos-13\" is unknown. available labels are \"windows-latest\", \"windows-latest-8-cores\", \"windows-2025\", \"windows-2025-vs2026\", \"windows-2022\", \"windows-11-arm\", \"ubuntu-slim\", \"ubuntu-latest\", \"ubuntu-latest-4-cores\", \"ubuntu-latest-8-cores\", \"ubuntu-latest-16-cores\", \"ubuntu-24.04\", \"ubuntu-24.04-arm\", \"ubuntu-22.04\", \"ubuntu-22.04-arm\", \"macos-latest\", \"macos-latest-xlarge\", \"macos-latest-large\", \"macos-26-intel\", \"macos-26-xlarge\", \"macos-26-large\", \"macos-26\", \"macos-15-intel\", \"macos-15-xlarge\", \"macos-15-large\", \"macos-15\", \"macos-14-xlarge\", \"macos-14-large\", \"macos-14\", \"self-hosted\", \"x64\", \"arm\", \"arm64\", \"linux\", \"macos\", \"windows\". if it is a custom label for self-hosted runner, set list of labels in actionlint.yaml config file","filepath":".github/workflows/release.yml","line":37,"column":43,"kind":"runner-label","snippet":"        os: [ubuntu-latest, macos-latest, macos-13, windows-latest]\n                                          ^~~~~~~~~","end_column":51}]
```

### shellcheck

```text
{"comments":[{"file":"scripts/smoke.sh","line":182,"endLine":182,"column":23,"endColumn":27,"level":"info","code":2086,"message":"Double quote to prevent globbing and word splitting.","fix":{"replacements":[{"column":23,"endColumn":23,"endLine":182,"insertionPoint":"afterEnd","line":182,"precedence":13,"replacement":"\""},{"column":27,"endColumn":27,"endLine":182,"insertionPoint":"beforeStart","line":182,"precedence":13,"replacement":"\""}]}},{"file":"deploy/relay/provision-oracle.sh","line":182,"endLine":182,"column":15,"endColumn":15,"level":"info","code":1009,"message":"The mentioned syntax error was in this double quoted string.","fix":null},{"file":"deploy/relay/provision-oracle.sh","line":182,"endLine":182,"column":72,"endColumn":72,"level":"error","code":1073,"message":"Couldn't parse this command expansion. Fix to allow more checks.","fix":null},{"file":"deploy/relay/provision-oracle.sh","line":182,"endLine":182,"column":94,"endColumn":94,"level":"error","code":1036,"message":"'(' is invalid here. Did you forget to escape it?","fix":null},{"file":"deploy/relay/provision-oracle.sh","line":182,"endLine":182,"column":94,"endColumn":94,"level":"error","code":1072,"message":"Expected end of $(..) expression. Fix any mentioned problems and try again.","fix":null}]}
```

### dotenv-linter

```text
Nothing to check
```

### hadolint

```text
[{"code":"DL3008","column":1,"file":"Dockerfile","level":"warning","line":5,"message":"Pin versions in apt get install. Instead of `apt-get install <package>` use `apt-get install <package>=<version>`"},{"code":"DL3008","column":1,"file":"Dockerfile","level":"warning","line":15,"message":"Pin versions in apt get install. Instead of `apt-get install <package>` use `apt-get install <package>=<version>`"},{"code":"DL3066","column":1,"file":"Dockerfile","level":"info","line":21,"message":"Non-numeric user-id may not be resolvable by host system"},{"code":"DL3025","column":1,"file":"Dockerfile","level":"warning","line":23,"message":"Use arguments JSON notation for CMD and ENTRYPOINT arguments"}]
```

### clippy

```text
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#proc-macro2@1.0.92","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proc-macro2-1.0.92/Cargo.toml","target":{"kind":["custom-build"],"crate_types":["bin"],"name":"build-script-build","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proc-macro2-1.0.92/build.rs","edition":"2021","doc":false,"doctest":false,"test":false},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","proc-macro"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/build/proc-macro2-15ce45f581a27756/build-script-build"],"executable":null,"fresh":true}
{"reason":"build-script-executed","package_id":"registry+https://github.com/rust-lang/crates.io-index#proc-macro2@1.0.92","linked_libs":[],"linked_paths":[],"cfgs":["wrap_proc_macro"],"env":[],"out_dir":"/Users/rocket/pixelchangecheck/target/debug/build/proc-macro2-b42367ef94f1d90d/out"}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#unicode-ident@1.0.14","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/unicode-ident-1.0.14/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"unicode_ident","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/unicode-ident-1.0.14/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libunicode_ident-fb58f01d54fd5390.rlib","/Users/rocket/pixelchangecheck/target/debug/deps/libunicode_ident-fb58f01d54fd5390.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#libc@0.2.186","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/libc-0.2.186/Cargo.toml","target":{"kind":["custom-build"],"crate_types":["bin"],"name":"build-script-build","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/libc-0.2.186/build.rs","edition":"2021","doc":false,"doctest":false,"test":false},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/build/libc-a445ca78c07af8ee/build-script-build"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#cfg-if@1.0.0","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cfg-if-1.0.0/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"cfg_if","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cfg-if-1.0.0/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libcfg_if-522e6c4c2751c32f.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#find-msvc-tools@0.1.14","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/find-msvc-tools-0.1.14/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"find_msvc_tools","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/find-msvc-tools-0.1.14/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libfind_msvc_tools-1b291442e17b56ce.rlib","/Users/rocket/pixelchangecheck/target/debug/deps/libfind_msvc_tools-1b291442e17b56ce.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#shlex@2.0.1","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/shlex-2.0.1/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"shlex","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/shlex-2.0.1/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libshlex-e4955143f82da2f9.rlib","/Users/rocket/pixelchangecheck/target/debug/deps/libshlex-e4955143f82da2f9.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#autocfg@1.4.0","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/autocfg-1.4.0/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"autocfg","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/autocfg-1.4.0/src/lib.rs","edition":"2015","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libautocfg-4673a4a72392d896.rlib","/Users/rocket/pixelchangecheck/target/debug/deps/libautocfg-4673a4a72392d896.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#version_check@0.9.5","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/version_check-0.9.5/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"version_check","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/version_check-0.9.5/src/lib.rs","edition":"2015","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libversion_check-d1ca79b948c773a3.rlib","/Users/rocket/pixelchangecheck/target/debug/deps/libversion_check-d1ca79b948c773a3.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#crossbeam-utils@0.8.21","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-utils-0.8.21/Cargo.toml","target":{"kind":["custom-build"],"crate_types":["bin"],"name":"build-script-build","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-utils-0.8.21/build.rs","edition":"2021","doc":false,"doctest":false,"test":false},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/build/crossbeam-utils-03adc6c2bd320317/build-script-build"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#core-foundation-sys@0.8.7","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/core-foundation-sys-0.8.7/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"core_foundation_sys","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/core-foundation-sys-0.8.7/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","link"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libcore_foundation_sys-a64d5445025e5aac.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#proc-macro2@1.0.92","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proc-macro2-1.0.92/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"proc_macro2","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proc-macro2-1.0.92/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","proc-macro"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libproc_macro2-901a6ed899ac0b03.rlib","/Users/rocket/pixelchangecheck/target/debug/deps/libproc_macro2-901a6ed899ac0b03.rmeta"],"executable":null,"fresh":true}
{"reason":"build-script-executed","package_id":"registry+https://github.com/rust-lang/crates.io-index#libc@0.2.186","linked_libs":[],"linked_paths":[],"cfgs":["freebsd12"],"env":[],"out_dir":"/Users/rocket/pixelchangecheck/target/debug/build/libc-a8cfcc9aff5b357b/out"}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#cc@1.5.1","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cc-1.5.1/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"cc","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cc-1.5.1/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libcc-2450543a3564f788.rlib","/Users/rocket/pixelchangecheck/target/debug/deps/libcc-2450543a3564f788.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#generic-array@0.14.7","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/generic-array-0.14.7/Cargo.toml","target":{"kind":["custom-build"],"crate_types":["bin"],"name":"build-script-build","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/generic-array-0.14.7/build.rs","edition":"2015","doc":false,"doctest":false,"test":false},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["more_lengths"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/build/generic-array-e613920282a69e4a/build-script-build"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#typenum@1.20.1","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.1/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"typenum","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.1/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libtypenum-65a0e7cef1fc3346.rmeta"],"executable":null,"fresh":true}
{"reason":"build-script-executed","package_id":"registry+https://github.com/rust-lang/crates.io-index#crossbeam-utils@0.8.21","linked_libs":[],"linked_paths":[],"cfgs":[],"env":[],"out_dir":"/Users/rocket/pixelchangecheck/target/debug/build/crossbeam-utils-daeb49ec0a45de92/out"}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#smallvec@1.13.2","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/smallvec-1.13.2/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"smallvec","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/smallvec-1.13.2/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libsmallvec-0d9971779955fad0.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#log@0.4.22","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/log-0.4.22/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"log","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/log-0.4.22/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/liblog-5a4c00420b17a862.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#once_cell@1.20.2","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/once_cell-1.20.2/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"once_cell","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/once_cell-1.20.2/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["alloc","default","race","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libonce_cell-5002cb66a1801487.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#glob@0.3.4","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/glob-0.3.4/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"glob","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/glob-0.3.4/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libglob-ee45f17193554f3c.rlib","/Users/rocket/pixelchangecheck/target/debug/deps/libglob-ee45f17193554f3c.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#quote@1.0.38","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/quote-1.0.38/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"quote","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/quote-1.0.38/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","proc-macro"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libquote-d29a2748d7016929.rlib","/Users/rocket/pixelchangecheck/target/debug/deps/libquote-d29a2748d7016929.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#libc@0.2.186","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/libc-0.2.186/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"libc","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/libc-0.2.186/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/liblibc-e9ce85694272feae.rmeta"],"executable":null,"fresh":true}
{"reason":"build-script-executed","package_id":"registry+https://github.com/rust-lang/crates.io-index#generic-array@0.14.7","linked_libs":[],"linked_paths":[],"cfgs":["relaxed_coherence"],"env":[],"out_dir":"/Users/rocket/pixelchangecheck/target/debug/build/generic-array-d8c87df9dfa136fb/out"}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#crossbeam-utils@0.8.21","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-utils-0.8.21/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"crossbeam_utils","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-utils-0.8.21/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libcrossbeam_utils-722cb6f118e7fc46.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#ring@0.17.8","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.17.8/Cargo.toml","target":{"kind":["custom-build"],"crate_types":["bin"],"name":"build-script-build","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.17.8/build.rs","edition":"2021","doc":false,"doctest":false,"test":false},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["alloc","default","dev_urandom_fallback"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/build/ring-7a5845410c48eb7d/build-script-build"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#clang-sys@1.9.1","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/clang-sys-1.9.1/Cargo.toml","target":{"kind":["custom-build"],"crate_types":["bin"],"name":"build-script-build","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/clang-sys-1.9.1/build.rs","edition":"2021","doc":false,"doctest":false,"test":false},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["clang_10_0","clang_11_0","clang_3_5","clang_3_6","clang_3_7","clang_3_8","clang_3_9","clang_4_0","clang_5_0","clang_6_0","clang_7_0","clang_8_0","clang_9_0","libloading","runtime"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/build/clang-sys-39cee6ca8eb2a026/build-script-build"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#bitflags@1.3.2","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bitflags-1.3.2/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"bitflags","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bitflags-1.3.2/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libbitflags-ca47bf931fa93763.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#simd-adler32@0.3.7","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simd-adler32-0.3.7/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"simd_adler32","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simd-adler32-0.3.7/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["const-generics","default","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libsimd_adler32-ea32764e63d64a1b.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#libc@0.2.186","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/libc-0.2.186/Cargo.toml","target":{"kind":["custom-build"],"crate_types":["bin"],"name":"build-script-build","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/libc-0.2.186/build.rs","edition":"2021","doc":false,"doctest":false,"test":false},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/build/libc-e0b7743df4b0d95c/build-script-build"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#tracing-core@0.1.33","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tracing-core-0.1.33/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"tracing_core","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tracing-core-0.1.33/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","once_cell","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libtracing_core-b9047212b2dec1ab.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#syn@2.0.93","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/syn-2.0.93/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"syn","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/syn-2.0.93/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["clone-impls","default","derive","extra-traits","full","parsing","printing","proc-macro","visit","visit-mut"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libsyn-3bcc4a3713080b9c.rlib","/Users/rocket/pixelchangecheck/target/debug/deps/libsyn-3bcc4a3713080b9c.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#getrandom@0.2.15","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/getrandom-0.2.15/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"getrandom","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/getrandom-0.2.15/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libgetrandom-e8f27e767602866b.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#generic-array@0.14.7","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/generic-array-0.14.7/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"generic_array","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/generic-array-0.14.7/src/lib.rs","edition":"2015","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["more_lengths"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libgeneric_array-7dafb9bc1d017a68.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#core-foundation@0.9.4","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/core-foundation-0.9.4/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"core_foundation","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/core-foundation-0.9.4/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","link"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libcore_foundation-cff6088e840fbf4a.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#syn@3.0.6","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/syn-3.0.6/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"syn","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/syn-3.0.6/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["clone-impls","default","derive","parsing","printing","proc-macro"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libsyn-b1625a1fd8f89d0d.rlib","/Users/rocket/pixelchangecheck/target/debug/deps/libsyn-b1625a1fd8f89d0d.rmeta"],"executable":null,"fresh":true}
{"reason":"build-script-executed","package_id":"registry+https://github.com/rust-lang/crates.io-index#ring@0.17.8","linked_libs":["static=ring_core_0_17_8_","static=ring_core_0_17_8_test"],"linked_paths":["native=/Users/rocket/pixelchangecheck/target/debug/build/ring-3b3ca46dabdb9067/out"],"cfgs":[],"env":[["RING_CORE_PREFIX","ring_core_0_17_8_"]],"out_dir":"/Users/rocket/pixelchangecheck/target/debug/build/ring-3b3ca46dabdb9067/out"}
{"reason":"build-script-executed","package_id":"registry+https://github.com/rust-lang/crates.io-index#libc@0.2.186","linked_libs":[],"linked_paths":[],"cfgs":["freebsd12"],"env":[],"out_dir":"/Users/rocket/pixelchangecheck/target/debug/build/libc-47d7c1147c2a03e2/out"}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#crossbeam-epoch@0.9.18","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"crossbeam_epoch","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["alloc","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libcrossbeam_epoch-753cb7b93bc872e5.rmeta"],"executable":null,"fresh":true}
{"reason":"build-script-executed","package_id":"registry+https://github.com/rust-lang/crates.io-index#clang-sys@1.9.1","linked_libs":[],"linked_paths":[],"cfgs":[],"env":[],"out_dir":"/Users/rocket/pixelchangecheck/target/debug/build/clang-sys-751308a6e2a7c1cd/out"}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#memchr@2.7.4","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/memchr-2.7.4/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"memchr","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/memchr-2.7.4/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["alloc","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libmemchr-7957f86865f51734.rlib","/Users/rocket/pixelchangecheck/target/debug/deps/libmemchr-7957f86865f51734.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#rand_core@0.6.4","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rand_core-0.6.4/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"rand_core","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rand_core-0.6.4/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["alloc","getrandom","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/librand_core-2ac9733e56492050.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#regex-syntax@0.8.11","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/regex-syntax-0.8.11/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"regex_syntax","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/regex-syntax-0.8.11/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["std","unicode-perl"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libregex_syntax-f926544c54d5ce4a.rlib","/Users/rocket/pixelchangecheck/target/debug/deps/libregex_syntax-f926544c54d5ce4a.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#serde_core@1.0.229","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/serde_core-1.0.229/Cargo.toml","target":{"kind":["custom-build"],"crate_types":["bin"],"name":"build-script-build","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/serde_core-1.0.229/build.rs","edition":"2021","doc":false,"doctest":false,"test":false},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["result","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/build/serde_core-22f6894ec9fde823/build-script-build"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#spin@0.9.8","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/spin-0.9.8/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"spin","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/spin-0.9.8/src/lib.rs","edition":"2015","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["once"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libspin-fe268ab952f7362d.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#minimal-lexical@0.2.1","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/minimal-lexical-0.2.1/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"minimal_lexical","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/minimal-lexical-0.2.1/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libminimal_lexical-f15c3a0c15facee9.rlib","/Users/rocket/pixelchangecheck/target/debug/deps/libminimal_lexical-f15c3a0c15facee9.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#byteorder@1.5.0","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/byteorder-1.5.0/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"byteorder","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/byteorder-1.5.0/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libbyteorder-b183daee5b38300a.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#rayon-core@1.12.1","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rayon-core-1.12.1/Cargo.toml","target":{"kind":["custom-build"],"crate_types":["bin"],"name":"build-script-build","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rayon-core-1.12.1/build.rs","edition":"2021","doc":false,"doctest":false,"test":false},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/build/rayon-core-a47601c25fb47e6b/build-script-build"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#cfg-if@1.0.0","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cfg-if-1.0.0/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"cfg_if","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cfg-if-1.0.0/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libcfg_if-a6d1cd5f0d3faa5c.rlib","/Users/rocket/pixelchangecheck/target/debug/deps/libcfg_if-a6d1cd5f0d3faa5c.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#untrusted@0.9.0","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/untrusted-0.9.0/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"untrusted","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/untrusted-0.9.0/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libuntrusted-b1f1525daf5454cc.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#pin-project-lite@0.2.15","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/pin-project-lite-0.2.15/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"pin_project_lite","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/pin-project-lite-0.2.15/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libpin_project_lite-dde9d104e80f1b27.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#crypto-common@0.1.7","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crypto-common-0.1.7/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"crypto_common","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crypto-common-0.1.7/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["getrandom","rand_core","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libcrypto_common-968187ce52624360.rmeta"],"executable":null,"fresh":true}
{"reason":"build-script-executed","package_id":"registry+https://github.com/rust-lang/crates.io-index#serde_core@1.0.229","linked_libs":[],"linked_paths":[],"cfgs":[],"env":[],"out_dir":"/Users/rocket/pixelchangecheck/target/debug/build/serde_core-c91095b0cb067745/out"}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#libloading@0.8.6","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/libloading-0.8.6/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"libloading","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/libloading-0.8.6/src/lib.rs","edition":"2015","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/liblibloading-031118584f16e02f.rlib","/Users/rocket/pixelchangecheck/target/debug/deps/liblibloading-031118584f16e02f.rmeta"],"executable":null,"fresh":true}
{"reason":"build-script-executed","package_id":"registry+https://github.com/rust-lang/crates.io-index#rayon-core@1.12.1","linked_libs":[],"linked_paths":[],"cfgs":[],"env":[],"out_dir":"/Users/rocket/pixelchangecheck/target/debug/build/rayon-core-5a30b4098ed5106f/out"}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#regex-automata@0.4.18","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/regex-automata-0.4.18/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"regex_automata","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/regex-automata-0.4.18/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["alloc","meta","nfa-pikevm","nfa-thompson","std","syntax","unicode-perl","unicode-word-boundary"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libregex_automata-f9648988da3bef45.rlib","/Users/rocket/pixelchangecheck/target/debug/deps/libregex_automata-f9648988da3bef45.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#nom@7.1.3","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nom-7.1.3/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"nom","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nom-7.1.3/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["alloc","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libnom-64d6cadf78d8bf46.rlib","/Users/rocket/pixelchangecheck/target/debug/deps/libnom-64d6cadf78d8bf46.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#ring@0.17.8","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.17.8/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"ring","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.17.8/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["alloc","default","dev_urandom_fallback"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libring-3058dcf0d041aeb2.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#crossbeam-deque@0.8.6","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-deque-0.8.6/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"crossbeam_deque","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-deque-0.8.6/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libcrossbeam_deque-7ba2792b921826ec.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#libc@0.2.186","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/libc-0.2.186/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"libc","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/libc-0.2.186/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/liblibc-5a302a93a45055ce.rlib","/Users/rocket/pixelchangecheck/target/debug/deps/liblibc-5a302a93a45055ce.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#serde@1.0.229","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/serde-1.0.229/Cargo.toml","target":{"kind":["custom-build"],"crate_types":["bin"],"name":"build-script-build","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/serde-1.0.229/build.rs","edition":"2021","doc":false,"doctest":false,"test":false},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","derive","serde_derive","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/build/serde-87834b964f3d557b/build-script-build"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#bindgen@0.72.1","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bindgen-0.72.1/Cargo.toml","target":{"kind":["custom-build"],"crate_types":["bin"],"name":"build-script-build","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bindgen-0.72.1/build.rs","edition":"2021","doc":false,"doctest":false,"test":false},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["runtime"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/build/bindgen-9b875c2e864de327/build-script-build"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#either@1.13.0","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.13.0/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"either","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.13.0/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libeither-970fccacf32023a6.rlib","/Users/rocket/pixelchangecheck/target/debug/deps/libeither-970fccacf32023a6.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#adler2@2.0.0","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/adler2-2.0.0/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"adler2","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/adler2-2.0.0/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libadler2-bbee5f012e6c9f0c.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#regex@1.13.1","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/regex-1.13.1/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"regex","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/regex-1.13.1/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["std","unicode-perl"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libregex-10b4b22d88f21df3.rlib","/Users/rocket/pixelchangecheck/target/debug/deps/libregex-10b4b22d88f21df3.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#clang-sys@1.9.1","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/clang-sys-1.9.1/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"clang_sys","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/clang-sys-1.9.1/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["clang_10_0","clang_11_0","clang_3_5","clang_3_6","clang_3_7","clang_3_8","clang_3_9","clang_4_0","clang_5_0","clang_6_0","clang_7_0","clang_8_0","clang_9_0","libloading","runtime"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libclang_sys-0d9df5057c7d24db.rlib","/Users/rocket/pixelchangecheck/target/debug/deps/libclang_sys-0d9df5057c7d24db.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#cexpr@0.6.0","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cexpr-0.6.0/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"cexpr","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cexpr-0.6.0/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libcexpr-c9d2e0861cc27c1a.rlib","/Users/rocket/pixelchangecheck/target/debug/deps/libcexpr-c9d2e0861cc27c1a.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#serde_core@1.0.229","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/serde_core-1.0.229/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"serde_core","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/serde_core-1.0.229/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["result","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libserde_core-e812f1b6da41aefc.rmeta"],"executable":null,"fresh":true}
{"reason":"build-script-executed","package_id":"registry+https://github.com/rust-lang/crates.io-index#serde@1.0.229","linked_libs":[],"linked_paths":[],"cfgs":["if_docsrs_then_no_serde_core"],"env":[],"out_dir":"/Users/rocket/pixelchangecheck/target/debug/build/serde-f6bfabb01c64155b/out"}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#rayon-core@1.12.1","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rayon-core-1.12.1/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"rayon_core","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rayon-core-1.12.1/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/librayon_core-533833fe9626de72.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#serde_derive@1.0.229","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/serde_derive-1.0.229/Cargo.toml","target":{"kind":["proc-macro"],"crate_types":["proc-macro"],"name":"serde_derive","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/serde_derive-1.0.229/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libserde_derive-ed2262ec9caa0d24.dylib"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#itertools@0.13.0","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/itertools-0.13.0/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"itertools","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/itertools-0.13.0/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":false},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libitertools-e452f2cb06d2c65e.rlib","/Users/rocket/pixelchangecheck/target/debug/deps/libitertools-e452f2cb06d2c65e.rmeta"],"executable":null,"fresh":true}
{"reason":"build-script-executed","package_id":"registry+https://github.com/rust-lang/crates.io-index#bindgen@0.72.1","linked_libs":[],"linked_paths":[],"cfgs":[],"env":[],"out_dir":"/Users/rocket/pixelchangecheck/target/debug/build/bindgen-3d7ac8005e3a9391/out"}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#miniz_oxide@0.8.2","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/miniz_oxide-0.8.2/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"miniz_oxide","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/miniz_oxide-0.8.2/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","simd","simd-adler32","with-alloc"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libminiz_oxide-41006fe57dd81219.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#tracing-attributes@0.1.28","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tracing-attributes-0.1.28/Cargo.toml","target":{"kind":["proc-macro"],"crate_types":["proc-macro"],"name":"tracing_attributes","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tracing-attributes-0.1.28/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libtracing_attributes-458dbc69bd2fb9f3.dylib"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#zeroize_derive@1.5.0","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/zeroize_derive-1.5.0/Cargo.toml","target":{"kind":["proc-macro"],"crate_types":["proc-macro"],"name":"zeroize_derive","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/zeroize_derive-1.5.0/src/lib.rs","edition":"2024","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libzeroize_derive-980668f69ef6ce6f.dylib"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#zerocopy-derive@0.7.35","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/zerocopy-derive-0.7.35/Cargo.toml","target":{"kind":["proc-macro"],"crate_types":["proc-macro"],"name":"zerocopy_derive","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/zerocopy-derive-0.7.35/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libzerocopy_derive-fcbe7fd050d2d259.dylib"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#lock_api@0.4.12","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/lock_api-0.4.12/Cargo.toml","target":{"kind":["custom-build"],"crate_types":["bin"],"name":"build-script-build","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/lock_api-0.4.12/build.rs","edition":"2021","doc":false,"doctest":false,"test":false},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["atomic_usize","default"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/build/lock_api-8f2f5d3039e637a6/build-script-build"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#bitflags@2.6.0","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bitflags-2.6.0/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"bitflags","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bitflags-2.6.0/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libbitflags-473145bdc36caf2e.rlib","/Users/rocket/pixelchangecheck/target/debug/deps/libbitflags-473145bdc36caf2e.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#itoa@1.0.14","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/itoa-1.0.14/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"itoa","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/itoa-1.0.14/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libitoa-14569a6a67097239.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#parking_lot_core@0.9.10","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/parking_lot_core-0.9.10/Cargo.toml","target":{"kind":["custom-build"],"crate_types":["bin"],"name":"build-script-build","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/parking_lot_core-0.9.10/build.rs","edition":"2021","doc":false,"doctest":false,"test":false},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/build/parking_lot_core-f7b5c26695403ae6/build-script-build"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#rustc-hash@2.1.3","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rustc-hash-2.1.3/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"rustc_hash","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rustc-hash-2.1.3/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/librustc_hash-ee8619da94745cd8.rlib","/Users/rocket/pixelchangecheck/target/debug/deps/librustc_hash-ee8619da94745cd8.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#shlex@1.3.0","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/shlex-1.3.0/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"shlex","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/shlex-1.3.0/src/lib.rs","edition":"2015","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libshlex-ccc2311e163efc2f.rlib","/Users/rocket/pixelchangecheck/target/debug/deps/libshlex-ccc2311e163efc2f.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#tracing@0.1.41","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tracing-0.1.41/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"tracing","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tracing-0.1.41/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["attributes","default","log","std","tracing-attributes"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libtracing-fe2a6efaabb71d20.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#zerocopy@0.7.35","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/zerocopy-0.7.35/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"zerocopy","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/zerocopy-0.7.35/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["byteorder","default","derive","simd","zerocopy-derive"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libzerocopy-c997f05c41681665.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#zeroize@1.9.0","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/zeroize-1.9.0/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"zeroize","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/zeroize-1.9.0/src/lib.rs","edition":"2024","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["alloc","zeroize_derive"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libzeroize-3db17d8d8e913b4c.rmeta"],"executable":null,"fresh":true}
{"reason":"build-script-executed","package_id":"registry+https://github.com/rust-lang/crates.io-index#lock_api@0.4.12","linked_libs":[],"linked_paths":[],"cfgs":["has_const_fn_trait_bound"],"env":[],"out_dir":"/Users/rocket/pixelchangecheck/target/debug/build/lock_api-2eaba2df6582d525/out"}
{"reason":"build-script-executed","package_id":"registry+https://github.com/rust-lang/crates.io-index#parking_lot_core@0.9.10","linked_libs":[],"linked_paths":[],"cfgs":[],"env":[],"out_dir":"/Users/rocket/pixelchangecheck/target/debug/build/parking_lot_core-c7699bee62d30d8f/out"}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#serde@1.0.229","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/serde-1.0.229/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"serde","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/serde-1.0.229/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","derive","serde_derive","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libserde-7af537af539a09fb.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#crc32fast@1.4.2","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crc32fast-1.4.2/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"crc32fast","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crc32fast-1.4.2/src/lib.rs","edition":"2015","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libcrc32fast-2025e5d4092eb47f.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#either@1.13.0","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.13.0/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"either","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.13.0/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libeither-9bec555e6479cae1.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#bindgen@0.72.1","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bindgen-0.72.1/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"bindgen","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bindgen-0.72.1/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["runtime"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libbindgen-b7b992568295d533.rlib","/Users/rocket/pixelchangecheck/target/debug/deps/libbindgen-b7b992568295d533.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#scopeguard@1.2.0","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/scopeguard-1.2.0/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"scopeguard","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/scopeguard-1.2.0/src/lib.rs","edition":"2015","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libscopeguard-80b9b448658b595a.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#subtle@2.6.1","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/subtle-2.6.1/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"subtle","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/subtle-2.6.1/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libsubtle-97e87d188944ab97.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#semver@1.0.28","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/semver-1.0.28/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"semver","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/semver-1.0.28/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libsemver-fd2f8a8e1a0c567a.rlib","/Users/rocket/pixelchangecheck/target/debug/deps/libsemver-fd2f8a8e1a0c567a.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#powerfmt@0.2.0","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/powerfmt-0.2.0/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"powerfmt","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/powerfmt-0.2.0/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libpowerfmt-a3d58aa341a1dbe3.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#rustls@0.21.12","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rustls-0.21.12/Cargo.toml","target":{"kind":["custom-build"],"crate_types":["bin"],"name":"build-script-build","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rustls-0.21.12/build.rs","edition":"2021","doc":false,"doctest":false,"test":false},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["dangerous_configuration","default","log","logging","quic","tls12"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/build/rustls-bc30b2df3c016174/build-script-build"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#bytes@1.9.0","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bytes-1.9.0/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"bytes","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bytes-1.9.0/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libbytes-bf64e1f2b06d8674.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#flate2@1.0.35","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/flate2-1.0.35/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"flate2","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/flate2-1.0.35/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["any_impl","default","miniz_oxide","rust_backend"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libflate2-6b95dafb22e8b54f.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#parking_lot_core@0.9.10","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/parking_lot_core-0.9.10/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"parking_lot_core","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/parking_lot_core-0.9.10/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libparking_lot_core-7eb4e18a3530cc05.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#rayon@1.10.0","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rayon-1.10.0/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"rayon","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rayon-1.10.0/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/librayon-947ce83e85f93da8.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#lock_api@0.4.12","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/lock_api-0.4.12/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"lock_api","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/lock_api-0.4.12/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["atomic_usize","default"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/liblock_api-cc36780bc381b66d.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#coreaudio-sys@0.2.18","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/coreaudio-sys-0.2.18/Cargo.toml","target":{"kind":["custom-build"],"crate_types":["bin"],"name":"build-script-build","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/coreaudio-sys-0.2.18/build.rs","edition":"2024","doc":false,"doctest":false,"test":false},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["audio_unit","core_audio"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/build/coreaudio-sys-72cb5f76578b2f8d/build-script-build"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#deranged@0.3.11","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/deranged-0.3.11/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"deranged","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/deranged-0.3.11/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["alloc","powerfmt","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libderanged-a434a69733634014.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#rustc_version@0.4.1","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rustc_version-0.4.1/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"rustc_version","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rustc_version-0.4.1/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/librustc_version-11b381c310ad5e98.rlib","/Users/rocket/pixelchangecheck/target/debug/deps/librustc_version-11b381c310ad5e98.rmeta"],"executable":null,"fresh":true}
{"reason":"build-script-executed","package_id":"registry+https://github.com/rust-lang/crates.io-index#rustls@0.21.12","linked_libs":[],"linked_paths":[],"cfgs":[],"env":[],"out_dir":"/Users/rocket/pixelchangecheck/target/debug/build/rustls-a27e4f9ea0d4f3d7/out"}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#ppv-lite86@0.2.20","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ppv-lite86-0.2.20/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"ppv_lite86","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ppv-lite86-0.2.20/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["simd","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libppv_lite86-f4ef51c52bf1ecbd.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#sct@0.7.1","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/sct-0.7.1/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"sct","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/sct-0.7.1/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libsct-27b7cb1362ade9b1.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#rustls-webpki@0.101.7","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rustls-webpki-0.101.7/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"webpki","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rustls-webpki-0.101.7/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["alloc","default","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libwebpki-fa74687fc1de8db5.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#core-graphics-types@0.1.3","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/core-graphics-types-0.1.3/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"core_graphics_types","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/core-graphics-types-0.1.3/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","link"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libcore_graphics_types-35bbd1704c89afeb.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#foreign-types-macros@0.2.3","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/foreign-types-macros-0.2.3/Cargo.toml","target":{"kind":["proc-macro"],"crate_types":["proc-macro"],"name":"foreign_types_macros","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/foreign-types-macros-0.2.3/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libforeign_types_macros-6fe43c8c739a432c.dylib"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#socket2@0.5.8","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/socket2-0.5.8/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"socket2","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/socket2-0.5.8/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["all"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libsocket2-d8ce7b87caa5ccc9.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#security-framework-sys@2.13.0","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/security-framework-sys-2.13.0/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"security_framework_sys","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/security-framework-sys-2.13.0/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["OSX_10_10","OSX_10_11","OSX_10_12","OSX_10_9"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libsecurity_framework_sys-530412b633b1daad.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#ring@0.16.20","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.16.20/Cargo.toml","target":{"kind":["custom-build"],"crate_types":["bin"],"name":"build-script-build","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.16.20/build.rs","edition":"2018","doc":false,"doctest":false,"test":false},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["alloc","default","dev_urandom_fallback","once_cell"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/build/ring-271a4f4b8e3e5f39/build-script-build"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#cmake@0.1.58","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cmake-0.1.58/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"cmake","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cmake-0.1.58/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libcmake-dd3d12cf734827a3.rlib","/Users/rocket/pixelchangecheck/target/debug/deps/libcmake-dd3d12cf734827a3.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#slab@0.4.9","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/slab-0.4.9/Cargo.toml","target":{"kind":["custom-build"],"crate_types":["bin"],"name":"build-script-build","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/slab-0.4.9/build.rs","edition":"2018","doc":false,"doctest":false,"test":false},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/build/slab-16be462b44f9bd06/build-script-build"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#num-traits@0.2.19","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/num-traits-0.2.19/Cargo.toml","target":{"kind":["custom-build"],"crate_types":["bin"],"name":"build-script-build","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/num-traits-0.2.19/build.rs","edition":"2021","doc":false,"doctest":false,"test":false},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/build/num-traits-2eb651e8f82cff0a/build-script-build"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#num-conv@0.1.0","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/num-conv-0.1.0/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"num_conv","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/num-conv-0.1.0/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libnum_conv-845f8ed1482cf1df.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#foreign-types-shared@0.3.1","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/foreign-types-shared-0.3.1/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"foreign_types_shared","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/foreign-types-shared-0.3.1/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libforeign_types_shared-dc3cf79e7a248468.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#bitflags@2.6.0","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bitflags-2.6.0/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"bitflags","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bitflags-2.6.0/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libbitflags-9606e0f09105ff66.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#base64@0.21.7","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/base64-0.21.7/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"base64","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/base64-0.21.7/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["alloc","default","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libbase64-05b037d3854f95f8.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#weezl@0.1.8","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/weezl-0.1.8/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"weezl","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/weezl-0.1.8/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["alloc","default","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libweezl-56f3c4a072801dfd.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#anyhow@1.0.95","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/anyhow-1.0.95/Cargo.toml","target":{"kind":["custom-build"],"crate_types":["bin"],"name":"build-script-build","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/anyhow-1.0.95/build.rs","edition":"2018","doc":false,"doctest":false,"test":false},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/build/anyhow-56b8d66c8e257a7e/build-script-build"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#thiserror@1.0.69","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/thiserror-1.0.69/Cargo.toml","target":{"kind":["custom-build"],"crate_types":["bin"],"name":"build-script-build","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/thiserror-1.0.69/build.rs","edition":"2021","doc":false,"doctest":false,"test":false},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/build/thiserror-f51fd2e0f1b4c7bd/build-script-build"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#time-core@0.1.2","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/time-core-0.1.2/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"time_core","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/time-core-0.1.2/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libtime_core-a3ca08278b2331f8.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#utf8parse@0.2.2","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/utf8parse-0.2.2/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"utf8parse","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/utf8parse-0.2.2/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libutf8parse-41c3cd03225d4b1d.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#regex-syntax@0.6.29","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/regex-syntax-0.6.29/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"regex_syntax","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/regex-syntax-0.6.29/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","unicode","unicode-age","unicode-bool","unicode-case","unicode-gencat","unicode-perl","unicode-script","unicode-segment"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libregex_syntax-3e7272f4ed2c40a1.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#regex-syntax@0.8.11","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/regex-syntax-0.8.11/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"regex_syntax","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/regex-syntax-0.8.11/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["std","unicode-case","unicode-perl"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libregex_syntax-7f5a0babb68d6e8f.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#bytemuck@1.21.0","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bytemuck-1.21.0/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"bytemuck","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bytemuck-1.21.0/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["extern_crate_alloc"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libbytemuck-7b5b30d243339af6.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#serde_json@1.0.134","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/serde_json-1.0.134/Cargo.toml","target":{"kind":["custom-build"],"crate_types":["bin"],"name":"build-script-build","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/serde_json-1.0.134/build.rs","edition":"2021","doc":false,"doctest":false,"test":false},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/build/serde_json-2323cae1c8611387/build-script-build"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#opusic-sys@0.7.5","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/opusic-sys-0.7.5/Cargo.toml","target":{"kind":["custom-build"],"crate_types":["bin"],"name":"build-script-build","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/opusic-sys-0.7.5/build.rs","edition":"2021","doc":false,"doctest":false,"test":false},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["bundled","default"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/build/opusic-sys-1aa810c29c04da07/build-script-build"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#time@0.3.37","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/time-0.3.37/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"time","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/time-0.3.37/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["alloc","formatting","parsing","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libtime-72b9b1540b557b5b.rmeta"],"executable":null,"fresh":true}
{"reason":"build-script-executed","package_id":"registry+https://github.com/rust-lang/crates.io-index#thiserror@1.0.69","linked_libs":[],"linked_paths":[],"cfgs":[],"env":[],"out_dir":"/Users/rocket/pixelchangecheck/target/debug/build/thiserror-6d025c3824800db8/out"}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#anstyle-parse@0.2.7","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/anstyle-parse-0.2.7/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"anstyle_parse","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/anstyle-parse-0.2.7/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","utf8"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libanstyle_parse-0a8af253b486928d.rmeta"],"executable":null,"fresh":true}
{"reason":"build-script-executed","package_id":"registry+https://github.com/rust-lang/crates.io-index#serde_json@1.0.134","linked_libs":[],"linked_paths":[],"cfgs":["fast_arithmetic=\"64\""],"env":[],"out_dir":"/Users/rocket/pixelchangecheck/target/debug/build/serde_json-a217d77e841a621b/out"}
{"reason":"build-script-executed","package_id":"registry+https://github.com/rust-lang/crates.io-index#anyhow@1.0.95","linked_libs":[],"linked_paths":[],"cfgs":["std_backtrace"],"env":[],"out_dir":"/Users/rocket/pixelchangecheck/target/debug/build/anyhow-6a5de729723946b0/out"}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#regex-automata@0.1.10","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/regex-automata-0.1.10/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"regex_automata","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/regex-automata-0.1.10/src/lib.rs","edition":"2015","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","regex-syntax","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libregex_automata-4697bff0aa2ebc98.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#regex-automata@0.4.18","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/regex-automata-0.4.18/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"regex_automata","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/regex-automata-0.4.18/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["alloc","meta","nfa-pikevm","nfa-thompson","std","syntax","unicode-case","unicode-perl","unicode-word-boundary"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libregex_automata-1c0d871f448e7d59.rmeta"],"executable":null,"fresh":true}
{"reason":"build-script-executed","package_id":"registry+https://github.com/rust-lang/crates.io-index#slab@0.4.9","linked_libs":[],"linked_paths":[],"cfgs":[],"env":[],"out_dir":"/Users/rocket/pixelchangecheck/target/debug/build/slab-a355ea128e85198e/out"}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#rustls-pemfile@1.0.4","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rustls-pemfile-1.0.4/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"rustls_pemfile","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rustls-pemfile-1.0.4/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/librustls_pemfile-b89e76dc5b8fb361.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#security-framework@2.11.1","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/security-framework-2.11.1/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"security_framework","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/security-framework-2.11.1/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["OSX_10_10","OSX_10_11","OSX_10_12","OSX_10_9","default"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libsecurity_framework-4c2ae2e6dcff0dc0.rmeta"],"executable":null,"fresh":true}
{"reason":"build-script-executed","package_id":"registry+https://github.com/rust-lang/crates.io-index#ring@0.16.20","linked_libs":["static=ring-core","static=ring-test"],"linked_paths":["native=/Users/rocket/pixelchangecheck/target/debug/build/ring-dad938872b39843f/out"],"cfgs":[],"env":[],"out_dir":"/Users/rocket/pixelchangecheck/target/debug/build/ring-dad938872b39843f/out"}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#foreign-types@0.5.0","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/foreign-types-0.5.0/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"foreign_types","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/foreign-types-0.5.0/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libforeign_types-e9a8bbccf25a4643.rmeta"],"executable":null,"fresh":true}
{"reason":"build-script-executed","package_id":"registry+https://github.com/rust-lang/crates.io-index#num-traits@0.2.19","linked_libs":[],"linked_paths":[],"cfgs":["has_total_cmp"],"env":[],"out_dir":"/Users/rocket/pixelchangecheck/target/debug/build/num-traits-9c00f19aff09baaa/out"}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#rand_chacha@0.3.1","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rand_chacha-0.3.1/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"rand_chacha","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rand_chacha-0.3.1/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/librand_chacha-20ed222ee03d0698.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#parking_lot@0.12.3","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/parking_lot-0.12.3/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"parking_lot","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/parking_lot-0.12.3/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libparking_lot-8e25cea1b5590b0f.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#curve25519-dalek@4.1.3","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/curve25519-dalek-4.1.3/Cargo.toml","target":{"kind":["custom-build"],"crate_types":["bin"],"name":"build-script-build","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/curve25519-dalek-4.1.3/build.rs","edition":"2021","doc":false,"doctest":false,"test":false},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["alloc","precomputed-tables","zeroize"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/build/curve25519-dalek-ca4f1b141deae3cc/build-script-build"],"executable":null,"fresh":true}
{"reason":"build-script-executed","package_id":"registry+https://github.com/rust-lang/crates.io-index#coreaudio-sys@0.2.18","linked_libs":["framework=AudioUnit","framework=CoreAudio"],"linked_paths":[],"cfgs":[],"env":[],"out_dir":"/Users/rocket/pixelchangecheck/target/debug/build/coreaudio-sys-40dd1e86d2f01538/out"}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#rustls@0.21.12","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rustls-0.21.12/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"rustls","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rustls-0.21.12/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["dangerous_configuration","default","log","logging","quic","tls12"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/librustls-f2dfc1ff46fb886a.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#jpeg-decoder@0.3.1","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/jpeg-decoder-0.3.1/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"jpeg_decoder","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/jpeg-decoder-0.3.1/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["rayon"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libjpeg_decoder-9d83f9a01ab6630d.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#thiserror-impl@1.0.69","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/thiserror-impl-1.0.69/Cargo.toml","target":{"kind":["proc-macro"],"crate_types":["proc-macro"],"name":"thiserror_impl","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/thiserror-impl-1.0.69/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libthiserror_impl-9605e08f515ebc90.dylib"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#tokio-macros@2.4.0","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-macros-2.4.0/Cargo.toml","target":{"kind":["proc-macro"],"crate_types":["proc-macro"],"name":"tokio_macros","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-macros-2.4.0/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libtokio_macros-a7958050fa14b4a7.dylib"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#block-buffer@0.10.4","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/block-buffer-0.10.4/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"block_buffer","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/block-buffer-0.10.4/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libblock_buffer-25ed9bed9a568746.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#inout@0.1.4","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/inout-0.1.4/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"inout","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/inout-0.1.4/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libinout-98b15d7a3c5b0fc4.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#signal-hook-registry@1.4.2","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/signal-hook-registry-1.4.2/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"signal_hook_registry","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/signal-hook-registry-1.4.2/src/lib.rs","edition":"2015","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libsignal_hook_registry-0a95a3e2941876de.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#zune-inflate@0.2.54","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/zune-inflate-0.2.54/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"zune_inflate","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/zune-inflate-0.2.54/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["simd-adler32","zlib"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libzune_inflate-d08016640a3ea9b8.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#mio@1.0.3","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/mio-1.0.3/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"mio","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/mio-1.0.3/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["net","os-ext","os-poll"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libmio-d40533f18b3f14e2.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#fdeflate@0.3.7","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/fdeflate-0.3.7/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"fdeflate","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/fdeflate-0.3.7/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libfdeflate-9a202b921580991f.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#half@2.4.1","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/half-2.4.1/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"half","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/half-2.4.1/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["alloc","default","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libhalf-551ee2c12281932c.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#bit_field@0.10.2","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bit_field-0.10.2/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"bit_field","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bit_field-0.10.2/src/lib.rs","edition":"2015","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libbit_field-11776b4d1abf8d93.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#overload@0.1.1","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/overload-0.1.1/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"overload","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/overload-0.1.1/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/liboverload-e47d68d57842b571.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#lazy_static@1.5.0","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/lazy_static-1.5.0/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"lazy_static","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/lazy_static-1.5.0/src/lib.rs","edition":"2015","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/liblazy_static-bb5b27e51a17bea3.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#is_terminal_polyfill@1.70.2","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/is_terminal_polyfill-1.70.2/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"is_terminal_polyfill","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/is_terminal_polyfill-1.70.2/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libis_terminal_polyfill-b90b605621929774.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#memchr@2.7.4","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/memchr-2.7.4/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"memchr","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/memchr-2.7.4/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["alloc","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libmemchr-3db9e4608a769aef.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#lebe@0.5.2","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/lebe-0.5.2/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"lebe","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/lebe-0.5.2/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/liblebe-ab312d291e5c5be9.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#ryu@1.0.18","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ryu-1.0.18/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"ryu","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ryu-1.0.18/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libryu-eb402c5d047d693a.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#tinyvec_macros@0.1.1","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tinyvec_macros-0.1.1/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"tinyvec_macros","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tinyvec_macros-0.1.1/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libtinyvec_macros-a5e24f78cbf29044.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#untrusted@0.7.1","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/untrusted-0.7.1/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"untrusted","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/untrusted-0.7.1/src/untrusted.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libuntrusted-7518921e982edd3e.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#colorchoice@1.0.5","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/colorchoice-1.0.5/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"colorchoice","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/colorchoice-1.0.5/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libcolorchoice-b949b477ab41af8e.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#color_quant@1.1.0","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/color_quant-1.1.0/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"color_quant","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/color_quant-1.1.0/src/lib.rs","edition":"2015","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libcolor_quant-b776cdea4b93c7c1.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#anstyle@1.0.14","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/anstyle-1.0.14/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"anstyle","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/anstyle-1.0.14/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libanstyle-20226e04854d305e.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#foreign-types-shared@0.1.1","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/foreign-types-shared-0.1.1/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"foreign_types_shared","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/foreign-types-shared-0.1.1/src/lib.rs","edition":"2015","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libforeign_types_shared-c5c578d83fba7aa9.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#anstyle-query@1.1.5","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/anstyle-query-1.1.5/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"anstyle_query","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/anstyle-query-1.1.5/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libanstyle_query-11117425835bce29.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#thiserror@2.0.21","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/thiserror-2.0.21/Cargo.toml","target":{"kind":["custom-build"],"crate_types":["bin"],"name":"build-script-build","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/thiserror-2.0.21/build.rs","edition":"2021","doc":false,"doctest":false,"test":false},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/build/thiserror-2ee3a361617f0a5d/build-script-build"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#sharded-slab@0.1.7","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/sharded-slab-0.1.7/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"sharded_slab","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/sharded-slab-0.1.7/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libsharded_slab-ad90c5e85848a8cb.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#tinyvec@1.8.1","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tinyvec-1.8.1/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"tinyvec","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tinyvec-1.8.1/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["alloc","default","tinyvec_macros"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libtinyvec-4d55a66076331093.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#gif@0.13.1","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gif-0.13.1/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"gif","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gif-0.13.1/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["color_quant","default","raii_no_panic","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libgif-c4e4eb8666c357c1.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#exr@1.73.0","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/exr-1.73.0/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"exr","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/exr-1.73.0/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libexr-e3304b557a0acd9a.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#ring@0.16.20","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.16.20/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"ring","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.16.20/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["alloc","default","dev_urandom_fallback","once_cell"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libring-41c9884717bbeb86.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#serde_json@1.0.134","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/serde_json-1.0.134/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"serde_json","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/serde_json-1.0.134/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libserde_json-ca9801c1e3feb721.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#nu-ansi-term@0.46.0","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nu-ansi-term-0.46.0/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"nu_ansi_term","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nu-ansi-term-0.46.0/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libnu_ansi_term-5a559a3d2333c6f5.rmeta"],"executable":null,"fresh":true}
{"reason":"build-script-executed","package_id":"registry+https://github.com/rust-lang/crates.io-index#thiserror@2.0.21","linked_libs":[],"linked_paths":[],"cfgs":[],"env":[],"out_dir":"/Users/rocket/pixelchangecheck/target/debug/build/thiserror-b71f5992307e23cb/out"}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#anstream@0.6.21","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/anstream-0.6.21/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"anstream","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/anstream-0.6.21/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["auto","default","wincon"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libanstream-d3d23a5224fd3172.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#foreign-types@0.3.2","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/foreign-types-0.3.2/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"foreign_types","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/foreign-types-0.3.2/src/lib.rs","edition":"2015","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libforeign_types-1efa1043efcfc82d.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#cipher@0.4.4","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cipher-0.4.4/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"cipher","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cipher-0.4.4/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["zeroize"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libcipher-56990dd904c7b422.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#tokio@1.42.0","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.42.0/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"tokio","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.42.0/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["bytes","default","fs","full","io-std","io-util","libc","macros","mio","net","parking_lot","process","rt","rt-multi-thread","signal","signal-hook-registry","socket2","sync","time","tokio-macros"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libtokio-636e4e87bcdafa6c.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#png@0.17.16","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/png-0.17.16/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"png","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/png-0.17.16/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libpng-e560c55191211c91.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#digest@0.10.7","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/digest-0.10.7/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"digest","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/digest-0.10.7/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["alloc","block-buffer","core-api","default","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libdigest-8bccacd2959da3ca.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#core-graphics@0.23.2","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/core-graphics-0.23.2/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"core_graphics","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/core-graphics-0.23.2/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","link"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libcore_graphics-55e40b5af363f18c.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#rand@0.8.5","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rand-0.8.5/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"rand","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rand-0.8.5/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["alloc","default","getrandom","libc","rand_chacha","std","std_rng"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/librand-ff51f6b1718bf47a.rmeta"],"executable":null,"fresh":true}
{"reason":"build-script-executed","package_id":"registry+https://github.com/rust-lang/crates.io-index#curve25519-dalek@4.1.3","linked_libs":[],"linked_paths":[],"cfgs":["curve25519_dalek_bits=\"64\"","curve25519_dalek_backend=\"serial\""],"env":[],"out_dir":"/Users/rocket/pixelchangecheck/target/debug/build/curve25519-dalek-4280c49c59aef0a7/out"}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#num-traits@0.2.19","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/num-traits-0.2.19/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"num_traits","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/num-traits-0.2.19/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libnum_traits-29aeefbf5f7318dc.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#tiff@0.9.1","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tiff-0.9.1/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"tiff","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tiff-0.9.1/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libtiff-5c0a4a39e4d9fb3c.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#coreaudio-sys@0.2.18","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/coreaudio-sys-0.2.18/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"coreaudio_sys","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/coreaudio-sys-0.2.18/src/lib.rs","edition":"2024","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["audio_unit","core_audio"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libcoreaudio_sys-ebe1bf9ab5e781ae.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#thiserror@1.0.69","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/thiserror-1.0.69/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"thiserror","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/thiserror-1.0.69/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libthiserror-432a63671f6eecdf.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#matchers@0.1.0","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/matchers-0.1.0/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"matchers","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/matchers-0.1.0/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libmatchers-a0a8bfe92cf2f2a3.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#anyhow@1.0.95","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/anyhow-1.0.95/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"anyhow","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/anyhow-1.0.95/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libanyhow-63276ce0b109727e.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#rustls-native-certs@0.6.3","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rustls-native-certs-0.6.3/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"rustls_native_certs","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rustls-native-certs-0.6.3/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/librustls_native_certs-945e65ce0b1c13a1.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#regex@1.13.1","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/regex-1.13.1/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"regex","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/regex-1.13.1/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["std","unicode-case","unicode-perl"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libregex-924b3c5b5f8444f5.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#slab@0.4.9","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/slab-0.4.9/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"slab","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/slab-0.4.9/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libslab-b9eb7b6a44b6b68d.rmeta"],"executable":null,"fresh":true}
{"reason":"build-script-executed","package_id":"registry+https://github.com/rust-lang/crates.io-index#opusic-sys@0.7.5","linked_libs":["static=opus"],"linked_paths":["native=/Users/rocket/pixelchangecheck/target/debug/build/opusic-sys-de20299cab2d52c5/out/lib"],"cfgs":[],"env":[],"out_dir":"/Users/rocket/pixelchangecheck/target/debug/build/opusic-sys-de20299cab2d52c5/out"}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#qoi@0.4.1","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/qoi-0.4.1/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"qoi","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/qoi-0.4.1/src/lib.rs","edition":"2021","doc":true,"doctest":false,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libqoi-35aaeebc189525ff.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#universal-hash@0.5.1","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/universal-hash-0.5.1/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"universal_hash","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/universal-hash-0.5.1/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libuniversal_hash-4ea8e1fe0a049b00.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#tracing-serde@0.2.0","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tracing-serde-0.2.0/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"tracing_serde","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tracing-serde-0.2.0/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libtracing_serde-245f688581fa3eaa.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#thiserror-impl@2.0.21","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/thiserror-impl-2.0.21/Cargo.toml","target":{"kind":["proc-macro"],"crate_types":["proc-macro"],"name":"thiserror_impl","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/thiserror-impl-2.0.21/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libthiserror_impl-1526f0eb15f8175a.dylib"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#cpufeatures@0.2.17","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cpufeatures-0.2.17/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"cpufeatures","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cpufeatures-0.2.17/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libcpufeatures-7c9d47d33c5b96a7.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#tracing-log@0.2.0","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tracing-log-0.2.0/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"tracing_log","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tracing-log-0.2.0/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["log-tracer","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libtracing_log-e9bdc7fe251909e6.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#minifb@0.27.0","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/minifb-0.27.0/Cargo.toml","target":{"kind":["custom-build"],"crate_types":["bin"],"name":"build-script-build","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/minifb-0.27.0/build.rs","edition":"2018","doc":false,"doctest":false,"test":false},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","dlib","dlopen","lazy_static","libc","tempfile","wayland","wayland-client","wayland-cursor","wayland-protocols","x11","x11-dl"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/build/minifb-24e8eb726b638030/build-script-build"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#thread_local@1.1.8","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/thread_local-1.1.8/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"thread_local","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/thread_local-1.1.8/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libthread_local-4f4c0c0c1ea6c2c0.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#clap_lex@1.1.0","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/clap_lex-1.1.0/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"clap_lex","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/clap_lex-1.1.0/src/lib.rs","edition":"2024","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libclap_lex-eda5a65c1e9a9f50.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#cpal@0.15.3","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cpal-0.15.3/Cargo.toml","target":{"kind":["custom-build"],"crate_types":["bin"],"name":"build-script-build","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cpal-0.15.3/build.rs","edition":"2021","doc":false,"doctest":false,"test":false},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/build/cpal-e324587592c71c93/build-script-build"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#strsim@0.11.1","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/strsim-0.11.1/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"strsim","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/strsim-0.11.1/src/lib.rs","edition":"2015","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libstrsim-128cb255eb314a1b.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#static_assertions@1.1.0","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/static_assertions-1.1.0/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"static_assertions","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/static_assertions-1.1.0/src/lib.rs","edition":"2015","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libstatic_assertions-9e620e0a2e2255ff.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#heck@0.5.0","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/heck-0.5.0/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"heck","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/heck-0.5.0/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libheck-ec3a93721bab7113.rlib","/Users/rocket/pixelchangecheck/target/debug/deps/libheck-ec3a93721bab7113.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#base64@0.22.1","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/base64-0.22.1/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"base64","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/base64-0.22.1/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["alloc","default","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libbase64-1ba25f5af2dc3a33.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#opaque-debug@0.3.1","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/opaque-debug-0.3.1/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"opaque_debug","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/opaque-debug-0.3.1/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libopaque_debug-5a1f3a33d1eefab6.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#rustc-hash@1.1.0","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rustc-hash-1.1.0/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"rustc_hash","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rustc-hash-1.1.0/src/lib.rs","edition":"2015","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/librustc_hash-97f2d59c8441c124.rmeta"],"executable":null,"fresh":true}
{"reason":"build-script-executed","package_id":"registry+https://github.com/rust-lang/crates.io-index#cpal@0.15.3","linked_libs":[],"linked_paths":[],"cfgs":[],"env":[],"out_dir":"/Users/rocket/pixelchangecheck/target/debug/build/cpal-4738607a40a382a4/out"}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#tracing-subscriber@0.3.19","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tracing-subscriber-0.3.19/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"tracing_subscriber","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tracing-subscriber-0.3.19/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["alloc","ansi","default","env-filter","fmt","json","matchers","nu-ansi-term","once_cell","regex","registry","serde","serde_json","sharded-slab","smallvec","std","thread_local","tracing","tracing-log","tracing-serde"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libtracing_subscriber-f3b3f6edf5bea671.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#image@0.24.9","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/image-0.24.9/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"image","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/image-0.24.9/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["bmp","dds","default","dxt","exr","farbfeld","gif","hdr","ico","jpeg","jpeg_rayon","openexr","png","pnm","qoi","tga","tiff","webp"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libimage-ac7582a0b0b908e4.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#thiserror@2.0.21","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/thiserror-2.0.21/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"thiserror","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/thiserror-2.0.21/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libthiserror-8b5ad39463e7c252.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#clap_builder@4.5.60","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/clap_builder-4.5.60/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"clap_builder","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/clap_builder-4.5.60/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["color","error-context","help","std","suggestions","usage"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libclap_builder-4ec53bc93a9218dc.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#twox-hash@1.6.3","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/twox-hash-1.6.3/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"twox_hash","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/twox-hash-1.6.3/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libtwox_hash-cff044ee6f9145c0.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#poly1305@0.8.0","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/poly1305-0.8.0/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"poly1305","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/poly1305-0.8.0/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libpoly1305-c1b8b09a3aa44826.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#clap_derive@4.5.55","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/clap_derive-4.5.55/Cargo.toml","target":{"kind":["proc-macro"],"crate_types":["proc-macro"],"name":"clap_derive","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/clap_derive-4.5.55/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libclap_derive-240e6aecd6c04ac0.dylib"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#pem@3.0.4","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/pem-3.0.4/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"pem","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/pem-3.0.4/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libpem-57a0ddd49c0c4f71.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#quinn-proto@0.10.6","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/quinn-proto-0.10.6/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"quinn_proto","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/quinn-proto-0.10.6/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["log","native-certs","ring","rustls","rustls-native-certs","tls-rustls"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libquinn_proto-8ff0eb140e5a5ff5.rmeta"],"executable":null,"fresh":true}
{"reason":"build-script-executed","package_id":"registry+https://github.com/rust-lang/crates.io-index#minifb@0.27.0","linked_libs":["static=minifb_native","framework=Metal","framework=MetalKit"],"linked_paths":["native=/Users/rocket/pixelchangecheck/target/debug/build/minifb-93f714e0979d3139/out"],"cfgs":[],"env":[],"out_dir":"/Users/rocket/pixelchangecheck/target/debug/build/minifb-93f714e0979d3139/out"}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#coreaudio-rs@0.11.3","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/coreaudio-rs-0.11.3/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"coreaudio","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/coreaudio-rs-0.11.3/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["audio_unit","core_audio"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libcoreaudio-758f222f8e117628.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#display-info@0.4.8","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/display-info-0.4.8/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"display_info","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/display-info-0.4.8/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libdisplay_info-14e570672b7ba9ec.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#opusic-sys@0.7.5","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/opusic-sys-0.7.5/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"opusic_sys","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/opusic-sys-0.7.5/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["bundled","default"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libopusic_sys-86c7265b385d3099.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#chacha20@0.9.1","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/chacha20-0.9.1/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"chacha20","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/chacha20-0.9.1/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["zeroize"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libchacha20-b663788889db4ab1.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#core-graphics@0.22.3","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/core-graphics-0.22.3/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"core_graphics","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/core-graphics-0.22.3/src/lib.rs","edition":"2015","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libcore_graphics-1c6b31061f344660.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#curve25519-dalek@4.1.3","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/curve25519-dalek-4.1.3/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"curve25519_dalek","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/curve25519-dalek-4.1.3/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["alloc","precomputed-tables","zeroize"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libcurve25519_dalek-6cdcbdaf475892a5.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#yasna@0.5.2","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/yasna-0.5.2/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"yasna","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/yasna-0.5.2/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","std","time"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libyasna-773630ba98fd05ff.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#quinn-udp@0.4.1","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/quinn-udp-0.4.1/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"quinn_udp","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/quinn-udp-0.4.1/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["log"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libquinn_udp-36b42f3ad032bd86.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#aead@0.5.2","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/aead-0.5.2/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"aead","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/aead-0.5.2/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["alloc","getrandom","rand_core"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libaead-d228056cd8aa80eb.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#crossbeam-channel@0.5.17","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-channel-0.5.17/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"crossbeam_channel","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-channel-0.5.17/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libcrossbeam_channel-e44619aeb468b745.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#mach2@0.4.3","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/mach2-0.4.3/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"mach2","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/mach2-0.4.3/src/lib.rs","edition":"2015","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libmach2-2e5edc10ac8ea83c.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#ascii@1.1.0","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ascii-1.1.0/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"ascii","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ascii-1.1.0/src/lib.rs","edition":"2015","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["alloc","default","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libascii-618a0fbebae2d603.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#chunked_transfer@1.5.0","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/chunked_transfer-1.5.0/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"chunked_transfer","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/chunked_transfer-1.5.0/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libchunked_transfer-67eb0bb0592f94ab.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#dasp_sample@0.11.0","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/dasp_sample-0.11.0/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"dasp_sample","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/dasp_sample-0.11.0/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libdasp_sample-5cedeaec84fc1c85.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#httpdate@1.0.3","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/httpdate-1.0.3/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"httpdate","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/httpdate-1.0.3/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libhttpdate-606a06afd6eaf740.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#raw-window-handle@0.6.2","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/raw-window-handle-0.6.2/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"raw_window_handle","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/raw-window-handle-0.6.2/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libraw_window_handle-570b3b7e00cf9bfd.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#symlink@0.1.0","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/symlink-0.1.0/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"symlink","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/symlink-0.1.0/src/lib.rs","edition":"2015","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libsymlink-645a7e9228e43dc7.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#percent-encoding@2.3.1","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/percent-encoding-2.3.1/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"percent_encoding","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/percent-encoding-2.3.1/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["alloc","default","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libpercent_encoding-551e24e27191ba3a.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#rcgen@0.12.1","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rcgen-0.12.1/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"rcgen","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rcgen-0.12.1/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","pem","ring"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/librcgen-2bc7df78eea48f02.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#x25519-dalek@2.0.1","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/x25519-dalek-2.0.1/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"x25519_dalek","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/x25519-dalek-2.0.1/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["alloc","default","precomputed-tables","zeroize"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libx25519_dalek-c0ef390e2e63c4e3.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#quinn@0.10.2","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/quinn-0.10.2/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"quinn","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/quinn-0.10.2/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","log","native-certs","ring","runtime-tokio","rustls","tls-rustls"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libquinn-a70c108c3a98a700.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#chacha20poly1305@0.10.1","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/chacha20poly1305-0.10.1/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"chacha20poly1305","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/chacha20poly1305-0.10.1/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["alloc","default","getrandom","rand_core"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libchacha20poly1305-547cb10ef3b91d04.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#lz4_flex@0.11.3","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/lz4_flex-0.11.3/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"lz4_flex","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/lz4_flex-0.11.3/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","frame","safe-decode","safe-encode","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/liblz4_flex-e8cb22dc6431b83b.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#screenshots@0.8.10","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/screenshots-0.8.10/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"screenshots","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/screenshots-0.8.10/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libscreenshots-7befc4d86131b4b8.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#tiny_http@0.12.0","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tiny_http-0.12.0/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"tiny_http","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tiny_http-0.12.0/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libtiny_http-5733f99b5c2853a1.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#cpal@0.15.3","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cpal-0.15.3/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"cpal","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cpal-0.15.3/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libcpal-746e4f2fbddd2ff0.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#minifb@0.27.0","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/minifb-0.27.0/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"minifb","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/minifb-0.27.0/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","dlib","dlopen","lazy_static","libc","tempfile","wayland","wayland-client","wayland-cursor","wayland-protocols","x11","x11-dl"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libminifb-e67c5b90ead7a299.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#tracing-appender@0.2.5","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tracing-appender-0.2.5/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"tracing_appender","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tracing-appender-0.2.5/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libtracing_appender-7f5eba1252d0f7cd.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#clap@4.5.60","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/clap-4.5.60/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"clap","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/clap-4.5.60/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["color","default","derive","error-context","help","std","suggestions","usage"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libclap-c7b3d1bf256fb623.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#opus@0.4.0","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/opus-0.4.0/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"opus","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/opus-0.4.0/src/lib.rs","edition":"2015","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libopus-592d67a375ff6b2d.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#sha2@0.10.9","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/sha2-0.10.9/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"sha2","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/sha2-0.10.9/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libsha2-60bff79bb356a608.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#sha1@0.10.7","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/sha1-0.10.7/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"sha1","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/sha1-0.10.7/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libsha1-29fdbe867ec233c2.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#tokio-rustls@0.24.1","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-rustls-0.24.1/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"tokio_rustls","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-rustls-0.24.1/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","logging","tls12"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libtokio_rustls-9e25fc0364bd610e.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#rgb@0.8.50","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rgb-0.8.50/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"rgb","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rgb-0.8.50/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["argb","as-bytes","bytemuck","default","grb"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/librgb-44b488ccf535a937.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#bincode@1.3.3","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bincode-1.3.3/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"bincode","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bincode-1.3.3/src/lib.rs","edition":"2015","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libbincode-902f2460ca23f353.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#async-trait@0.1.83","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/async-trait-0.1.83/Cargo.toml","target":{"kind":["proc-macro"],"crate_types":["proc-macro"],"name":"async_trait","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/async-trait-0.1.83/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libasync_trait-285b82754303e713.dylib"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#num_cpus@1.16.0","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/num_cpus-1.16.0/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"num_cpus","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/num_cpus-1.16.0/src/lib.rs","edition":"2015","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libnum_cpus-7b8d843e363c5b57.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#jpeg-encoder@0.5.1","manifest_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/jpeg-encoder-0.5.1/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"jpeg_encoder","src_path":"/Users/rocket/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/jpeg-encoder-0.5.1/src/lib.rs","edition":"2018","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","simd","std"],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libjpeg_encoder-818519c98ee1671c.rmeta"],"executable":null,"fresh":true}
{"reason":"compiler-artifact","package_id":"path+file:///Users/rocket/pixelchangecheck#pixel-change-check-client@0.1.0","manifest_path":"/Users/rocket/pixelchangecheck/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"pixel_change_check_client","src_path":"/Users/rocket/pixelchangecheck/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libpixel_change_check_client-763d47393fc71f0d.rmeta"],"executable":null,"fresh":false}
{"reason":"compiler-artifact","package_id":"path+file:///Users/rocket/pixelchangecheck#pixel-change-check-client@0.1.0","manifest_path":"/Users/rocket/pixelchangecheck/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"pixel_change_check_client","src_path":"/Users/rocket/pixelchangecheck/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":true},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libpixel_change_check_client-5c59cf51ac098417.rmeta"],"executable":null,"fresh":false}
{"reason":"compiler-artifact","package_id":"path+file:///Users/rocket/pixelchangecheck#pixel-change-check-client@0.1.0","manifest_path":"/Users/rocket/pixelchangecheck/Cargo.toml","target":{"kind":["example"],"crate_types":["bin"],"name":"benchmarks","src_path":"/Users/rocket/pixelchangecheck/benches/benchmarks.rs","edition":"2021","doc":false,"doctest":false,"test":false},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/examples/libbenchmarks-591337f31d7fcdb2.rmeta"],"executable":null,"fresh":false}
{"reason":"compiler-artifact","package_id":"path+file:///Users/rocket/pixelchangecheck#pixel-change-check-client@0.1.0","manifest_path":"/Users/rocket/pixelchangecheck/Cargo.toml","target":{"kind":["example"],"crate_types":["bin"],"name":"simple_screen_share","src_path":"/Users/rocket/pixelchangecheck/examples/simple_screen_share.rs","edition":"2021","doc":false,"doctest":false,"test":false},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/examples/libsimple_screen_share-4faeecfae8b33e67.rmeta"],"executable":null,"fresh":false}
{"reason":"compiler-artifact","package_id":"path+file:///Users/rocket/pixelchangecheck#pixel-change-check-client@0.1.0","manifest_path":"/Users/rocket/pixelchangecheck/Cargo.toml","target":{"kind":["test"],"crate_types":["bin"],"name":"replication","src_path":"/Users/rocket/pixelchangecheck/tests/replication.rs","edition":"2021","doc":false,"doctest":false,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":true},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libreplication-d0b7763bf5de3628.rmeta"],"executable":null,"fresh":false}
{"reason":"compiler-artifact","package_id":"path+file:///Users/rocket/pixelchangecheck#pixel-change-check-client@0.1.0","manifest_path":"/Users/rocket/pixelchangecheck/Cargo.toml","target":{"kind":["test"],"crate_types":["bin"],"name":"transport","src_path":"/Users/rocket/pixelchangecheck/tests/transport.rs","edition":"2021","doc":false,"doctest":false,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":true},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libtransport-2e7b1a74c37df86d.rmeta"],"executable":null,"fresh":false}
{"reason":"compiler-artifact","package_id":"path+file:///Users/rocket/pixelchangecheck#pixel-change-check-client@0.1.0","manifest_path":"/Users/rocket/pixelchangecheck/Cargo.toml","target":{"kind":["bin"],"crate_types":["bin"],"name":"pixel-change-check-client","src_path":"/Users/rocket/pixelchangecheck/src/main.rs","edition":"2021","doc":true,"doctest":false,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libpixel_change_check_client-3666197747228b33.rmeta"],"executable":null,"fresh":false}
{"reason":"compiler-artifact","package_id":"path+file:///Users/rocket/pixelchangecheck#pixel-change-check-client@0.1.0","manifest_path":"/Users/rocket/pixelchangecheck/Cargo.toml","target":{"kind":["bin"],"crate_types":["bin"],"name":"pixel-change-check-client","src_path":"/Users/rocket/pixelchangecheck/src/main.rs","edition":"2021","doc":true,"doctest":false,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":true},"features":[],"filenames":["/Users/rocket/pixelchangecheck/target/debug/deps/libpixel_change_check_client-d6f2bd1862d8821c.rmeta"],"executable":null,"fresh":false}
{"reason":"build-finished","success":true}

[stderr]
    Checking pixel-change-check-client v0.1.0 (/Users/rocket/pixelchangecheck)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.08s
```

### oxlint

```text
{ "diagnostics": [{"message": "This module could be mistakenly parsed as script instead of module","code": "import(unambiguous)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/import/unambiguous.html","help": "Add at least one import or export statement to unambiguously mark this file as a module","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 0,"length": 0,"line": 1,"column": 1}}]},
{"message": "File has too many classes (5). Maximum allowed is 1","code": "eslint(max-classes-per-file)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/max-classes-per-file.html","help": "Reduce the number of classes in this file","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 539,"length": 1,"line": 12,"column": 1}}]},
{"message": "File has too many lines (538).","code": "eslint(max-lines)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/max-lines.html","help": "Maximum allowed is 300.","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 18118,"length": 0,"line": 538,"column": 3}}]},
{"message": "Unexpected function declaration in the global scope.","code": "eslint(no-implicit-globals)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-implicit-globals.html","help": "Wrap it in an IIFE for a local variable, or assign it as a global property for a global variable.","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 12989,"length": 11,"line": 357,"column": 10}}]},
{"message": "Unexpected function declaration in the global scope.","code": "eslint(no-implicit-globals)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-implicit-globals.html","help": "Wrap it in an IIFE for a local variable, or assign it as a global property for a global variable.","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 2662,"length": 9,"line": 79,"column": 10}}]},
{"message": "Unexpected function declaration in the global scope.","code": "eslint(no-implicit-globals)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-implicit-globals.html","help": "Wrap it in an IIFE for a local variable, or assign it as a global property for a global variable.","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 17337,"length": 8,"line": 506,"column": 10}}]},
{"message": "Unexpected function declaration in the global scope.","code": "eslint(no-implicit-globals)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-implicit-globals.html","help": "Wrap it in an IIFE for a local variable, or assign it as a global property for a global variable.","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 17728,"length": 21,"line": 522,"column": 10}}]},
{"message": "Unexpected function declaration in the global scope.","code": "eslint(no-implicit-globals)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-implicit-globals.html","help": "Wrap it in an IIFE for a local variable, or assign it as a global property for a global variable.","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 9658,"length": 10,"line": 267,"column": 10}}]},
{"message": "Unexpected function declaration in the global scope.","code": "eslint(no-implicit-globals)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-implicit-globals.html","help": "Wrap it in an IIFE for a local variable, or assign it as a global property for a global variable.","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 9442,"length": 4,"line": 260,"column": 10}}]},
{"message": "Unexpected function declaration in the global scope.","code": "eslint(no-implicit-globals)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-implicit-globals.html","help": "Wrap it in an IIFE for a local variable, or assign it as a global property for a global variable.","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 9943,"length": 4,"line": 276,"column": 10}}]},
{"message": "Unexpected function declaration in the global scope.","code": "eslint(no-implicit-globals)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-implicit-globals.html","help": "Wrap it in an IIFE for a local variable, or assign it as a global property for a global variable.","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 17536,"length": 9,"line": 514,"column": 10}}]},
{"message": "Unexpected function declaration in the global scope.","code": "eslint(no-implicit-globals)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-implicit-globals.html","help": "Wrap it in an IIFE for a local variable, or assign it as a global property for a global variable.","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 10312,"length": 12,"line": 286,"column": 10}}]},
{"message": "Unexpected function declaration in the global scope.","code": "eslint(no-implicit-globals)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-implicit-globals.html","help": "Wrap it in an IIFE for a local variable, or assign it as a global property for a global variable.","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 3936,"length": 22,"line": 112,"column": 10}}]},
{"message": "Unexpected function declaration in the global scope.","code": "eslint(no-implicit-globals)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-implicit-globals.html","help": "Wrap it in an IIFE for a local variable, or assign it as a global property for a global variable.","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 9277,"length": 4,"line": 253,"column": 10}}]},
{"message": "Unexpected comment inline with code","code": "eslint(no-inline-comments)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-inline-comments.html","help": "Move the comment to a separate line","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 3240,"length": 34,"line": 93,"column": 33}}]},
{"message": "'location' is not defined.","code": "eslint(no-undef)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-undef.html","help": "Either define 'location' or remove the reference to it. If 'location' is a global variable, add it to the 'globals' configuration.","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 14585,"length": 8,"line": 417,"column": 19}}]},
{"message": "'location' is not defined.","code": "eslint(no-undef)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-undef.html","help": "Either define 'location' or remove the reference to it. If 'location' is a global variable, add it to the 'globals' configuration.","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 14662,"length": 8,"line": 418,"column": 31}}]},
{"message": "'TextEncoder' is not defined.","code": "eslint(no-undef)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-undef.html","help": "Either define 'TextEncoder' or remove the reference to it. If 'TextEncoder' is a global variable, add it to the 'globals' configuration.","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 13028,"length": 11,"line": 358,"column": 19}}]},
{"message": "'TextDecoder' is not defined.","code": "eslint(no-undef)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-undef.html","help": "Either define 'TextDecoder' or remove the reference to it. If 'TextDecoder' is a global variable, add it to the 'globals' configuration.","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 12808,"length": 11,"line": 348,"column": 32}}]},
{"message": "'WebSocket' is not defined.","code": "eslint(no-undef)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-undef.html","help": "Either define 'WebSocket' or remove the reference to it. If 'WebSocket' is a global variable, add it to the 'globals' configuration.","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 14745,"length": 9,"line": 419,"column": 24}}]},
{"message": "'WebSocket' is not defined.","code": "eslint(no-undef)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-undef.html","help": "Either define 'WebSocket' or remove the reference to it. If 'WebSocket' is a global variable, add it to the 'globals' configuration.","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 15538,"length": 9,"line": 441,"column": 51}}]},
{"message": "'WebSocket' is not defined.","code": "eslint(no-undef)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-undef.html","help": "Either define 'WebSocket' or remove the reference to it. If 'WebSocket' is a global variable, add it to the 'globals' configuration.","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 17248,"length": 9,"line": 500,"column": 51}}]},
{"message": "'requestAnimationFrame' is not defined.","code": "eslint(no-undef)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-undef.html","help": "Either define 'requestAnimationFrame' or remove the reference to it. If 'requestAnimationFrame' is a global variable, add it to the 'globals' configuration.","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 13925,"length": 21,"line": 391,"column": 5}}]},
{"message": "'createImageBitmap' is not defined.","code": "eslint(no-undef)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-undef.html","help": "Either define 'createImageBitmap' or remove the reference to it. If 'createImageBitmap' is a global variable, add it to the 'globals' configuration.","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 6781,"length": 17,"line": 189,"column": 28}}]},
{"message": "'OffscreenCanvas' is not defined.","code": "eslint(no-undef)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-undef.html","help": "Either define 'OffscreenCanvas' or remove the reference to it. If 'OffscreenCanvas' is a global variable, add it to the 'globals' configuration.","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 7027,"length": 15,"line": 193,"column": 23}}]},
{"message": "'document' is not defined.","code": "eslint(no-undef)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-undef.html","help": "Either define 'document' or remove the reference to it. If 'document' is a global variable, add it to the 'globals' configuration.","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 4657,"length": 8,"line": 137,"column": 21}}]},
{"message": "'document' is not defined.","code": "eslint(no-undef)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-undef.html","help": "Either define 'document' or remove the reference to it. If 'document' is a global variable, add it to the 'globals' configuration.","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 13689,"length": 8,"line": 383,"column": 19}}]},
{"message": "'document' is not defined.","code": "eslint(no-undef)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-undef.html","help": "Either define 'document' or remove the reference to it. If 'document' is a global variable, add it to the 'globals' configuration.","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 14478,"length": 8,"line": 412,"column": 16}}]},
{"message": "'Blob' is not defined.","code": "eslint(no-undef)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-undef.html","help": "Either define 'Blob' or remove the reference to it. If 'Blob' is a global variable, add it to the 'globals' configuration.","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 6714,"length": 4,"line": 188,"column": 24}}]},
{"message": "'ImageData' is not defined.","code": "eslint(no-undef)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-undef.html","help": "Either define 'ImageData' or remove the reference to it. If 'ImageData' is a global variable, add it to the 'globals' configuration.","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 14276,"length": 9,"line": 404,"column": 31}}]},
{"message": "'setTimeout' is not defined.","code": "eslint(no-undef)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-undef.html","help": "Either define 'setTimeout' or remove the reference to it. If 'setTimeout' is a global variable, add it to the 'globals' configuration.","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 15341,"length": 10,"line": 435,"column": 7}}]},
{"message": "'window' is not defined.","code": "eslint(no-undef)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-undef.html","help": "Either define 'window' or remove the reference to it. If 'window' is a global variable, add it to the 'globals' configuration.","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 17814,"length": 6,"line": 526,"column": 1}}]},
{"message": "Variable 'FMT_PNG' is declared but never used. Unused variables should start with a '_'.","code": "eslint(no-unused-vars)","severity": "error","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-unused-vars.html","help": "Consider removing this declaration.","filename": "src/server/renderer/client.js","labels": [{"label": "'FMT_PNG' is declared here","span": {"offset": 1010,"length": 7,"line": 33,"column": 7}}]},
{"message": "Unexpected class with only a constructor.","code": "typescript(no-extraneous-class)","severity": "error","url": "https://oxc.rs/docs/guide/usage/linter/rules/typescript/no-extraneous-class.html","help": "Try replacing this class with a standalone function or deleting it entirely","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 1030,"length": 8,"line": 35,"column": 7}}]},
{"message": "Unary operator '++' used.","code": "eslint(no-plusplus)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-plusplus.html","help": "Use the assignment operator `+=` instead.","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 1806,"length": 13,"line": 58,"column": 50}}]},
{"message": "Variable declarations should be sorted","code": "eslint(sort-vars)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/sort-vars.html","help": "Sort variable declarations in ascending order (case-sensitive by default).","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 2742,"length": 5,"line": 81,"column": 14}}]},
{"message": "Unary operator '++' used.","code": "eslint(no-plusplus)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-plusplus.html","help": "Use the assignment operator `+=` instead.","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 2798,"length": 3,"line": 83,"column": 23}}]},
{"message": "Unexpected use of `\">>\"`.","code": "eslint(no-bitwise)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-bitwise.html","help": "bitwise operators are not allowed, maybe you mistyped `&&` or `||`?","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 2822,"length": 10,"line": 84,"column": 19}}]},
{"message": "Unary operator '++' used.","code": "eslint(no-plusplus)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-plusplus.html","help": "Use the assignment operator `+=` instead.","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 2892,"length": 3,"line": 87,"column": 20}}]},
{"message": "Unexpected use of `\"|\"`.","code": "eslint(no-bitwise)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-bitwise.html","help": "bitwise operators are not allowed, maybe you mistyped `&&` or `||`?","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 3294,"length": 26,"line": 94,"column": 20}}]},
{"message": "Unexpected use of `\"<<\"`.","code": "eslint(no-bitwise)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-bitwise.html","help": "bitwise operators are not allowed, maybe you mistyped `&&` or `||`?","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 3304,"length": 15,"line": 94,"column": 30}}]},
{"message": "Unexpected use of `\"&\"`.","code": "eslint(no-bitwise)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-bitwise.html","help": "bitwise operators are not allowed, maybe you mistyped `&&` or `||`?","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 3433,"length": 12,"line": 97,"column": 17}}]},
{"message": "Unary operator '++' used.","code": "eslint(no-plusplus)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-plusplus.html","help": "Use the assignment operator `+=` instead.","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 3503,"length": 3,"line": 100,"column": 20}}]},
{"message": "Unary operator '++' used.","code": "eslint(no-plusplus)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-plusplus.html","help": "Use the assignment operator `+=` instead.","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 3785,"length": 3,"line": 106,"column": 32}}]},
{"message": "Unary operator '++' used.","code": "eslint(no-plusplus)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-plusplus.html","help": "Use the assignment operator `+=` instead.","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 3794,"length": 3,"line": 106,"column": 41}}]},
{"message": "Unary operator '++' used.","code": "eslint(no-plusplus)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-plusplus.html","help": "Use the assignment operator `+=` instead.","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 3805,"length": 6,"line": 106,"column": 52}}]},
{"message": "Do not use `new Array(singleArgument)`.","code": "unicorn(no-new-array)","severity": "error","url": "https://oxc.rs/docs/guide/usage/linter/rules/unicorn/no-new-array.html","help": "It's not clear whether the argument is meant to be the length of the array or the only element. If the argument is the array's length, consider using `Array.from({ length: n })`. If the argument is the only element, use `[element]`.","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 5208,"length": 17,"line": 151,"column": 14}}]},
{"message": "Unexpected use of `undefined`","code": "eslint(no-undefined)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-undefined.html","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 5550,"length": 9,"line": 160,"column": 30}}]},
{"message": "async is not allowed","code": "oxc(no-async-await)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/oxc/no-async-await.html","help": "Remove the `async` keyword","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 5815,"length": 5,"line": 168,"column": 3}}]},
{"message": "Unexpected use of `undefined`","code": "eslint(no-undefined)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-undefined.html","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 6178,"length": 9,"line": 175,"column": 54}}]},
{"message": "'join' was used before it was defined.","code": "eslint(no-use-before-define)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-use-before-define.html","help": "Move the declaration before any references to it, or remove the reference if it is not needed.","filename": "src/server/renderer/client.js","labels": [{"label": "used here","span": {"offset": 6618,"length": 4,"line": 186,"column": 36}},{"label": "defined here","span": {"offset": 9277,"length": 4,"line": 253,"column": 10}}]},
{"message": "object spread property are not allowed. ","code": "oxc(no-rest-spread-properties)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/oxc/no-rest-spread-properties.html","help": "Use `Object.assign()` to combine objects instead of object spread syntax.","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 8579,"length": 5,"line": 230,"column": 23}}]},
{"message": "'blit' was used before it was defined.","code": "eslint(no-use-before-define)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-use-before-define.html","help": "Move the declaration before any references to it, or remove the reference if it is not needed.","filename": "src/server/renderer/client.js","labels": [{"label": "used here","span": {"offset": 8895,"length": 4,"line": 240,"column": 32}},{"label": "defined here","span": {"offset": 9442,"length": 4,"line": 260,"column": 10}}]},
{"message": "'fill' was used before it was defined.","code": "eslint(no-use-before-define)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-use-before-define.html","help": "Move the declaration before any references to it, or remove the reference if it is not needed.","filename": "src/server/renderer/client.js","labels": [{"label": "used here","span": {"offset": 8997,"length": 4,"line": 241,"column": 37}},{"label": "defined here","span": {"offset": 9943,"length": 4,"line": 276,"column": 10}}]},
{"message": "'blitRegion' was used before it was defined.","code": "eslint(no-use-before-define)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-use-before-define.html","help": "Move the declaration before any references to it, or remove the reference if it is not needed.","filename": "src/server/renderer/client.js","labels": [{"label": "used here","span": {"offset": 9073,"length": 10,"line": 242,"column": 12}},{"label": "defined here","span": {"offset": 9658,"length": 10,"line": 267,"column": 10}}]},
{"message": "Unary operator '++' used.","code": "eslint(no-plusplus)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-plusplus.html","help": "Use the assignment operator `+=` instead.","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 9536,"length": 5,"line": 262,"column": 30}}]},
{"message": "Unary operator '++' used.","code": "eslint(no-plusplus)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-plusplus.html","help": "Use the assignment operator `+=` instead.","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 9766,"length": 5,"line": 269,"column": 30}}]},
{"message": "Unary operator '++' used.","code": "eslint(no-plusplus)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-plusplus.html","help": "Use the assignment operator `+=` instead.","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 10053,"length": 3,"line": 278,"column": 26}}]},
{"message": "Unary operator '++' used.","code": "eslint(no-plusplus)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-plusplus.html","help": "Use the assignment operator `+=` instead.","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 10164,"length": 3,"line": 279,"column": 26}}]},
{"message": "The function `parseMessage` has too many lines (70). Maximum allowed is 50.","code": "eslint(max-lines-per-function)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/max-lines-per-function.html","help": "Consider splitting it into smaller functions.","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 10303,"length": 2675,"line": 286,"column": 1}}]},
{"message": "Unary operator '++' used.","code": "eslint(no-plusplus)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-plusplus.html","help": "Use the assignment operator `+=` instead.","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 11747,"length": 3,"line": 323,"column": 34}}]},
{"message": "Variable declarations should be sorted","code": "eslint(sort-vars)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/sort-vars.html","help": "Sort variable declarations in ascending order (case-sensitive by default).","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 11794,"length": 11,"line": 324,"column": 41}}]},
{"message": "Variable declarations should be sorted","code": "eslint(sort-vars)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/sort-vars.html","help": "Sort variable declarations in ascending order (case-sensitive by default).","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 11807,"length": 11,"line": 324,"column": 54}}]},
{"message": "Prefer `.querySelector()` over `.getElementById()`.","code": "unicorn(prefer-query-selector)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/unicorn/prefer-query-selector.html","help": "It's better to use the same method to query DOM elements. This helps keep consistency and it lends itself to future improvements (e.g. more specific selectors).","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 13698,"length": 14,"line": 383,"column": 28}}]},
{"message": "Expected method `status` to have this.","code": "eslint(class-methods-use-this)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/class-methods-use-this.html","help": "Consider converting method `status` to a static method.","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 14448,"length": 6,"line": 411,"column": 3}}]},
{"message": "Prefer `.querySelector()` over `.getElementById()`.","code": "unicorn(prefer-query-selector)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/unicorn/prefer-query-selector.html","help": "It's better to use the same method to query DOM elements. This helps keep consistency and it lends itself to future improvements (e.g. more specific selectors).","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 14487,"length": 14,"line": 412,"column": 25}}]},
{"message": "Prefer `addEventListener()` over their `on`-function counterparts.","code": "unicorn(prefer-add-event-listener)","severity": "error","url": "https://oxc.rs/docs/guide/usage/linter/rules/unicorn/prefer-add-event-listener.html","help": "`addEventListener()` can register multiple handlers and accepts options such as `{ once: true }`; assigning to `on<event>` replaces any previously registered handler.","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 14838,"length": 6,"line": 423,"column": 12}}]},
{"message": "Prefer `addEventListener()` over their `on`-function counterparts.","code": "unicorn(prefer-add-event-listener)","severity": "error","url": "https://oxc.rs/docs/guide/usage/linter/rules/unicorn/prefer-add-event-listener.html","help": "`addEventListener()` can register multiple handlers and accepts options such as `{ once: true }`; assigning to `on<event>` replaces any previously registered handler.","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 15118,"length": 7,"line": 430,"column": 12}}]},
{"message": "Prefer `addEventListener()` over their `on`-function counterparts.","code": "unicorn(prefer-add-event-listener)","severity": "error","url": "https://oxc.rs/docs/guide/usage/linter/rules/unicorn/prefer-add-event-listener.html","help": "`addEventListener()` can register multiple handlers and accepts options such as `{ once: true }`; assigning to `on<event>` replaces any previously registered handler.","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 15240,"length": 7,"line": 433,"column": 12}}]},
{"message": "Prefer `addEventListener()` over their `on`-function counterparts.","code": "unicorn(prefer-add-event-listener)","severity": "error","url": "https://oxc.rs/docs/guide/usage/linter/rules/unicorn/prefer-add-event-listener.html","help": "`addEventListener()` can register multiple handlers and accepts options such as `{ once: true }`; assigning to `on<event>` replaces any previously registered handler.","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 15399,"length": 9,"line": 437,"column": 12}}]},
{"message": "'encodeAck' was used before it was defined.","code": "eslint(no-use-before-define)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-use-before-define.html","help": "Move the declaration before any references to it, or remove the reference if it is not needed.","filename": "src/server/renderer/client.js","labels": [{"label": "used here","span": {"offset": 15579,"length": 9,"line": 442,"column": 24}},{"label": "defined here","span": {"offset": 17536,"length": 9,"line": 514,"column": 10}}]},
{"message": "The async method `onMessage` has too many lines (52). Maximum allowed is 50.","code": "eslint(max-lines-per-function)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/max-lines-per-function.html","help": "Consider splitting it into smaller functions.","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 15624,"length": 1550,"line": 446,"column": 18}}]},
{"message": "async is not allowed","code": "oxc(no-async-await)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/oxc/no-async-await.html","help": "Remove the `async` keyword","filename": "src/server/renderer/client.js","labels": [{"span": {"offset": 15609,"length": 5,"line": 446,"column": 3}}]},
{"message": "'encodeRequestKeyframe' was used before it was defined.","code": "eslint(no-use-before-define)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-use-before-define.html","help": "Move the declaration before any references to it, or remove the reference if it is not needed.","filename": "src/server/renderer/client.js","labels": [{"label": "used here","span": {"offset": 17289,"length": 21,"line": 501,"column": 24}},{"label": "defined here","span": {"offset": 17728,"length": 21,"line": 522,"column": 10}}]}],
              "number_of_files": 1,
              "number_of_rules": 315,
              "threads_count": 10,
              "start_time": 0.0351885
            }
```

### eslint

```text
[{"filePath":"/Users/rocket/pixelchangecheck/src/server/renderer/client.js","messages":[{"ruleId":"no-unused-vars","severity":2,"message":"'FMT_PNG' is assigned a value but never used.","line":33,"column":7,"messageId":"unusedVar","endLine":33,"endColumn":14,"suggestions":[{"messageId":"removeVar","data":{"varName":"FMT_PNG"},"fix":{"range":[1004,1022],"text":""},"desc":"Remove unused variable 'FMT_PNG'."}]},{"ruleId":"no-undef","severity":2,"message":"'document' is not defined.","line":137,"column":21,"messageId":"undef","endLine":137,"endColumn":29},{"ruleId":"no-undef","severity":2,"message":"'Blob' is not defined.","line":188,"column":24,"messageId":"undef","endLine":188,"endColumn":28},{"ruleId":"no-undef","severity":2,"message":"'createImageBitmap' is not defined.","line":189,"column":28,"messageId":"undef","endLine":189,"endColumn":45},{"ruleId":"no-undef","severity":2,"message":"'OffscreenCanvas' is not defined.","line":193,"column":23,"messageId":"undef","endLine":193,"endColumn":38},{"ruleId":"no-undef","severity":2,"message":"'TextDecoder' is not defined.","line":348,"column":32,"messageId":"undef","endLine":348,"endColumn":43},{"ruleId":"no-undef","severity":2,"message":"'TextEncoder' is not defined.","line":358,"column":19,"messageId":"undef","endLine":358,"endColumn":30},{"ruleId":"no-undef","severity":2,"message":"'document' is not defined.","line":383,"column":19,"messageId":"undef","endLine":383,"endColumn":27},{"ruleId":"no-undef","severity":2,"message":"'requestAnimationFrame' is not defined.","line":391,"column":5,"messageId":"undef","endLine":391,"endColumn":26},{"ruleId":"no-undef","severity":2,"message":"'ImageData' is not defined.","line":404,"column":31,"messageId":"undef","endLine":404,"endColumn":40},{"ruleId":"no-undef","severity":2,"message":"'document' is not defined.","line":412,"column":16,"messageId":"undef","endLine":412,"endColumn":24},{"ruleId":"no-undef","severity":2,"message":"'location' is not defined.","line":417,"column":19,"messageId":"undef","endLine":417,"endColumn":27},{"ruleId":"no-undef","severity":2,"message":"'location' is not defined.","line":418,"column":31,"messageId":"undef","endLine":418,"endColumn":39},{"ruleId":"no-undef","severity":2,"message":"'WebSocket' is not defined.","line":419,"column":24,"messageId":"undef","endLine":419,"endColumn":33},{"ruleId":"no-undef","severity":2,"message":"'setTimeout' is not defined.","line":435,"column":7,"messageId":"undef","endLine":435,"endColumn":17},{"ruleId":"no-undef","severity":2,"message":"'WebSocket' is not defined.","line":441,"column":51,"messageId":"undef","endLine":441,"endColumn":60},{"ruleId":"no-undef","severity":2,"message":"'WebSocket' is not defined.","line":500,"column":51,"messageId":"undef","endLine":500,"endColumn":60},{"ruleId":"no-undef","severity":2,"message":"'window' is not defined.","line":526,"column":1,"messageId":"undef","endLine":526,"endColumn":7}],"suppressedMessages":[],"errorCount":18,"fatalErrorCount":0,"warningCount":0,"fixableErrorCount":0,"fixableWarningCount":0,"source":"// PixelChangeCheck browser compositor.\n//\n// This is a direct port of src/pcc/compositor.rs. The same invariants\n// hold here: a snapshot is committed atomically, an update never reads a\n// reference this buffer does not have, a revision is applied at most\n// once, and a copy reads the surface as it was *before* the transaction\n// (which is what makes an overlapping move or a swap work).\n//\n// The wire format is the same explicit little-endian one the native\n// viewer parses. Nothing about the stream is re-encoded for the browser.\n\nconst PROTOCOL_VERSION = 3;\nconst MAX_FRAME_BYTES = 100_000_000;\n\nconst OP_RECT = 0x01;\nconst OP_FILL = 0x02;\nconst OP_COPY = 0x03;\n\nconst K_HELLO = 0x01;\nconst K_REQUEST_KEYFRAME = 0x02;\nconst K_ACK = 0x03;\nconst K_SNAPSHOT_BEGIN = 0x04;\nconst K_SNAPSHOT_CHUNK = 0x05;\nconst K_PARTIAL_UPDATE = 0x06;\nconst K_KEEP_ALIVE = 0x07;\nconst K_QUALITY = 0x08;\nconst K_ERROR = 0x09;\nconst K_BYE = 0x0A;\nconst K_SNAPSHOT_COMMIT = 0x0B;\n\nconst FMT_RAW = 0;\nconst FMT_LZ4 = 1;\nconst FMT_PNG = 2;\n\nclass Rejected {\n  constructor(reason) { this.reason = reason; }\n}\n\nclass Reject extends Error {}\n\n// ---------------------------------------------------------------- reader\n\nclass Reader {\n  constructor(view, offset = 0) {\n    // A DataView for little-endian reads and a Uint8Array over the same\n    // bytes for slicing: they are different objects and only one of them\n    // can be sliced.\n    this.view = view;\n    this.bytes = new Uint8Array(view.buffer, view.byteOffset, view.byteLength);\n    this.offset = offset;\n  }\n  get remaining() { return this.view.byteLength - this.offset; }\n  need(n) {\n    if (n > this.remaining) {\n      throw new Reject(`truncated message: need ${n} more bytes, have ${this.remaining}`);\n    }\n  }\n  u8() { this.need(1); return this.view.getUint8(this.offset++); }\n  u16() { this.need(2); const v = this.view.getUint16(this.offset, true); this.offset += 2; return v; }\n  u32() { this.need(4); const v = this.view.getUint32(this.offset, true); this.offset += 4; return v; }\n  u64() {\n    this.need(8);\n    const v = this.view.getBigUint64(this.offset, true);\n    this.offset += 8;\n    return Number(v);\n  }\n  take(n) { this.need(n); const v = this.bytes.subarray(this.offset, this.offset + n); this.offset += n; return v; }\n  allocLen(max, what) {\n    const n = this.u32();\n    if (n > max) throw new Reject(`${what} too large: ${n} (max ${max})`);\n    return n;\n  }\n}\n\n// ------------------------------------------------------------- lz4 block\n\n// LZ4 block format decoder. Small on purpose: the format is a handful of\n// opcodes and pulling in a library for it would be larger than this file.\nfunction lz4Decode(src, expected) {\n  const dst = new Uint8Array(expected);\n  let s = 0, d = 0;\n  while (s < src.length) {\n    const token = src[s++];\n    let literal = token >> 4;\n    if (literal === 15) {\n      let n;\n      do { n = src[s++]; literal += n; } while (n === 255);\n    }\n    if (s + literal > src.length) throw new Reject('lz4: literal run runs past the input');\n    if (d + literal > dst.length) throw new Reject('lz4: output overflow while copying literals');\n    dst.set(src.subarray(s, s + literal), d);\n    s += literal; d += literal;\n    if (s >= src.length) break; // last sequence has literals only\n    const offset = src[s] | (src[s + 1] << 8);\n    s += 2;\n    if (offset === 0 || offset > d) throw new Reject('lz4: invalid match offset');\n    let match = token & 0x0F;\n    if (match === 15) {\n      let n;\n      do { n = src[s++]; match += n; } while (n === 255);\n    }\n    match += 4;\n    if (d + match > dst.length) throw new Reject('lz4: output overflow while copying a match');\n    // Overlapping copies are legal and common: copy byte by byte.\n    let from = d - offset;\n    for (let i = 0; i < match; i++) dst[d++] = dst[from++];\n  }\n  if (d !== expected) throw new Reject(`lz4: decoded to ${d} bytes, expected ${expected}`);\n  return dst;\n}\n\nfunction lz4DecodeSizePrepended(src, expected) {\n  if (src.length < 4) throw new Reject('lz4: payload too short');\n  const size = new DataView(src.buffer, src.byteOffset, 4).getUint32(0, true);\n  if (size !== expected) {\n    throw new Reject(`lz4: declared ${size} bytes, geometry requires ${expected}`);\n  }\n  return lz4Decode(src.subarray(4), expected);\n}\n\n// ------------------------------------------------------------ compositor\n\nclass Compositor {\n  constructor() {\n    this.buffer = null;\n    this.width = 0;\n    this.height = 0;\n    this.epoch = 0;\n    this.rev = 0;\n    this.fresh = false;\n    this.incoming = null;\n    this.onPaint = null;\n  }\n\n  surface() {\n    if (this.buffer) return this.buffer;\n    const scratch = document.createElement('canvas');\n    scratch.width = 1;\n    scratch.height = 1;\n    return scratch.getContext('2d').createImageData(1, 1).data;\n  }\n\n  beginSnapshot(epoch, width, height, format, totalLen, chunks) {\n    if (this.fresh && epoch < this.epoch) {\n      throw new Reject(`snapshot epoch ${epoch} is older than the installed epoch ${this.epoch}`);\n    }\n    if (totalLen === 0) throw new Reject('snapshot announced with total_len=0');\n    this.incoming = {\n      epoch, width, height, format, totalLen,\n      expected: chunks,\n      parts: new Array(chunks),\n      received: 0,\n    };\n  }\n\n  pushSnapshotChunk(index, data) {\n    const inc = this.incoming;\n    if (!inc) throw new Reject(`snapshot chunk ${index} with no snapshot in progress`);\n    if (index >= inc.expected) throw new Reject(`snapshot chunk index ${index} outside 0..${inc.expected}`);\n    if (inc.parts[index] !== undefined) throw new Reject(`snapshot chunk ${index} arrived twice`);\n    inc.parts[index] = data;\n    inc.received += data.length;\n    if (inc.received > inc.totalLen) {\n      throw new Reject(`snapshot overflows: ${inc.received} > ${inc.totalLen}`);\n    }\n  }\n\n  async commitSnapshot(rev, epoch) {\n    const inc = this.incoming;\n    if (!inc) throw new Reject('snapshot commit with no snapshot in progress');\n    this.incoming = null;\n    if (inc.epoch !== epoch) {\n      throw new Reject(`snapshot commit epoch ${epoch} does not match the transfer's ${inc.epoch}`);\n    }\n    const missing = inc.parts.findIndex((p) => p === undefined);\n    if (missing >= 0) throw new Reject(`snapshot committed with chunk ${missing} missing`);\n    if (inc.received !== inc.totalLen) {\n      throw new Reject(`snapshot is ${inc.received} bytes, expected ${inc.totalLen}`);\n    }\n\n    let rgb;\n    if (inc.format === FMT_RAW) {\n      rgb = new Uint8ClampedArray(inc.totalLen);\n      rgb.set(inc.parts[0]);\n    } else if (inc.format === FMT_LZ4) {\n      rgb = lz4DecodeSizePrepended(join(inc.parts, inc.totalLen), inc.width * inc.height * 3);\n    } else {\n      const blob = new Blob(inc.parts, { type: 'image/png' });\n      const bitmap = await createImageBitmap(blob);\n      if (bitmap.width !== inc.width || bitmap.height !== inc.height) {\n        throw new Reject(`snapshot declares ${inc.width}x${inc.height} but decoded ${bitmap.width}x${bitmap.height}`);\n      }\n      const off = new OffscreenCanvas(inc.width, inc.height);\n      const ctx = off.getContext('2d');\n      ctx.drawImage(bitmap, 0, 0);\n      rgb = ctx.getImageData(0, 0, inc.width, inc.height).data;\n      bitmap.close();\n      if (rgb.length > MAX_FRAME_BYTES) throw new Reject('decoded snapshot exceeds the frame budget');\n    }\n\n    // Nothing above this line touched `this`, so a failed commit leaves\n    // the displayed surface exactly as it was.\n    this.buffer = rgb;\n    this.width = inc.width;\n    this.height = inc.height;\n    this.epoch = inc.epoch;\n    this.rev = rev;\n    this.fresh = true;\n    this.paint();\n  }\n\n  applyOps(rev, epoch, ops) {\n    if (!this.fresh) throw new Rejected('needs-snapshot');\n    if (epoch < this.epoch) throw new Rejected('stale-epoch');\n    if (epoch > this.epoch) throw new Rejected('needs-snapshot');\n    if (rev <= this.rev) throw new Rejected('already-applied');\n\n    // Validate and materialise everything first.\n    const staged = [];\n    for (const op of ops) {\n      if (op.w === 0 || op.h === 0) throw new Reject(`empty op ${op.w}x${op.h}`);\n      if (op.x + op.w > this.width || op.y + op.h > this.height) {\n        throw new Reject(`op (${op.x},${op.y}) ${op.w}x${op.h} exceeds frame ${this.width}x${this.height}`);\n      }\n      if (op.kind === OP_COPY\n          && (op.srcX + op.w > this.width || op.srcY + op.h > this.height)) {\n        throw new Reject(`copy source (${op.srcX},${op.srcY}) ${op.w}x${op.h} exceeds frame ${this.width}x${this.height}`);\n      }\n      if (op.kind === OP_RECT) {\n        staged.push({ ...op, pixels: lz4DecodeSizePrepended(op.payload, op.w * op.h * 3) });\n      } else {\n        staged.push(op);\n      }\n    }\n\n    const needsSnapshot = staged.some((o) => o.kind === OP_COPY);\n    const base = needsSnapshot ? this.buffer.slice() : null;\n\n    for (const op of staged) {\n      if (op.kind === OP_RECT) blit(this.buffer, this.width, op.x, op.y, op.w, op.h, op.pixels);\n      else if (op.kind === OP_FILL) fill(this.buffer, this.width, op.x, op.y, op.w, op.h, op.color);\n      else blitRegion(this.buffer, this.width, op.x, op.y, op.w, op.h, base, op.srcX, op.srcY);\n    }\n    this.rev = rev;\n    this.paint();\n  }\n\n  paint() {\n    if (this.onPaint) this.onPaint(this);\n  }\n}\n\nfunction join(parts, totalLen) {\n  const out = new Uint8Array(totalLen);\n  let at = 0;\n  for (const p of parts) { out.set(p, at); at += p.length; }\n  return out;\n}\n\nfunction blit(dst, dstWidth, x, y, w, h, src) {\n  const rowBytes = w * 3;\n  for (let row = 0; row < h; row++) {\n    dst.set(src.subarray(row * rowBytes, (row + 1) * rowBytes), ((y + row) * dstWidth + x) * 3);\n  }\n}\n\nfunction blitRegion(dst, dstWidth, x, y, w, h, src, sx, sy) {\n  const rowBytes = w * 3;\n  for (let row = 0; row < h; row++) {\n    const to = ((y + row) * dstWidth + x) * 3;\n    const from = ((sy + row) * dstWidth + sx) * 3;\n    dst.set(src.subarray(from, from + rowBytes), to);\n  }\n}\n\nfunction fill(dst, dstWidth, x, y, w, h, color) {\n  const row = new Uint8ClampedArray(w * 3);\n  for (let i = 0; i < w; i++) { row[i * 3] = color[0]; row[i * 3 + 1] = color[1]; row[i * 3 + 2] = color[2]; }\n  for (let r = 0; r < h; r++) {\n    dst.set(row, ((y + r) * dstWidth + x) * 3);\n  }\n}\n\n// ----------------------------------------------------------- message read\n\nfunction parseMessage(bytes) {\n  const buf = bytes instanceof Uint8Array ? bytes : new Uint8Array(bytes);\n  if (buf.length < 5) throw new Reject(`message too short: ${buf.length} bytes`);\n  const view = new DataView(buf.buffer, buf.byteOffset, buf.byteLength);\n  if (view.getUint8(0) !== PROTOCOL_VERSION) {\n    throw new Reject(`protocol version mismatch: expected ${PROTOCOL_VERSION}, got ${view.getUint8(0)}`);\n  }\n  const len = view.getUint32(1, true);\n  if (len !== buf.length - 5) throw new Reject(`message length mismatch: header says ${len}`);\n  const r = new Reader(view, 5);\n  const kind = r.u8();\n  switch (kind) {\n    case K_SNAPSHOT_BEGIN:\n      return {\n        kind,\n        rev: r.u64(),\n        epoch: r.u32(),\n        width: r.u32(),\n        height: r.u32(),\n        format: r.u8(),\n        totalLen: r.allocLen(MAX_FRAME_BYTES + 65536, 'snapshot'),\n        chunks: r.u32(),\n      };\n    case K_SNAPSHOT_CHUNK: {\n      const rev = r.u64();\n      const index = r.u32();\n      const n = r.allocLen(1024 * 1024, 'snapshot chunk');\n      return { kind, rev, index, data: r.take(n).slice() };\n    }\n    case K_SNAPSHOT_COMMIT:\n      return { kind, rev: r.u64(), epoch: r.u32() };\n    case K_PARTIAL_UPDATE: {\n      const rev = r.u64();\n      const epoch = r.u32();\n      const count = r.u32();\n      if (count > 8192) throw new Reject(`too many ops in one update: ${count}`);\n      const ops = [];\n      for (let i = 0; i < count; i++) {\n        const x = r.u32(), y = r.u32(), w = r.u32(), h = r.u32();\n        const opKind = r.u8();\n        if (opKind === OP_RECT) {\n          const n = r.allocLen(w * h * 3 + 16, 'rect payload');\n          ops.push({ kind: OP_RECT, x, y, w, h, payload: r.take(n).slice() });\n        } else if (opKind === OP_FILL) {\n          const n = r.allocLen(3, 'fill colour');\n          ops.push({ kind: OP_FILL, x, y, w, h, color: Array.from(r.take(n)) });\n        } else if (opKind === OP_COPY) {\n          const srcX = r.u32();\n          const srcY = r.u32();\n          ops.push({ kind: OP_COPY, x, y, w, h, srcX, srcY });\n        } else {\n          throw new Reject(`unknown op kind: 0x${opKind.toString(16)}`);\n        }\n      }\n      return { kind, rev, epoch, ops };\n    }\n    case K_KEEP_ALIVE:\n      return { kind, rev: r.u64() };\n    case K_QUALITY:\n      return { kind, targetFps: r.u32(), maxFps: r.u32(), quality: r.view.getFloat32(r.offset, true) };\n    case K_ERROR: {\n      const n = r.allocLen(1024, 'Error message');\n      return { kind, text: new TextDecoder().decode(r.take(n)) };\n    }\n    case K_BYE:\n      return { kind };\n    default:\n      throw new Reject(`unknown message kind: 0x${kind.toString(16)}`);\n  }\n}\n\nfunction encodeHello(token) {\n  const raw = new TextEncoder().encode(token);\n  const out = new Uint8Array(5 + 1 + 2 + raw.length);\n  const view = new DataView(out.buffer);\n  const bodyAt = 5;\n  out[0] = PROTOCOL_VERSION;\n  view.setUint32(1, 1 + 2 + raw.length, true);\n  out[bodyAt] = K_HELLO;\n  view.setUint16(bodyAt + 1, raw.length, true);\n  out.set(raw, bodyAt + 3);\n  return out;\n}\n\n// ------------------------------------------------------------- transport\n\nclass Session {\n  constructor(token) {\n    this.token = token;\n    this.compositor = new Compositor();\n    this.socket = null;\n    this.lastAck = 0;\n    this.paintScheduled = false;\n    this.setupCanvas();\n  }\n\n  setupCanvas() {\n    this.canvas = document.getElementById('screen');\n    this.ctx = this.canvas.getContext('2d');\n    this.compositor.onPaint = () => this.schedulePaint();\n  }\n\n  schedulePaint() {\n    if (this.paintScheduled) return;\n    this.paintScheduled = true;\n    requestAnimationFrame(() => {\n      this.paintScheduled = false;\n      this.paint();\n    });\n  }\n\n  paint() {\n    const c = this.compositor;\n    if (!c.fresh) return;\n    if (this.canvas.width !== c.width || this.canvas.height !== c.height) {\n      this.canvas.width = c.width;\n      this.canvas.height = c.height;\n    }\n    this.ctx.putImageData(new ImageData(new Uint8ClampedArray(c.buffer), c.width, c.height), 0, 0);\n    if (c.rev !== this.lastAck) {\n      this.lastAck = c.rev;\n      this.sendAck(c.rev);\n    }\n  }\n\n  status(text) {\n    const el = document.getElementById('status');\n    if (el) el.textContent = text;\n  }\n\n  connect() {\n    const proto = location.protocol === 'https:' ? 'wss' : 'ws';\n    const url = `${proto}://${location.host}/ws?token=${encodeURIComponent(this.token)}`;\n    const socket = new WebSocket(url);\n    socket.binaryType = 'arraybuffer';\n    this.socket = socket;\n\n    socket.onopen = () => {\n      this.status('connected');\n      socket.send(encodeHello(this.token));\n    };\n    // onerror is called with an Event, not a message; the detail lives on\n    // the event, so read it from there rather than treating the argument\n    // as a string.\n    socket.onerror = (event) => {\n      this.status(`connection error: ${(event && event.message) || 'unknown'}`);\n    };\n    socket.onclose = (event) => {\n      this.status(`disconnected (code ${event.code}); retrying in 2s`);\n      setTimeout(() => this.connect(), 2000);\n    };\n    socket.onmessage = (event) => this.onMessage(new Uint8Array(event.data));\n  }\n\n  sendAck(rev) {\n    if (this.socket && this.socket.readyState === WebSocket.OPEN) {\n      this.socket.send(encodeAck(rev));\n    }\n  }\n\n  async onMessage(bytes) {\n    let msg;\n    try {\n      msg = parseMessage(bytes);\n    } catch (e) {\n      this.status(`rejected: ${e.message}`);\n      return;\n    }\n    const c = this.compositor;\n    try {\n      switch (msg.kind) {\n        case K_SNAPSHOT_BEGIN:\n          c.beginSnapshot(msg.epoch, msg.width, msg.height, msg.format, msg.totalLen, msg.chunks);\n          break;\n        case K_SNAPSHOT_CHUNK:\n          c.pushSnapshotChunk(msg.index, msg.data);\n          break;\n        case K_SNAPSHOT_COMMIT:\n          await c.commitSnapshot(msg.rev, msg.epoch);\n          this.status(`${c.width}x${c.height} rev ${c.rev}`);\n          break;\n        case K_PARTIAL_UPDATE:\n          c.applyOps(msg.rev, msg.epoch, msg.ops);\n          break;\n        case K_KEEP_ALIVE:\n        case K_QUALITY:\n          break;\n        case K_ERROR:\n          this.status(msg.text);\n          break;\n        case K_BYE:\n          this.status('the sharer ended the session');\n          this.socket.close();\n          break;\n        default:\n          break;\n      }\n    } catch (e) {\n      if (e instanceof Rejected) {\n        // A revision we already hold is the expected consequence of\n        // joining mid-stream: the join snapshot covers it. Only a missing\n        // base is worth repairing.\n        if (e.reason === 'needs-snapshot' || e.reason === 'stale-epoch') {\n          this.status(`repairing: ${e.reason}`);\n          this.requestKeyframe();\n        }\n      } else {\n        this.status(`invalid update: ${e.message}`);\n        this.requestKeyframe();\n      }\n    }\n  }\n\n  requestKeyframe() {\n    if (this.socket && this.socket.readyState === WebSocket.OPEN) {\n      this.socket.send(encodeRequestKeyframe());\n    }\n  }\n}\n\nfunction envelope(body) {\n  const out = new Uint8Array(5 + body.length);\n  out[0] = PROTOCOL_VERSION;\n  new DataView(out.buffer).setUint32(1, body.length, true);\n  out.set(body, 5);\n  return out;\n}\n\nfunction encodeAck(rev) {\n  const body = new Uint8Array(9);\n  const view = new DataView(body.buffer);\n  body[0] = K_ACK;\n  view.setBigUint64(1, BigInt(rev), true);\n  return envelope(body);\n}\n\nfunction encodeRequestKeyframe() {\n  return envelope(new Uint8Array([K_REQUEST_KEYFRAME]));\n}\n\nwindow.pcc = {\n  connect(token) {\n    const session = new Session(token);\n    session.connect();\n    return session;\n  },\n  // Exposed so the same parser and the same LZ4 decoder can be exercised\n  // by a test page without a live sharer.\n  parseMessage,\n  lz4Decode,\n  Compositor,\n  PROTOCOL_VERSION,\n};\n","usedDeprecatedRules":[]}]
```

### madge

```text
Processed 1 file (1.6s) 



[stderr]
- Finding files
✔ No circular dependency found!
```

### taplo

```text
[stderr]
 INFO taplo:lint_files:collect_files: found files total=2 excluded=0 files=["/Users/rocket/pixelchangecheck/Cargo.toml", "/Users/rocket/pixelchangecheck/bughunt.toml"] cwd="/Users/rocket/pixelchangecheck"
```

### yamllint

```text
.omp/config.yml
  2:16      error    no new line character at the end of file  (new-line-at-end-of-file)

.scc/config.yaml
  4:3       error    wrong indentation: expected 4 but found 2  (indentation)
```
