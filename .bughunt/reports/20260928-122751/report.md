# BugHunt Report

- Generated: `2026-09-28T12:27:51.555607-04:00`
- Profile: `all`
- Defense health: **57/100** (execution/defense health, not probability of bug-freedom)
- Elapsed: **3.5s**
- Raw normalized findings: **104**
- Logical issue clusters: **67** (non-destructive dedup view)
- Distinct repeated signals: **33**
- Cross-tool correlated locations: **0**
- Deterministic auto-fixes: **0** total (**0 safe**, 0 unsafe, 0 review-required)
- Agent repair queue: [`agent/FIX_QUEUE.md`](agent/FIX_QUEUE.md)
- Auto-fix inventory: [`agent/AUTOFIX.md`](agent/AUTOFIX.md)
- Coverage-weighted risk map: [`agent/RISK_MAP.md`](agent/RISK_MAP.md)
- Deduplicated repair queue: [`agent/DEDUPLICATED_QUEUE.md`](agent/DEDUPLICATED_QUEUE.md)
- Repair checklist: [`agent/CHECKLIST.md`](agent/CHECKLIST.md)

## ODC-style defect taxonomy

- **63** — `checking`
- **37** — `function`
- **2** — `timing/serialization`
- **1** — `build/package/merge`
- **1** — `interface`

## Highest-priority repeated signals

