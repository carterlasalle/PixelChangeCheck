# BugHunt Fix Queue

Generated `2026-09-28T19:28:03.383799-04:00` from profile `all`.

**134 findings** across **34 repeated-signal groups**.
**0 deterministic auto-fixes available**: 0 safe, 0 unsafe, 0 review-required.

Read `AGENT_INSTRUCTIONS.md` and `CHECKLIST.md` first. `queue.json` is authoritative.
`DEDUPLICATED_QUEUE.md` is the preferred repair view; raw findings remain available for evidence.

Also inspect `BLIND_SPOTS.md`; analyzer execution errors should be repaired before claiming coverage.
Prioritize `RISK_MAP.md`, especially files combining uncovered branches, complexity, and cross-tool agreement.
If debugging a hanging/flaky/CI-dependent defense, use the `ci-fix-dont-freeze` skill before weakening it.

## Highest-priority repeated signals

| # | Count | Signal | Example |
|---:|---:|---|---|
| 1 | 17 | `eslint:no-undef` | 'document' is not defined. |
| 2 | 4 | `oxlint:unicorn(prefer-add-event-listener)` | Prefer `addEventListener()` over their `on`-function counterparts. |
| 3 | 1 | `actionlint:[{"message":"label <path> is unknown. available labels are <path> <path> <pa…` | [{"message":"label \"macos-13\" is unknown. available labels are \"windows-latest\", \"windows-latest-8-cores\", \"windows-2025\", \"windows-2025-vs2026\", \"windows-2022\", \"wind |
| 4 | 1 | `eslint:no-unused-vars` | 'FMT_PNG' is assigned a value but never used. |
| 5 | 1 | `oxlint:eslint(no-unused-vars)` | Variable 'FMT_PNG' is declared but never used. Unused variables should start with a '_'. |
| 6 | 1 | `oxlint:typescript(no-extraneous-class)` | Unexpected class with only a constructor. |
| 7 | 1 | `oxlint:unicorn(no-new-array)` | Do not use `new Array(singleArgument)`. |
| 8 | 1 | `scc:BHGRAPH001` | scc index failed (exit 1): error: index: store: sqlite: database is locked  |
| 9 | 1 | `shellcheck:SC1036` | '(' is invalid here. Did you forget to escape it? |
| 10 | 1 | `shellcheck:SC1072` | Expected end of $(..) expression. Fix any mentioned problems and try again. |
| 11 | 1 | `shellcheck:SC1073` | Couldn't parse this command expansion. Fix to allow more checks. |
| 12 | 1 | `yamllint:<n>:<n> error wrong indentation: expected <n> but found <n> (indentation)` | 4:3       error    wrong indentation: expected 4 but found 2  (indentation) |
| 13 | 30 | `lizard:LIZARD_CCN` | receive_once has 177 NLOC, 33 CCN, 1139 token, 3 PARAM, 204 length, 0 ND |
| 14 | 17 | `oxlint:eslint(no-undef)` | 'location' is not defined. |
| 15 | 12 | `oxlint:eslint(no-plusplus)` | Unary operator '++' used. |
| 16 | 11 | `oxlint:eslint(no-implicit-globals)` | Unexpected function declaration in the global scope. |
| 17 | 6 | `oxlint:eslint(no-use-before-define)` | 'join' was used before it was defined. |
| 18 | 4 | `oxlint:eslint(no-bitwise)` | Unexpected use of `">>"`. |
| 19 | 3 | `oxlint:eslint(sort-vars)` | Variable declarations should be sorted |
| 20 | 2 | `hadolint:DL3008` | Pin versions in apt get install. Instead of `apt-get install <package>` use `apt-get install <package>=<version>` |
| 21 | 2 | `oxlint:eslint(max-lines-per-function)` | The function `parseMessage` has too many lines (70). Maximum allowed is 50. |
| 22 | 2 | `oxlint:eslint(no-undefined)` | Unexpected use of `undefined` |
| 23 | 2 | `oxlint:oxc(no-async-await)` | async is not allowed |
| 24 | 2 | `oxlint:unicorn(prefer-query-selector)` | Prefer `.querySelector()` over `.getElementById()`. |
| 25 | 1 | `hadolint:DL3025` | Use arguments JSON notation for CMD and ENTRYPOINT arguments |
| 26 | 1 | `oxlint:eslint(class-methods-use-this)` | Expected method `status` to have this. |
| 27 | 1 | `oxlint:eslint(max-classes-per-file)` | File has too many classes (5). Maximum allowed is 1 |
| 28 | 1 | `oxlint:eslint(max-lines)` | File has too many lines (538). |
| 29 | 1 | `oxlint:eslint(no-inline-comments)` | Unexpected comment inline with code |
| 30 | 1 | `oxlint:import(unambiguous)` | This module could be mistakenly parsed as script instead of module |
| 31 | 1 | `oxlint:oxc(no-rest-spread-properties)` | object spread property are not allowed.  |
| 32 | 1 | `hadolint:DL3066` | Non-numeric user-id may not be resolvable by host system |
| 33 | 1 | `shellcheck:SC1009` | The mentioned syntax error was in this double quoted string. |
| 34 | 1 | `shellcheck:SC2086` | Double quote to prevent globbing and word splitting. |

## One-by-one queue

- [ ] **1. `BH-E691EB215F2994FF`** `actionlint` — `<unknown>:?:?` — [{"message":"label \"macos-13\" is unknown. available labels are \"windows-latest\", \"windows-latest-8-cores\", \"windows-2025\", \"windows-2025-vs2026\", \"windows-2022\", \"windows-11-arm\", \"ubuntu-slim\", \"ubuntu-latest\", \"ubuntu-latest-4-cores\", \"ubuntu-latest-8-cores\", \"ubuntu-latest-16-cores\", \"ubuntu-24.04\", \"ubuntu-24.04-arm\", \"ubuntu-22.04\", \"ubuntu-22.04-arm\", \"macos-latest\", \"macos-latest-xlarge\", \"macos-latest-large\", \"macos-26-intel\", \"macos-26-xlarge\", \"macos-26-large\", \"macos-26\", \"macos-15-intel\", \"macos-15-xlarge\", \"macos-15-large\", \"macos-15\", \"macos-14-xlarge\", \"macos-14-large\", \"macos-14\", \"self-hosted\", \"x64\", \"arm\", \"arm64\", \"linux\", \"macos\", \"windows\". if it is a custom label for self-hosted runner, set list of labels in actionlint.yaml config file","filepath":".github/workflows/release.yml","line":37,"column":43,"kind":"runner-label","snippet":"        os: [ubuntu-latest, macos-latest, macos-13, window
- [ ] **2. `BH-C5AC5B4CB3B55E5D`** `scc/BHGRAPH001` — `<unknown>:?:?` — scc index failed (exit 1): error: index: store: sqlite: database is locked

- [ ] **3. `BH-F02F2F421D4184BE`** `yamllint` — `<unknown>:?:?` — 4:3       error    wrong indentation: expected 4 but found 2  (indentation)
- [ ] **4. `BH-B9BDACF1820C06C9`** `shellcheck/SC1073` — `deploy/relay/provision-oracle.sh:182:72` — Couldn't parse this command expansion. Fix to allow more checks.
- [ ] **5. `BH-13EFABB6E6F20A4F`** `shellcheck/SC1036` — `deploy/relay/provision-oracle.sh:182:94` — '(' is invalid here. Did you forget to escape it?
- [ ] **6. `BH-73679292A497882F`** `shellcheck/SC1072` — `deploy/relay/provision-oracle.sh:182:94` — Expected end of $(..) expression. Fix any mentioned problems and try again.
- [ ] **7. `BH-F01ED09DE398FD59`** `oxlint/eslint(no-unused-vars)` — `src/server/renderer/client.js:?:?` — Variable 'FMT_PNG' is declared but never used. Unused variables should start with a '_'.
- [ ] **8. `BH-B21015E7721FFA3E`** `oxlint/typescript(no-extraneous-class)` — `src/server/renderer/client.js:?:?` — Unexpected class with only a constructor.
- [ ] **9. `BH-E105D5995218A028`** `oxlint/unicorn(no-new-array)` — `src/server/renderer/client.js:?:?` — Do not use `new Array(singleArgument)`.
- [ ] **10. `BH-3FE9863C39C16172`** `oxlint/unicorn(prefer-add-event-listener)` — `src/server/renderer/client.js:?:?` — Prefer `addEventListener()` over their `on`-function counterparts.
- [ ] **11. `BH-3FE9863C39C16172`** `oxlint/unicorn(prefer-add-event-listener)` — `src/server/renderer/client.js:?:?` — Prefer `addEventListener()` over their `on`-function counterparts.
- [ ] **12. `BH-3FE9863C39C16172`** `oxlint/unicorn(prefer-add-event-listener)` — `src/server/renderer/client.js:?:?` — Prefer `addEventListener()` over their `on`-function counterparts.
- [ ] **13. `BH-3FE9863C39C16172`** `oxlint/unicorn(prefer-add-event-listener)` — `src/server/renderer/client.js:?:?` — Prefer `addEventListener()` over their `on`-function counterparts.
- [ ] **14. `BH-F9EE3C2A97D02E90`** `eslint/no-unused-vars` — `src/server/renderer/client.js:33:7` — 'FMT_PNG' is assigned a value but never used.
- [ ] **15. `BH-442844C5E0F800CC`** `eslint/no-undef` — `src/server/renderer/client.js:137:21` — 'document' is not defined.
- [ ] **16. `BH-E978965F5F67E680`** `eslint/no-undef` — `src/server/renderer/client.js:188:24` — 'Blob' is not defined.
- [ ] **17. `BH-DEBE143C15DED943`** `eslint/no-undef` — `src/server/renderer/client.js:189:28` — 'createImageBitmap' is not defined.
- [ ] **18. `BH-0EC0BD628C3EB543`** `eslint/no-undef` — `src/server/renderer/client.js:193:23` — 'OffscreenCanvas' is not defined.
- [ ] **19. `BH-0B6A9C340807CC3A`** `eslint/no-undef` — `src/server/renderer/client.js:348:32` — 'TextDecoder' is not defined.
- [ ] **20. `BH-A8E315B9664ED7F1`** `eslint/no-undef` — `src/server/renderer/client.js:358:19` — 'TextEncoder' is not defined.
- [ ] **21. `BH-442844C5E0F800CC`** `eslint/no-undef` — `src/server/renderer/client.js:383:19` — 'document' is not defined.
- [ ] **22. `BH-D7D0A91F9B426F2E`** `eslint/no-undef` — `src/server/renderer/client.js:391:5` — 'requestAnimationFrame' is not defined.
- [ ] **23. `BH-7CFA34A2865BBB94`** `eslint/no-undef` — `src/server/renderer/client.js:404:31` — 'ImageData' is not defined.
- [ ] **24. `BH-442844C5E0F800CC`** `eslint/no-undef` — `src/server/renderer/client.js:412:16` — 'document' is not defined.
- [ ] **25. `BH-D10B23A3D6562ADE`** `eslint/no-undef` — `src/server/renderer/client.js:417:19` — 'location' is not defined.
- [ ] **26. `BH-D10B23A3D6562ADE`** `eslint/no-undef` — `src/server/renderer/client.js:418:31` — 'location' is not defined.
- [ ] **27. `BH-01E3C22BBBE38E0B`** `eslint/no-undef` — `src/server/renderer/client.js:419:24` — 'WebSocket' is not defined.
- [ ] **28. `BH-779EAA82D656D60C`** `eslint/no-undef` — `src/server/renderer/client.js:435:7` — 'setTimeout' is not defined.
- [ ] **29. `BH-01E3C22BBBE38E0B`** `eslint/no-undef` — `src/server/renderer/client.js:441:51` — 'WebSocket' is not defined.
- [ ] **30. `BH-01E3C22BBBE38E0B`** `eslint/no-undef` — `src/server/renderer/client.js:500:51` — 'WebSocket' is not defined.
- [ ] **31. `BH-229054FF4267132E`** `eslint/no-undef` — `src/server/renderer/client.js:526:1` — 'window' is not defined.
- [ ] **32. `BH-69C8D25F56F669A2`** `hadolint/DL3008` — `Dockerfile:5:1` — Pin versions in apt get install. Instead of `apt-get install <package>` use `apt-get install <package>=<version>`
- [ ] **33. `BH-69C8D25F56F669A2`** `hadolint/DL3008` — `Dockerfile:15:1` — Pin versions in apt get install. Instead of `apt-get install <package>` use `apt-get install <package>=<version>`
- [ ] **34. `BH-74DD16E6C8721C49`** `hadolint/DL3025` — `Dockerfile:23:1` — Use arguments JSON notation for CMD and ENTRYPOINT arguments
- [ ] **35. `BH-161A4B78F1E08411`** `lizard/LIZARD_CCN` — `src/app/share.rs:148:?` — run_share has 137 NLOC, 23 CCN, 905 token, 2 PARAM, 170 length, 0 ND
- [ ] **36. `BH-2039419E579FD5E6`** `lizard/LIZARD_CCN` — `src/app/share.rs:319:?` — start_web has 56 NLOC, 15 CCN, 387 token, 7 PARAM, 60 length, 0 ND
- [ ] **37. `BH-FFEFB93B834E9D10`** `lizard/LIZARD_CCN` — `src/app/share.rs:490:?` — serve_viewer has 151 NLOC, 17 CCN, 980 token, 8 PARAM, 168 length, 0 ND
- [ ] **38. `BH-B0598EFCA2F69C30`** `lizard/LIZARD_CCN` — `src/app/share.rs:761:?` — capture_loop has 184 NLOC, 36 CCN, 1406 token, 12 PARAM, 224 length, 0 ND
- [ ] **39. `BH-93903C0364AF2C55`** `lizard/LIZARD_CCN` — `src/app/view.rs:185:?` — receive_once has 177 NLOC, 33 CCN, 1139 token, 3 PARAM, 204 length, 0 ND
- [ ] **40. `BH-204E753A08C85D13`** `lizard/LIZARD_CCN` — `src/audio/playout.rs:116:?` — start_default_output has 38 NLOC, 15 CCN, 264 token, 1 PARAM, 38 length, 0 ND
- [ ] **41. `BH-75B6EA5BF08776C7`** `lizard/LIZARD_CCN` — `src/encoder/mod.rs:81:?` — decode_snapshot has 45 NLOC, 14 CCN, 277 token, 4 PARAM, 49 length, 0 ND
- [ ] **42. `BH-26ACBEAC796CD5C6`** `lizard/LIZARD_CCN` — `src/main.rs:184:?` — main has 130 NLOC, 24 CCN, 803 token, 0 PARAM, 138 length, 0 ND
- [ ] **43. `BH-284D4343F6622EC7`** `lizard/LIZARD_CCN` — `src/network/protocol.rs:185:?` — encode_body has 82 NLOC, 3 CCN, 647 token, 2 PARAM, 82 length, 0 ND
- [ ] **44. `BH-A19266937FFA9FB5`** `lizard/LIZARD_CCN` — `src/network/protocol.rs:413:?` — message has 105 NLOC, 43 CCN, 715 token, 1 PARAM, 108 length, 0 ND
- [ ] **45. `BH-F7A620F417529B9B`** `lizard/LIZARD_CCN` — `src/network/protocol.rs:522:?` — op has 56 NLOC, 15 CCN, 309 token, 1 PARAM, 59 length, 0 ND
- [ ] **46. `BH-A11AF4AAD82EFCFA`** `lizard/LIZARD_CCN` — `src/network/wire.rs:129:?` — validate has 47 NLOC, 18 CCN, 281 token, 3 PARAM, 49 length, 0 ND
- [ ] **47. `BH-D1A116DCF78BD907`** `lizard/LIZARD_CCN` — `src/pcc/compositor.rs:212:?` — apply_ops has 77 NLOC, 12 CCN, 573 token, 4 PARAM, 88 length, 0 ND
- [ ] **48. `BH-15A5192DAD0A0BE6`** `lizard/LIZARD_CCN` — `src/pcc/compositor.rs:379:?` — blit_region has 18 NLOC, 2 CCN, 139 token, 9 PARAM, 18 length, 0 ND
- [ ] **49. `BH-2CCFDA0D0D8F2B1A`** `lizard/LIZARD_CCN` — `src/pcc/detector.rs:147:?` — detect has 119 NLOC, 28 CCN, 902 token, 5 PARAM, 139 length, 0 ND
- [ ] **50. `BH-B45136FA8482B143`** `lizard/LIZARD_CCN` — `src/pcc/detector.rs:465:?` — verify_displacement_of has 34 NLOC, 11 CCN, 274 token, 10 PARAM, 36 length, 0 ND
- [ ] **51. `BH-4B47EFEB3FCEB2C1`** `lizard/LIZARD_CCN` — `src/pcc/planner.rs:132:?` — plan has 63 NLOC, 12 CCN, 416 token, 6 PARAM, 77 length, 0 ND
- [ ] **52. `BH-AC2617C20C0C131C`** `lizard/LIZARD_CCN` — `src/pcc/planner.rs:289:?` — check_shift has 51 NLOC, 12 CCN, 414 token, 10 PARAM, 60 length, 0 ND
- [ ] **53. `BH-FC3B755BB0255BB4`** `lizard/LIZARD_CCN` — `src/pcc/planner.rs:365:?` — subtract_region has 44 NLOC, 15 CCN, 358 token, 5 PARAM, 51 length, 0 ND
- [ ] **54. `BH-7F5239537BBBF2EE`** `lizard/LIZARD_CCN` — `src/pcc/types.rs:203:?` — capture_frame has 20 NLOC, 1 CCN, 48 token, 1 PARAM, 94 length, 0 ND
- [ ] **55. `BH-11B7A076EA0F0E09`** `lizard/LIZARD_CCN` — `src/reach/mod.rs:138:?` — classify has 24 NLOC, 17 CCN, 282 token, 1 PARAM, 27 length, 0 ND
- [ ] **56. `BH-60F986CB594AE8C9`** `lizard/LIZARD_CCN` — `src/reach/mod.rs:327:?` — diagnose has 65 NLOC, 12 CCN, 334 token, 1 PARAM, 77 length, 0 ND
- [ ] **57. `BH-817E2399756FFDE2`** `lizard/LIZARD_CCN` — `src/relay.rs:119:?` — connect has 45 NLOC, 14 CCN, 311 token, 6 PARAM, 51 length, 0 ND
- [ ] **58. `BH-F375F7D38217BE9C`** `lizard/LIZARD_CCN` — `src/relay.rs:293:?` — handle_client has 147 NLOC, 27 CCN, 981 token, 8 PARAM, 174 length, 0 ND
- [ ] **59. `BH-9881F18300301C12`** `oxlint/import(unambiguous)` — `src/server/renderer/client.js:?:?` — This module could be mistakenly parsed as script instead of module
- [ ] **60. `BH-BA6B7D909DB906F7`** `oxlint/eslint(max-classes-per-file)` — `src/server/renderer/client.js:?:?` — File has too many classes (5). Maximum allowed is 1
- [ ] **61. `BH-058B7E6479AE1D87`** `oxlint/eslint(max-lines)` — `src/server/renderer/client.js:?:?` — File has too many lines (538).
- [ ] **62. `BH-F365632C92257DD2`** `oxlint/eslint(no-implicit-globals)` — `src/server/renderer/client.js:?:?` — Unexpected function declaration in the global scope.
- [ ] **63. `BH-F365632C92257DD2`** `oxlint/eslint(no-implicit-globals)` — `src/server/renderer/client.js:?:?` — Unexpected function declaration in the global scope.
- [ ] **64. `BH-F365632C92257DD2`** `oxlint/eslint(no-implicit-globals)` — `src/server/renderer/client.js:?:?` — Unexpected function declaration in the global scope.
- [ ] **65. `BH-F365632C92257DD2`** `oxlint/eslint(no-implicit-globals)` — `src/server/renderer/client.js:?:?` — Unexpected function declaration in the global scope.
- [ ] **66. `BH-F365632C92257DD2`** `oxlint/eslint(no-implicit-globals)` — `src/server/renderer/client.js:?:?` — Unexpected function declaration in the global scope.
- [ ] **67. `BH-F365632C92257DD2`** `oxlint/eslint(no-implicit-globals)` — `src/server/renderer/client.js:?:?` — Unexpected function declaration in the global scope.
- [ ] **68. `BH-F365632C92257DD2`** `oxlint/eslint(no-implicit-globals)` — `src/server/renderer/client.js:?:?` — Unexpected function declaration in the global scope.
- [ ] **69. `BH-F365632C92257DD2`** `oxlint/eslint(no-implicit-globals)` — `src/server/renderer/client.js:?:?` — Unexpected function declaration in the global scope.
- [ ] **70. `BH-F365632C92257DD2`** `oxlint/eslint(no-implicit-globals)` — `src/server/renderer/client.js:?:?` — Unexpected function declaration in the global scope.
- [ ] **71. `BH-F365632C92257DD2`** `oxlint/eslint(no-implicit-globals)` — `src/server/renderer/client.js:?:?` — Unexpected function declaration in the global scope.
- [ ] **72. `BH-F365632C92257DD2`** `oxlint/eslint(no-implicit-globals)` — `src/server/renderer/client.js:?:?` — Unexpected function declaration in the global scope.
- [ ] **73. `BH-221968370A461DA2`** `oxlint/eslint(no-inline-comments)` — `src/server/renderer/client.js:?:?` — Unexpected comment inline with code
- [ ] **74. `BH-9F840E446F36BA51`** `oxlint/eslint(no-undef)` — `src/server/renderer/client.js:?:?` — 'location' is not defined.
- [ ] **75. `BH-9F840E446F36BA51`** `oxlint/eslint(no-undef)` — `src/server/renderer/client.js:?:?` — 'location' is not defined.
- [ ] **76. `BH-DF140AD0A3B9B32F`** `oxlint/eslint(no-undef)` — `src/server/renderer/client.js:?:?` — 'TextEncoder' is not defined.
- [ ] **77. `BH-E006C52C8E0C3D1A`** `oxlint/eslint(no-undef)` — `src/server/renderer/client.js:?:?` — 'TextDecoder' is not defined.
- [ ] **78. `BH-18F1BD3D731C1FDF`** `oxlint/eslint(no-undef)` — `src/server/renderer/client.js:?:?` — 'WebSocket' is not defined.
- [ ] **79. `BH-18F1BD3D731C1FDF`** `oxlint/eslint(no-undef)` — `src/server/renderer/client.js:?:?` — 'WebSocket' is not defined.
- [ ] **80. `BH-18F1BD3D731C1FDF`** `oxlint/eslint(no-undef)` — `src/server/renderer/client.js:?:?` — 'WebSocket' is not defined.
- [ ] **81. `BH-2A7DAD091CACD33F`** `oxlint/eslint(no-undef)` — `src/server/renderer/client.js:?:?` — 'requestAnimationFrame' is not defined.
- [ ] **82. `BH-069DF221D199C422`** `oxlint/eslint(no-undef)` — `src/server/renderer/client.js:?:?` — 'createImageBitmap' is not defined.
- [ ] **83. `BH-334C06DB21A13A74`** `oxlint/eslint(no-undef)` — `src/server/renderer/client.js:?:?` — 'OffscreenCanvas' is not defined.
- [ ] **84. `BH-52AD747F4C7B0B19`** `oxlint/eslint(no-undef)` — `src/server/renderer/client.js:?:?` — 'document' is not defined.
- [ ] **85. `BH-52AD747F4C7B0B19`** `oxlint/eslint(no-undef)` — `src/server/renderer/client.js:?:?` — 'document' is not defined.
- [ ] **86. `BH-52AD747F4C7B0B19`** `oxlint/eslint(no-undef)` — `src/server/renderer/client.js:?:?` — 'document' is not defined.
- [ ] **87. `BH-473942BA7A693751`** `oxlint/eslint(no-undef)` — `src/server/renderer/client.js:?:?` — 'Blob' is not defined.
- [ ] **88. `BH-7B36C35CD86B21B7`** `oxlint/eslint(no-undef)` — `src/server/renderer/client.js:?:?` — 'ImageData' is not defined.
- [ ] **89. `BH-28BB56463A161668`** `oxlint/eslint(no-undef)` — `src/server/renderer/client.js:?:?` — 'setTimeout' is not defined.
- [ ] **90. `BH-FF29A73AB7B354DB`** `oxlint/eslint(no-undef)` — `src/server/renderer/client.js:?:?` — 'window' is not defined.
- [ ] **91. `BH-9D54B0E8E4846BB4`** `oxlint/eslint(no-plusplus)` — `src/server/renderer/client.js:?:?` — Unary operator '++' used.
- [ ] **92. `BH-775675C146C7E0DA`** `oxlint/eslint(sort-vars)` — `src/server/renderer/client.js:?:?` — Variable declarations should be sorted
- [ ] **93. `BH-9D54B0E8E4846BB4`** `oxlint/eslint(no-plusplus)` — `src/server/renderer/client.js:?:?` — Unary operator '++' used.
- [ ] **94. `BH-EF9A13050E6C505A`** `oxlint/eslint(no-bitwise)` — `src/server/renderer/client.js:?:?` — Unexpected use of `">>"`.
- [ ] **95. `BH-9D54B0E8E4846BB4`** `oxlint/eslint(no-plusplus)` — `src/server/renderer/client.js:?:?` — Unary operator '++' used.
- [ ] **96. `BH-A1DB7EF4C1919AFC`** `oxlint/eslint(no-bitwise)` — `src/server/renderer/client.js:?:?` — Unexpected use of `"|"`.
- [ ] **97. `BH-D4004F3E52B67ACB`** `oxlint/eslint(no-bitwise)` — `src/server/renderer/client.js:?:?` — Unexpected use of `"<<"`.
- [ ] **98. `BH-820E23BFB99D04B8`** `oxlint/eslint(no-bitwise)` — `src/server/renderer/client.js:?:?` — Unexpected use of `"&"`.
- [ ] **99. `BH-9D54B0E8E4846BB4`** `oxlint/eslint(no-plusplus)` — `src/server/renderer/client.js:?:?` — Unary operator '++' used.
- [ ] **100. `BH-9D54B0E8E4846BB4`** `oxlint/eslint(no-plusplus)` — `src/server/renderer/client.js:?:?` — Unary operator '++' used.
- [ ] **101. `BH-9D54B0E8E4846BB4`** `oxlint/eslint(no-plusplus)` — `src/server/renderer/client.js:?:?` — Unary operator '++' used.
- [ ] **102. `BH-9D54B0E8E4846BB4`** `oxlint/eslint(no-plusplus)` — `src/server/renderer/client.js:?:?` — Unary operator '++' used.
- [ ] **103. `BH-2B475FC7B6315C2D`** `oxlint/eslint(no-undefined)` — `src/server/renderer/client.js:?:?` — Unexpected use of `undefined`
- [ ] **104. `BH-348F358994D72353`** `oxlint/oxc(no-async-await)` — `src/server/renderer/client.js:?:?` — async is not allowed
- [ ] **105. `BH-2B475FC7B6315C2D`** `oxlint/eslint(no-undefined)` — `src/server/renderer/client.js:?:?` — Unexpected use of `undefined`
- [ ] **106. `BH-32E00442ABED73B1`** `oxlint/eslint(no-use-before-define)` — `src/server/renderer/client.js:?:?` — 'join' was used before it was defined.
- [ ] **107. `BH-A4B4604200E06339`** `oxlint/oxc(no-rest-spread-properties)` — `src/server/renderer/client.js:?:?` — object spread property are not allowed. 
- [ ] **108. `BH-CA0C4D415D6C58E7`** `oxlint/eslint(no-use-before-define)` — `src/server/renderer/client.js:?:?` — 'blit' was used before it was defined.
- [ ] **109. `BH-607297B9C7FE47B5`** `oxlint/eslint(no-use-before-define)` — `src/server/renderer/client.js:?:?` — 'fill' was used before it was defined.
- [ ] **110. `BH-4C323958A0C748BB`** `oxlint/eslint(no-use-before-define)` — `src/server/renderer/client.js:?:?` — 'blitRegion' was used before it was defined.
- [ ] **111. `BH-9D54B0E8E4846BB4`** `oxlint/eslint(no-plusplus)` — `src/server/renderer/client.js:?:?` — Unary operator '++' used.
- [ ] **112. `BH-9D54B0E8E4846BB4`** `oxlint/eslint(no-plusplus)` — `src/server/renderer/client.js:?:?` — Unary operator '++' used.
- [ ] **113. `BH-9D54B0E8E4846BB4`** `oxlint/eslint(no-plusplus)` — `src/server/renderer/client.js:?:?` — Unary operator '++' used.
- [ ] **114. `BH-9D54B0E8E4846BB4`** `oxlint/eslint(no-plusplus)` — `src/server/renderer/client.js:?:?` — Unary operator '++' used.
- [ ] **115. `BH-E8C2F1A6730C5798`** `oxlint/eslint(max-lines-per-function)` — `src/server/renderer/client.js:?:?` — The function `parseMessage` has too many lines (70). Maximum allowed is 50.
- [ ] **116. `BH-9D54B0E8E4846BB4`** `oxlint/eslint(no-plusplus)` — `src/server/renderer/client.js:?:?` — Unary operator '++' used.
- [ ] **117. `BH-775675C146C7E0DA`** `oxlint/eslint(sort-vars)` — `src/server/renderer/client.js:?:?` — Variable declarations should be sorted
- [ ] **118. `BH-775675C146C7E0DA`** `oxlint/eslint(sort-vars)` — `src/server/renderer/client.js:?:?` — Variable declarations should be sorted
- [ ] **119. `BH-534A1CFE8B2CE037`** `oxlint/unicorn(prefer-query-selector)` — `src/server/renderer/client.js:?:?` — Prefer `.querySelector()` over `.getElementById()`.
- [ ] **120. `BH-2752E00BE3332DF2`** `oxlint/eslint(class-methods-use-this)` — `src/server/renderer/client.js:?:?` — Expected method `status` to have this.
- [ ] **121. `BH-534A1CFE8B2CE037`** `oxlint/unicorn(prefer-query-selector)` — `src/server/renderer/client.js:?:?` — Prefer `.querySelector()` over `.getElementById()`.
- [ ] **122. `BH-F3D8F5765AD53525`** `oxlint/eslint(no-use-before-define)` — `src/server/renderer/client.js:?:?` — 'encodeAck' was used before it was defined.
- [ ] **123. `BH-8A1B62A4EC71713B`** `oxlint/eslint(max-lines-per-function)` — `src/server/renderer/client.js:?:?` — The async method `onMessage` has too many lines (52). Maximum allowed is 50.
- [ ] **124. `BH-348F358994D72353`** `oxlint/oxc(no-async-await)` — `src/server/renderer/client.js:?:?` — async is not allowed
- [ ] **125. `BH-FDB09C23C5592E0B`** `oxlint/eslint(no-use-before-define)` — `src/server/renderer/client.js:?:?` — 'encodeRequestKeyframe' was used before it was defined.
- [ ] **126. `BH-7D5EB110F217DE1D`** `lizard/LIZARD_CCN` — `src/server/renderer/client.js:79:?` — lz4Decode has 31 NLOC, 14 CCN, 315 token, 2 PARAM, 32 length, 0 ND
- [ ] **127. `BH-857556D0A10D53F4`** `lizard/LIZARD_CCN` — `src/server/renderer/client.js:212:?` — applyOps has 31 NLOC, 18 CCN, 456 token, 3 PARAM, 35 length, 0 ND
- [ ] **128. `BH-8D59C9E0CB94EC5A`** `lizard/LIZARD_CCN` — `src/server/renderer/client.js:267:?` — blitRegion has 5 NLOC, 2 CCN, 61 token, 9 PARAM, 5 length, 0 ND
- [ ] **129. `BH-D61193B1CADB1F8D`** `lizard/LIZARD_CCN` — `src/server/renderer/client.js:286:?` — parseMessage has 68 NLOC, 18 CCN, 715 token, 1 PARAM, 68 length, 0 ND
- [ ] **130. `BH-FED69D0D537CCE8A`** `lizard/LIZARD_CCN` — `src/server/renderer/web.rs:456:?` — read_ws_frame has 39 NLOC, 12 CCN, 302 token, 1 PARAM, 39 length, 0 ND
- [ ] **131. `BH-DBC7B4C3909800A3`** `lizard/LIZARD_CCN` — `src/telemetry/mod.rs:160:?` — render_prometheus has 161 NLOC, 1 CCN, 603 token, 2 PARAM, 170 length, 0 ND
- [ ] **132. `BH-0D65D4FE005D8915`** `hadolint/DL3066` — `Dockerfile:21:1` — Non-numeric user-id may not be resolvable by host system
- [ ] **133. `BH-9869B909107784CB`** `shellcheck/SC1009` — `deploy/relay/provision-oracle.sh:182:15` — The mentioned syntax error was in this double quoted string.
- [ ] **134. `BH-9BF8BD064FC3605A`** `shellcheck/SC2086` — `scripts/smoke.sh:182:23` — Double quote to prevent globbing and word splitting.