| Count | Tool / rule | Representative message |
|---:|---|---|
| 17 | `eslint:no-undef` | 'document' is not defined. |
| 4 | `oxlint:unicorn(prefer-add-event-listener)` | Prefer `addEventListener()` over their `on`-function counterparts. |
| 1 | `actionlint:[{"message":"label <path> is unknown. available labels are <path> <path> <pa…` | [{"message":"label \"macos-13\" is unknown. available labels are \"windows-latest\", \"windows-latest-8-cores\", \"windows-2025\", \"windows-2025-vs2026\", \"windows-2022\", \"windows-11-arm\", \"ubun |
| 1 | `eslint:no-unused-vars` | 'FMT_PNG' is assigned a value but never used. |
| 1 | `knip:error: unable to find package.json` | ERROR: Unable to find package.json |
| 1 | `oxlint:eslint(no-unused-vars)` | Variable 'FMT_PNG' is declared but never used. Unused variables should start with a '_'. |
| 1 | `oxlint:typescript(no-extraneous-class)` | Unexpected class with only a constructor. |
| 1 | `oxlint:unicorn(no-new-array)` | Do not use `new Array(singleArgument)`. |
| 1 | `shellcheck:SC1036` | '(' is invalid here. Did you forget to escape it? |
| 1 | `shellcheck:SC1072` | Expected end of $(..) expression. Fix any mentioned problems and try again. |
| 1 | `shellcheck:SC1073` | Couldn't parse this command expansion. Fix to allow more checks. |
| 1 | `yamllint:<n>:<n> error wrong indentation: expected <n> but found <n> (indentation)` | 4:3       error    wrong indentation: expected 4 but found 2  (indentation) |
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
| 1 | `oxlint:eslint(class-methods-use-this)` | Expected method `status` to have this. |

## Hot files

- **92** findings — `src/server/renderer/client.js`
- **4** findings — `deploy/relay/provision-oracle.sh`
- **4** findings — `Dockerfile`

## Defense results

| Status | Tool | Class | Findings | Time | Note |
|---|---|---|---:|---:|---|
| PASS | `complexity` | complexity-budgets | 0 | 0.1s |  |
| FINDINGS | `system-ir` | structural-graph | 1 | 1.5s |  |
| PASS | `verify-gaps` | direct-verification | 0 | 0.3s |  |
| PASS | `protocol` | protocol-correctness | 0 | 0.1s |  |
| PASS | `data` | data-invariants | 0 | 0.1s |  |
| FINDINGS | `actionlint` | ci-correctness | 1 | 0.2s |  |
| FINDINGS | `shellcheck` | shell-correctness | 4 | 0.2s |  |
| PASS | `dotenv-linter` | config-correctness | 0 | 0.0s |  |
| FINDINGS | `hadolint` | container-correctness | 4 | 0.2s |  |
| FINDINGS | `oxlint` | js-ts-correctness | 74 | 0.1s |  |
| FINDINGS | `eslint` | js-ts-correctness | 18 | 3.2s |  |
| ERROR | `knip` | js-ts-dead-contract | 1 | 0.4s |  |
| PASS | `madge` | js-ts-architecture | 0 | 1.3s |  |
| PASS | `taplo` | config-correctness | 0 | 0.0s |  |
| FINDINGS | `yamllint` | config-correctness | 1 | 0.2s |  |
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
| SKIPPED | `lizard` | cross-language-complexity | 0 | 0.0s | lizard not installed |
| N/A | `vulture` | dead-code | 0 | 0.0s | not applicable: no first-party Python capability detected |
| N/A | `bandit` | security | 0 | 0.0s | not applicable: no first-party Python capability detected |
| N/A | `deptry` | dependencies | 0 | 0.0s | not applicable: no first-party Python capability detected |
| N/A | `import-linter` | architecture | 0 | 0.0s | not applicable: no first-party Python capability detected |
| N/A | `ast-grep` | structural | 0 | 0.0s | not applicable: no first-party Python capability detected |
| SKIPPED | `semgrep` | semantic-static | 0 | 0.0s | semgrep not installed |
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
| SKIPPED | `clippy` | rust-correctness | 0 | 0.0s | Rust detected but cargo/clippy is not installed |
| N/A | `cppcheck` | cpp-correctness | 0 | 0.0s | not applicable: no cpp capability detected |
| N/A | `clang-tidy` | cpp-correctness | 0 | 0.0s | not applicable: no cpp-compile-db capability detected |
| N/A | `infer` | whole-program-native | 0 | 0.0s | not applicable: no cpp-compile-db capability detected |
| N/A | `phpstan` | php-types | 0 | 0.0s | not applicable: no php capability detected |
| N/A | `react-doctor` | react-correctness | 0 | 0.0s | not applicable: no react capability detected |
| SKIPPED | `tsc` | ts-types | 0 | 0.0s | TypeScript detected but no root tsconfig.json project exists |
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

### BH-A5609DE26B67C21B — knip

- Location: `<unknown>:?:?`
- Severity: `error`
- Signal: `knip:error: unable to find package.json`
- Fingerprint: `a5609de26b67c21b`

ERROR: Unable to find package.json

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

### BH-2A62F0022F311771 — scc / scc:unresolved-references

- Location: `<unknown>:?:?`
- Severity: `warning`
- Signal: `scc:scc:unresolved-references`
- Fingerprint: `2a62f0022f311771`

scc left 1788 likely-internal call/reference edges unresolved (resolved=895); graph-based seam/contract conclusions are partial over these edges

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

## Accepted debt

_No accepted debt recorded in debt.toml._

## Execution failures / unavailable defenses

- **knip** — ERROR: see raw output
- **mutmut** — SKIPPED: explicitly skipped by user
- **lizard** — SKIPPED: lizard not installed
- **semgrep** — SKIPPED: semgrep not installed
- **schemathesis** — SKIPPED: no safe runnable API target discovered/configured
- **custom** — SKIPPED: no high-confidence repository-specific semantic campaign could be inferred
- **clippy** — SKIPPED: Rust detected but cargo/clippy is not installed
- **tsc** — SKIPPED: TypeScript detected but no root tsconfig.json project exists
- **publint** — SKIPPED: publint is installed but package.json is missing name/version; add both fields to make the package publishable

## Raw output

### complexity

```text
[]
```

### system-ir

```text
{"findings": [{"tool": "scc", "code": "scc:unresolved-references", "message": "scc left 1788 likely-internal call/reference edges unresolved (resolved=895); graph-based seam/contract conclusions are partial over these edges", "severity": "warning"}], "cache": "/Users/rocket/pixelchangecheck/.bughunt/cache/system-ir.json"}
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
{"comments":[{"file":"deploy/relay/provision-oracle.sh","line":182,"endLine":182,"column":15,"endColumn":15,"level":"info","code":1009,"message":"The mentioned syntax error was in this double quoted string.","fix":null},{"file":"deploy/relay/provision-oracle.sh","line":182,"endLine":182,"column":72,"endColumn":72,"level":"error","code":1073,"message":"Couldn't parse this command expansion. Fix to allow more checks.","fix":null},{"file":"deploy/relay/provision-oracle.sh","line":182,"endLine":182,"column":94,"endColumn":94,"level":"error","code":1036,"message":"'(' is invalid here. Did you forget to escape it?","fix":null},{"file":"deploy/relay/provision-oracle.sh","line":182,"endLine":182,"column":94,"endColumn":94,"level":"error","code":1072,"message":"Expected end of $(..) expression. Fix any mentioned problems and try again.","fix":null}]}
```

### dotenv-linter

```text
Nothing to check
```

### hadolint

```text
[{"code":"DL3008","column":1,"file":"Dockerfile","level":"warning","line":5,"message":"Pin versions in apt get install. Instead of `apt-get install <package>` use `apt-get install <package>=<version>`"},{"code":"DL3008","column":1,"file":"Dockerfile","level":"warning","line":15,"message":"Pin versions in apt get install. Instead of `apt-get install <package>` use `apt-get install <package>=<version>`"},{"code":"DL3066","column":1,"file":"Dockerfile","level":"info","line":21,"message":"Non-numeric user-id may not be resolvable by host system"},{"code":"DL3025","column":1,"file":"Dockerfile","level":"warning","line":23,"message":"Use arguments JSON notation for CMD and ENTRYPOINT arguments"}]
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
              "start_time": 0.037034042
            }
```

### eslint

```text
[{"filePath":"/Users/rocket/pixelchangecheck/src/server/renderer/client.js","messages":[{"ruleId":"no-unused-vars","severity":2,"message":"'FMT_PNG' is assigned a value but never used.","line":33,"column":7,"messageId":"unusedVar","endLine":33,"endColumn":14,"suggestions":[{"messageId":"removeVar","data":{"varName":"FMT_PNG"},"fix":{"range":[1004,1022],"text":""},"desc":"Remove unused variable 'FMT_PNG'."}]},{"ruleId":"no-undef","severity":2,"message":"'document' is not defined.","line":137,"column":21,"messageId":"undef","endLine":137,"endColumn":29},{"ruleId":"no-undef","severity":2,"message":"'Blob' is not defined.","line":188,"column":24,"messageId":"undef","endLine":188,"endColumn":28},{"ruleId":"no-undef","severity":2,"message":"'createImageBitmap' is not defined.","line":189,"column":28,"messageId":"undef","endLine":189,"endColumn":45},{"ruleId":"no-undef","severity":2,"message":"'OffscreenCanvas' is not defined.","line":193,"column":23,"messageId":"undef","endLine":193,"endColumn":38},{"ruleId":"no-undef","severity":2,"message":"'TextDecoder' is not defined.","line":348,"column":32,"messageId":"undef","endLine":348,"endColumn":43},{"ruleId":"no-undef","severity":2,"message":"'TextEncoder' is not defined.","line":358,"column":19,"messageId":"undef","endLine":358,"endColumn":30},{"ruleId":"no-undef","severity":2,"message":"'document' is not defined.","line":383,"column":19,"messageId":"undef","endLine":383,"endColumn":27},{"ruleId":"no-undef","severity":2,"message":"'requestAnimationFrame' is not defined.","line":391,"column":5,"messageId":"undef","endLine":391,"endColumn":26},{"ruleId":"no-undef","severity":2,"message":"'ImageData' is not defined.","line":404,"column":31,"messageId":"undef","endLine":404,"endColumn":40},{"ruleId":"no-undef","severity":2,"message":"'document' is not defined.","line":412,"column":16,"messageId":"undef","endLine":412,"endColumn":24},{"ruleId":"no-undef","severity":2,"message":"'location' is not defined.","line":417,"column":19,"messageId":"undef","endLine":417,"endColumn":27},{"ruleId":"no-undef","severity":2,"message":"'location' is not defined.","line":418,"column":31,"messageId":"undef","endLine":418,"endColumn":39},{"ruleId":"no-undef","severity":2,"message":"'WebSocket' is not defined.","line":419,"column":24,"messageId":"undef","endLine":419,"endColumn":33},{"ruleId":"no-undef","severity":2,"message":"'setTimeout' is not defined.","line":435,"column":7,"messageId":"undef","endLine":435,"endColumn":17},{"ruleId":"no-undef","severity":2,"message":"'WebSocket' is not defined.","line":441,"column":51,"messageId":"undef","endLine":441,"endColumn":60},{"ruleId":"no-undef","severity":2,"message":"'WebSocket' is not defined.","line":500,"column":51,"messageId":"undef","endLine":500,"endColumn":60},{"ruleId":"no-undef","severity":2,"message":"'window' is not defined.","line":526,"column":1,"messageId":"undef","endLine":526,"endColumn":7}],"suppressedMessages":[],"errorCount":18,"fatalErrorCount":0,"warningCount":0,"fixableErrorCount":0,"fixableWarningCount":0,"source":"// PixelChangeCheck browser compositor.\n//\n// This is a direct port of src/pcc/compositor.rs. The same invariants\n// hold here: a snapshot is committed atomically, an update never reads a\n// reference this buffer does not have, a revision is applied at most\n// once, and a copy reads the surface as it was *before* the transaction\n// (which is what makes an overlapping move or a swap work).\n//\n// The wire format is the same explicit little-endian one the native\n// viewer parses. Nothing about the stream is re-encoded for the browser.\n\nconst PROTOCOL_VERSION = 3;\nconst MAX_FRAME_BYTES = 100_000_000;\n\nconst OP_RECT = 0x01;\nconst OP_FILL = 0x02;\nconst OP_COPY = 0x03;\n\nconst K_HELLO = 0x01;\nconst K_REQUEST_KEYFRAME = 0x02;\nconst K_ACK = 0x03;\nconst K_SNAPSHOT_BEGIN = 0x04;\nconst K_SNAPSHOT_CHUNK = 0x05;\nconst K_PARTIAL_UPDATE = 0x06;\nconst K_KEEP_ALIVE = 0x07;\nconst K_QUALITY = 0x08;\nconst K_ERROR = 0x09;\nconst K_BYE = 0x0A;\nconst K_SNAPSHOT_COMMIT = 0x0B;\n\nconst FMT_RAW = 0;\nconst FMT_LZ4 = 1;\nconst FMT_PNG = 2;\n\nclass Rejected {\n  constructor(reason) { this.reason = reason; }\n}\n\nclass Reject extends Error {}\n\n// ---------------------------------------------------------------- reader\n\nclass Reader {\n  constructor(view, offset = 0) {\n    // A DataView for little-endian reads and a Uint8Array over the same\n    // bytes for slicing: they are different objects and only one of them\n    // can be sliced.\n    this.view = view;\n    this.bytes = new Uint8Array(view.buffer, view.byteOffset, view.byteLength);\n    this.offset = offset;\n  }\n  get remaining() { return this.view.byteLength - this.offset; }\n  need(n) {\n    if (n > this.remaining) {\n      throw new Reject(`truncated message: need ${n} more bytes, have ${this.remaining}`);\n    }\n  }\n  u8() { this.need(1); return this.view.getUint8(this.offset++); }\n  u16() { this.need(2); const v = this.view.getUint16(this.offset, true); this.offset += 2; return v; }\n  u32() { this.need(4); const v = this.view.getUint32(this.offset, true); this.offset += 4; return v; }\n  u64() {\n    this.need(8);\n    const v = this.view.getBigUint64(this.offset, true);\n    this.offset += 8;\n    return Number(v);\n  }\n  take(n) { this.need(n); const v = this.bytes.subarray(this.offset, this.offset + n); this.offset += n; return v; }\n  allocLen(max, what) {\n    const n = this.u32();\n    if (n > max) throw new Reject(`${what} too large: ${n} (max ${max})`);\n    return n;\n  }\n}\n\n// ------------------------------------------------------------- lz4 block\n\n// LZ4 block format decoder. Small on purpose: the format is a handful of\n// opcodes and pulling in a library for it would be larger than this file.\nfunction lz4Decode(src, expected) {\n  const dst = new Uint8Array(expected);\n  let s = 0, d = 0;\n  while (s < src.length) {\n    const token = src[s++];\n    let literal = token >> 4;\n    if (literal === 15) {\n      let n;\n      do { n = src[s++]; literal += n; } while (n === 255);\n    }\n    if (s + literal > src.length) throw new Reject('lz4: literal run runs past the input');\n    if (d + literal > dst.length) throw new Reject('lz4: output overflow while copying literals');\n    dst.set(src.subarray(s, s + literal), d);\n    s += literal; d += literal;\n    if (s >= src.length) break; // last sequence has literals only\n    const offset = src[s] | (src[s + 1] << 8);\n    s += 2;\n    if (offset === 0 || offset > d) throw new Reject('lz4: invalid match offset');\n    let match = token & 0x0F;\n    if (match === 15) {\n      let n;\n      do { n = src[s++]; match += n; } while (n === 255);\n    }\n    match += 4;\n    if (d + match > dst.length) throw new Reject('lz4: output overflow while copying a match');\n    // Overlapping copies are legal and common: copy byte by byte.\n    let from = d - offset;\n    for (let i = 0; i < match; i++) dst[d++] = dst[from++];\n  }\n  if (d !== expected) throw new Reject(`lz4: decoded to ${d} bytes, expected ${expected}`);\n  return dst;\n}\n\nfunction lz4DecodeSizePrepended(src, expected) {\n  if (src.length < 4) throw new Reject('lz4: payload too short');\n  const size = new DataView(src.buffer, src.byteOffset, 4).getUint32(0, true);\n  if (size !== expected) {\n    throw new Reject(`lz4: declared ${size} bytes, geometry requires ${expected}`);\n  }\n  return lz4Decode(src.subarray(4), expected);\n}\n\n// ------------------------------------------------------------ compositor\n\nclass Compositor {\n  constructor() {\n    this.buffer = null;\n    this.width = 0;\n    this.height = 0;\n    this.epoch = 0;\n    this.rev = 0;\n    this.fresh = false;\n    this.incoming = null;\n    this.onPaint = null;\n  }\n\n  surface() {\n    if (this.buffer) return this.buffer;\n    const scratch = document.createElement('canvas');\n    scratch.width = 1;\n    scratch.height = 1;\n    return scratch.getContext('2d').createImageData(1, 1).data;\n  }\n\n  beginSnapshot(epoch, width, height, format, totalLen, chunks) {\n    if (this.fresh && epoch < this.epoch) {\n      throw new Reject(`snapshot epoch ${epoch} is older than the installed epoch ${this.epoch}`);\n    }\n    if (totalLen === 0) throw new Reject('snapshot announced with total_len=0');\n    this.incoming = {\n      epoch, width, height, format, totalLen,\n      expected: chunks,\n      parts: new Array(chunks),\n      received: 0,\n    };\n  }\n\n  pushSnapshotChunk(index, data) {\n    const inc = this.incoming;\n    if (!inc) throw new Reject(`snapshot chunk ${index} with no snapshot in progress`);\n    if (index >= inc.expected) throw new Reject(`snapshot chunk index ${index} outside 0..${inc.expected}`);\n    if (inc.parts[index] !== undefined) throw new Reject(`snapshot chunk ${index} arrived twice`);\n    inc.parts[index] = data;\n    inc.received += data.length;\n    if (inc.received > inc.totalLen) {\n      throw new Reject(`snapshot overflows: ${inc.received} > ${inc.totalLen}`);\n    }\n  }\n\n  async commitSnapshot(rev, epoch) {\n    const inc = this.incoming;\n    if (!inc) throw new Reject('snapshot commit with no snapshot in progress');\n    this.incoming = null;\n    if (inc.epoch !== epoch) {\n      throw new Reject(`snapshot commit epoch ${epoch} does not match the transfer's ${inc.epoch}`);\n    }\n    const missing = inc.parts.findIndex((p) => p === undefined);\n    if (missing >= 0) throw new Reject(`snapshot committed with chunk ${missing} missing`);\n    if (inc.received !== inc.totalLen) {\n      throw new Reject(`snapshot is ${inc.received} bytes, expected ${inc.totalLen}`);\n    }\n\n    let rgb;\n    if (inc.format === FMT_RAW) {\n      rgb = new Uint8ClampedArray(inc.totalLen);\n      rgb.set(inc.parts[0]);\n    } else if (inc.format === FMT_LZ4) {\n      rgb = lz4DecodeSizePrepended(join(inc.parts, inc.totalLen), inc.width * inc.height * 3);\n    } else {\n      const blob = new Blob(inc.parts, { type: 'image/png' });\n      const bitmap = await createImageBitmap(blob);\n      if (bitmap.width !== inc.width || bitmap.height !== inc.height) {\n        throw new Reject(`snapshot declares ${inc.width}x${inc.height} but decoded ${bitmap.width}x${bitmap.height}`);\n      }\n      const off = new OffscreenCanvas(inc.width, inc.height);\n      const ctx = off.getContext('2d');\n      ctx.drawImage(bitmap, 0, 0);\n      rgb = ctx.getImageData(0, 0, inc.width, inc.height).data;\n      bitmap.close();\n      if (rgb.length > MAX_FRAME_BYTES) throw new Reject('decoded snapshot exceeds the frame budget');\n    }\n\n    // Nothing above this line touched `this`, so a failed commit leaves\n    // the displayed surface exactly as it was.\n    this.buffer = rgb;\n    this.width = inc.width;\n    this.height = inc.height;\n    this.epoch = inc.epoch;\n    this.rev = rev;\n    this.fresh = true;\n    this.paint();\n  }\n\n  applyOps(rev, epoch, ops) {\n    if (!this.fresh) throw new Rejected('needs-snapshot');\n    if (epoch < this.epoch) throw new Rejected('stale-epoch');\n    if (epoch > this.epoch) throw new Rejected('needs-snapshot');\n    if (rev <= this.rev) throw new Rejected('already-applied');\n\n    // Validate and materialise everything first.\n    const staged = [];\n    for (const op of ops) {\n      if (op.w === 0 || op.h === 0) throw new Reject(`empty op ${op.w}x${op.h}`);\n      if (op.x + op.w > this.width || op.y + op.h > this.height) {\n        throw new Reject(`op (${op.x},${op.y}) ${op.w}x${op.h} exceeds frame ${this.width}x${this.height}`);\n      }\n      if (op.kind === OP_COPY\n          && (op.srcX + op.w > this.width || op.srcY + op.h > this.height)) {\n        throw new Reject(`copy source (${op.srcX},${op.srcY}) ${op.w}x${op.h} exceeds frame ${this.width}x${this.height}`);\n      }\n      if (op.kind === OP_RECT) {\n        staged.push({ ...op, pixels: lz4DecodeSizePrepended(op.payload, op.w * op.h * 3) });\n      } else {\n        staged.push(op);\n      }\n    }\n\n    const needsSnapshot = staged.some((o) => o.kind === OP_COPY);\n    const base = needsSnapshot ? this.buffer.slice() : null;\n\n    for (const op of staged) {\n      if (op.kind === OP_RECT) blit(this.buffer, this.width, op.x, op.y, op.w, op.h, op.pixels);\n      else if (op.kind === OP_FILL) fill(this.buffer, this.width, op.x, op.y, op.w, op.h, op.color);\n      else blitRegion(this.buffer, this.width, op.x, op.y, op.w, op.h, base, op.srcX, op.srcY);\n    }\n    this.rev = rev;\n    this.paint();\n  }\n\n  paint() {\n    if (this.onPaint) this.onPaint(this);\n  }\n}\n\nfunction join(parts, totalLen) {\n  const out = new Uint8Array(totalLen);\n  let at = 0;\n  for (const p of parts) { out.set(p, at); at += p.length; }\n  return out;\n}\n\nfunction blit(dst, dstWidth, x, y, w, h, src) {\n  const rowBytes = w * 3;\n  for (let row = 0; row < h; row++) {\n    dst.set(src.subarray(row * rowBytes, (row + 1) * rowBytes), ((y + row) * dstWidth + x) * 3);\n  }\n}\n\nfunction blitRegion(dst, dstWidth, x, y, w, h, src, sx, sy) {\n  const rowBytes = w * 3;\n  for (let row = 0; row < h; row++) {\n    const to = ((y + row) * dstWidth + x) * 3;\n    const from = ((sy + row) * dstWidth + sx) * 3;\n    dst.set(src.subarray(from, from + rowBytes), to);\n  }\n}\n\nfunction fill(dst, dstWidth, x, y, w, h, color) {\n  const row = new Uint8ClampedArray(w * 3);\n  for (let i = 0; i < w; i++) { row[i * 3] = color[0]; row[i * 3 + 1] = color[1]; row[i * 3 + 2] = color[2]; }\n  for (let r = 0; r < h; r++) {\n    dst.set(row, ((y + r) * dstWidth + x) * 3);\n  }\n}\n\n// ----------------------------------------------------------- message read\n\nfunction parseMessage(bytes) {\n  const buf = bytes instanceof Uint8Array ? bytes : new Uint8Array(bytes);\n  if (buf.length < 5) throw new Reject(`message too short: ${buf.length} bytes`);\n  const view = new DataView(buf.buffer, buf.byteOffset, buf.byteLength);\n  if (view.getUint8(0) !== PROTOCOL_VERSION) {\n    throw new Reject(`protocol version mismatch: expected ${PROTOCOL_VERSION}, got ${view.getUint8(0)}`);\n  }\n  const len = view.getUint32(1, true);\n  if (len !== buf.length - 5) throw new Reject(`message length mismatch: header says ${len}`);\n  const r = new Reader(view, 5);\n  const kind = r.u8();\n  switch (kind) {\n    case K_SNAPSHOT_BEGIN:\n      return {\n        kind,\n        rev: r.u64(),\n        epoch: r.u32(),\n        width: r.u32(),\n        height: r.u32(),\n        format: r.u8(),\n        totalLen: r.allocLen(MAX_FRAME_BYTES + 65536, 'snapshot'),\n        chunks: r.u32(),\n      };\n    case K_SNAPSHOT_CHUNK: {\n      const rev = r.u64();\n      const index = r.u32();\n      const n = r.allocLen(1024 * 1024, 'snapshot chunk');\n      return { kind, rev, index, data: r.take(n).slice() };\n    }\n    case K_SNAPSHOT_COMMIT:\n      return { kind, rev: r.u64(), epoch: r.u32() };\n    case K_PARTIAL_UPDATE: {\n      const rev = r.u64();\n      const epoch = r.u32();\n      const count = r.u32();\n      if (count > 8192) throw new Reject(`too many ops in one update: ${count}`);\n      const ops = [];\n      for (let i = 0; i < count; i++) {\n        const x = r.u32(), y = r.u32(), w = r.u32(), h = r.u32();\n        const opKind = r.u8();\n        if (opKind === OP_RECT) {\n          const n = r.allocLen(w * h * 3 + 16, 'rect payload');\n          ops.push({ kind: OP_RECT, x, y, w, h, payload: r.take(n).slice() });\n        } else if (opKind === OP_FILL) {\n          const n = r.allocLen(3, 'fill colour');\n          ops.push({ kind: OP_FILL, x, y, w, h, color: Array.from(r.take(n)) });\n        } else if (opKind === OP_COPY) {\n          const srcX = r.u32();\n          const srcY = r.u32();\n          ops.push({ kind: OP_COPY, x, y, w, h, srcX, srcY });\n        } else {\n          throw new Reject(`unknown op kind: 0x${opKind.toString(16)}`);\n        }\n      }\n      return { kind, rev, epoch, ops };\n    }\n    case K_KEEP_ALIVE:\n      return { kind, rev: r.u64() };\n    case K_QUALITY:\n      return { kind, targetFps: r.u32(), maxFps: r.u32(), quality: r.view.getFloat32(r.offset, true) };\n    case K_ERROR: {\n      const n = r.allocLen(1024, 'Error message');\n      return { kind, text: new TextDecoder().decode(r.take(n)) };\n    }\n    case K_BYE:\n      return { kind };\n    default:\n      throw new Reject(`unknown message kind: 0x${kind.toString(16)}`);\n  }\n}\n\nfunction encodeHello(token) {\n  const raw = new TextEncoder().encode(token);\n  const out = new Uint8Array(5 + 1 + 2 + raw.length);\n  const view = new DataView(out.buffer);\n  const bodyAt = 5;\n  out[0] = PROTOCOL_VERSION;\n  view.setUint32(1, 1 + 2 + raw.length, true);\n  out[bodyAt] = K_HELLO;\n  view.setUint16(bodyAt + 1, raw.length, true);\n  out.set(raw, bodyAt + 3);\n  return out;\n}\n\n// ------------------------------------------------------------- transport\n\nclass Session {\n  constructor(token) {\n    this.token = token;\n    this.compositor = new Compositor();\n    this.socket = null;\n    this.lastAck = 0;\n    this.paintScheduled = false;\n    this.setupCanvas();\n  }\n\n  setupCanvas() {\n    this.canvas = document.getElementById('screen');\n    this.ctx = this.canvas.getContext('2d');\n    this.compositor.onPaint = () => this.schedulePaint();\n  }\n\n  schedulePaint() {\n    if (this.paintScheduled) return;\n    this.paintScheduled = true;\n    requestAnimationFrame(() => {\n      this.paintScheduled = false;\n      this.paint();\n    });\n  }\n\n  paint() {\n    const c = this.compositor;\n    if (!c.fresh) return;\n    if (this.canvas.width !== c.width || this.canvas.height !== c.height) {\n      this.canvas.width = c.width;\n      this.canvas.height = c.height;\n    }\n    this.ctx.putImageData(new ImageData(new Uint8ClampedArray(c.buffer), c.width, c.height), 0, 0);\n    if (c.rev !== this.lastAck) {\n      this.lastAck = c.rev;\n      this.sendAck(c.rev);\n    }\n  }\n\n  status(text) {\n    const el = document.getElementById('status');\n    if (el) el.textContent = text;\n  }\n\n  connect() {\n    const proto = location.protocol === 'https:' ? 'wss' : 'ws';\n    const url = `${proto}://${location.host}/ws?token=${encodeURIComponent(this.token)}`;\n    const socket = new WebSocket(url);\n    socket.binaryType = 'arraybuffer';\n    this.socket = socket;\n\n    socket.onopen = () => {\n      this.status('connected');\n      socket.send(encodeHello(this.token));\n    };\n    // onerror is called with an Event, not a message; the detail lives on\n    // the event, so read it from there rather than treating the argument\n    // as a string.\n    socket.onerror = (event) => {\n      this.status(`connection error: ${(event && event.message) || 'unknown'}`);\n    };\n    socket.onclose = (event) => {\n      this.status(`disconnected (code ${event.code}); retrying in 2s`);\n      setTimeout(() => this.connect(), 2000);\n    };\n    socket.onmessage = (event) => this.onMessage(new Uint8Array(event.data));\n  }\n\n  sendAck(rev) {\n    if (this.socket && this.socket.readyState === WebSocket.OPEN) {\n      this.socket.send(encodeAck(rev));\n    }\n  }\n\n  async onMessage(bytes) {\n    let msg;\n    try {\n      msg = parseMessage(bytes);\n    } catch (e) {\n      this.status(`rejected: ${e.message}`);\n      return;\n    }\n    const c = this.compositor;\n    try {\n      switch (msg.kind) {\n        case K_SNAPSHOT_BEGIN:\n          c.beginSnapshot(msg.epoch, msg.width, msg.height, msg.format, msg.totalLen, msg.chunks);\n          break;\n        case K_SNAPSHOT_CHUNK:\n          c.pushSnapshotChunk(msg.index, msg.data);\n          break;\n        case K_SNAPSHOT_COMMIT:\n          await c.commitSnapshot(msg.rev, msg.epoch);\n          this.status(`${c.width}x${c.height} rev ${c.rev}`);\n          break;\n        case K_PARTIAL_UPDATE:\n          c.applyOps(msg.rev, msg.epoch, msg.ops);\n          break;\n        case K_KEEP_ALIVE:\n        case K_QUALITY:\n          break;\n        case K_ERROR:\n          this.status(msg.text);\n          break;\n        case K_BYE:\n          this.status('the sharer ended the session');\n          this.socket.close();\n          break;\n        default:\n          break;\n      }\n    } catch (e) {\n      if (e instanceof Rejected) {\n        // A revision we already hold is the expected consequence of\n        // joining mid-stream: the join snapshot covers it. Only a missing\n        // base is worth repairing.\n        if (e.reason === 'needs-snapshot' || e.reason === 'stale-epoch') {\n          this.status(`repairing: ${e.reason}`);\n          this.requestKeyframe();\n        }\n      } else {\n        this.status(`invalid update: ${e.message}`);\n        this.requestKeyframe();\n      }\n    }\n  }\n\n  requestKeyframe() {\n    if (this.socket && this.socket.readyState === WebSocket.OPEN) {\n      this.socket.send(encodeRequestKeyframe());\n    }\n  }\n}\n\nfunction envelope(body) {\n  const out = new Uint8Array(5 + body.length);\n  out[0] = PROTOCOL_VERSION;\n  new DataView(out.buffer).setUint32(1, body.length, true);\n  out.set(body, 5);\n  return out;\n}\n\nfunction encodeAck(rev) {\n  const body = new Uint8Array(9);\n  const view = new DataView(body.buffer);\n  body[0] = K_ACK;\n  view.setBigUint64(1, BigInt(rev), true);\n  return envelope(body);\n}\n\nfunction encodeRequestKeyframe() {\n  return envelope(new Uint8Array([K_REQUEST_KEYFRAME]));\n}\n\nwindow.pcc = {\n  connect(token) {\n    const session = new Session(token);\n    session.connect();\n    return session;\n  },\n  // Exposed so the same parser and the same LZ4 decoder can be exercised\n  // by a test page without a live sharer.\n  parseMessage,\n  lz4Decode,\n  Compositor,\n  PROTOCOL_VERSION,\n};\n","usedDeprecatedRules":[]}]
```

### knip

```text
Run `knip --help` or visit https://knip.dev for help

[stderr]
ERROR: Unable to find package.json
```

### madge

```text
Processed 1 file (1.2s) 



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
  2:52      error    no new line character at the end of file  (new-line-at-end-of-file)

.scc/config.yaml
  4:3       error    wrong indentation: expected 4 but found 2  (indentation)
```
