# BugHunt Fix Queue

Generated `2026-09-28T12:27:51.555607-04:00` from profile `all`.

**104 findings** across **33 repeated-signal groups**.
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
| 5 | 1 | `knip:error: unable to find package.json` | ERROR: Unable to find package.json |
| 6 | 1 | `oxlint:eslint(no-unused-vars)` | Variable 'FMT_PNG' is declared but never used. Unused variables should start with a '_'. |
| 7 | 1 | `oxlint:typescript(no-extraneous-class)` | Unexpected class with only a constructor. |
| 8 | 1 | `oxlint:unicorn(no-new-array)` | Do not use `new Array(singleArgument)`. |
| 9 | 1 | `shellcheck:SC1036` | '(' is invalid here. Did you forget to escape it? |
| 10 | 1 | `shellcheck:SC1072` | Expected end of $(..) expression. Fix any mentioned problems and try again. |
| 11 | 1 | `shellcheck:SC1073` | Couldn't parse this command expansion. Fix to allow more checks. |
| 12 | 1 | `yamllint:<n>:<n> error wrong indentation: expected <n> but found <n> (indentation)` | 4:3       error    wrong indentation: expected 4 but found 2  (indentation) |
| 13 | 17 | `oxlint:eslint(no-undef)` | 'location' is not defined. |
| 14 | 12 | `oxlint:eslint(no-plusplus)` | Unary operator '++' used. |
| 15 | 11 | `oxlint:eslint(no-implicit-globals)` | Unexpected function declaration in the global scope. |
| 16 | 6 | `oxlint:eslint(no-use-before-define)` | 'join' was used before it was defined. |
| 17 | 4 | `oxlint:eslint(no-bitwise)` | Unexpected use of `">>"`. |
| 18 | 3 | `oxlint:eslint(sort-vars)` | Variable declarations should be sorted |
| 19 | 2 | `hadolint:DL3008` | Pin versions in apt get install. Instead of `apt-get install <package>` use `apt-get install <package>=<version>` |
| 20 | 2 | `oxlint:eslint(max-lines-per-function)` | The function `parseMessage` has too many lines (70). Maximum allowed is 50. |
| 21 | 2 | `oxlint:eslint(no-undefined)` | Unexpected use of `undefined` |
| 22 | 2 | `oxlint:oxc(no-async-await)` | async is not allowed |
| 23 | 2 | `oxlint:unicorn(prefer-query-selector)` | Prefer `.querySelector()` over `.getElementById()`. |
| 24 | 1 | `hadolint:DL3025` | Use arguments JSON notation for CMD and ENTRYPOINT arguments |
| 25 | 1 | `oxlint:eslint(class-methods-use-this)` | Expected method `status` to have this. |
| 26 | 1 | `oxlint:eslint(max-classes-per-file)` | File has too many classes (5). Maximum allowed is 1 |
| 27 | 1 | `oxlint:eslint(max-lines)` | File has too many lines (538). |
| 28 | 1 | `oxlint:eslint(no-inline-comments)` | Unexpected comment inline with code |
| 29 | 1 | `oxlint:import(unambiguous)` | This module could be mistakenly parsed as script instead of module |
| 30 | 1 | `oxlint:oxc(no-rest-spread-properties)` | object spread property are not allowed.  |
| 31 | 1 | `scc:scc:unresolved-references` | scc left 1788 likely-internal call/reference edges unresolved (resolved=895); graph-based seam/contract conclusions are partial over these edges |
| 32 | 1 | `hadolint:DL3066` | Non-numeric user-id may not be resolvable by host system |
| 33 | 1 | `shellcheck:SC1009` | The mentioned syntax error was in this double quoted string. |

## One-by-one queue

- [ ] **1. `BH-E691EB215F2994FF`** `actionlint` — `<unknown>:?:?` — [{"message":"label \"macos-13\" is unknown. available labels are \"windows-latest\", \"windows-latest-8-cores\", \"windows-2025\", \"windows-2025-vs2026\", \"windows-2022\", \"windows-11-arm\", \"ubuntu-slim\", \"ubuntu-latest\", \"ubuntu-latest-4-cores\", \"ubuntu-latest-8-cores\", \"ubuntu-latest-16-cores\", \"ubuntu-24.04\", \"ubuntu-24.04-arm\", \"ubuntu-22.04\", \"ubuntu-22.04-arm\", \"macos-latest\", \"macos-latest-xlarge\", \"macos-latest-large\", \"macos-26-intel\", \"macos-26-xlarge\", \"macos-26-large\", \"macos-26\", \"macos-15-intel\", \"macos-15-xlarge\", \"macos-15-large\", \"macos-15\", \"macos-14-xlarge\", \"macos-14-large\", \"macos-14\", \"self-hosted\", \"x64\", \"arm\", \"arm64\", \"linux\", \"macos\", \"windows\". if it is a custom label for self-hosted runner, set list of labels in actionlint.yaml config file","filepath":".github/workflows/release.yml","line":37,"column":43,"kind":"runner-label","snippet":"        os: [ubuntu-latest, macos-latest, macos-13, window
- [ ] **2. `BH-A5609DE26B67C21B`** `knip` — `<unknown>:?:?` — ERROR: Unable to find package.json
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
- [ ] **32. `BH-2A62F0022F311771`** `scc/scc:unresolved-references` — `<unknown>:?:?` — scc left 1788 likely-internal call/reference edges unresolved (resolved=895); graph-based seam/contract conclusions are partial over these edges
- [ ] **33. `BH-69C8D25F56F669A2`** `hadolint/DL3008` — `Dockerfile:5:1` — Pin versions in apt get install. Instead of `apt-get install <package>` use `apt-get install <package>=<version>`
- [ ] **34. `BH-69C8D25F56F669A2`** `hadolint/DL3008` — `Dockerfile:15:1` — Pin versions in apt get install. Instead of `apt-get install <package>` use `apt-get install <package>=<version>`
- [ ] **35. `BH-74DD16E6C8721C49`** `hadolint/DL3025` — `Dockerfile:23:1` — Use arguments JSON notation for CMD and ENTRYPOINT arguments
- [ ] **36. `BH-9881F18300301C12`** `oxlint/import(unambiguous)` — `src/server/renderer/client.js:?:?` — This module could be mistakenly parsed as script instead of module
- [ ] **37. `BH-BA6B7D909DB906F7`** `oxlint/eslint(max-classes-per-file)` — `src/server/renderer/client.js:?:?` — File has too many classes (5). Maximum allowed is 1
- [ ] **38. `BH-058B7E6479AE1D87`** `oxlint/eslint(max-lines)` — `src/server/renderer/client.js:?:?` — File has too many lines (538).
- [ ] **39. `BH-F365632C92257DD2`** `oxlint/eslint(no-implicit-globals)` — `src/server/renderer/client.js:?:?` — Unexpected function declaration in the global scope.
- [ ] **40. `BH-F365632C92257DD2`** `oxlint/eslint(no-implicit-globals)` — `src/server/renderer/client.js:?:?` — Unexpected function declaration in the global scope.
- [ ] **41. `BH-F365632C92257DD2`** `oxlint/eslint(no-implicit-globals)` — `src/server/renderer/client.js:?:?` — Unexpected function declaration in the global scope.
- [ ] **42. `BH-F365632C92257DD2`** `oxlint/eslint(no-implicit-globals)` — `src/server/renderer/client.js:?:?` — Unexpected function declaration in the global scope.
- [ ] **43. `BH-F365632C92257DD2`** `oxlint/eslint(no-implicit-globals)` — `src/server/renderer/client.js:?:?` — Unexpected function declaration in the global scope.
- [ ] **44. `BH-F365632C92257DD2`** `oxlint/eslint(no-implicit-globals)` — `src/server/renderer/client.js:?:?` — Unexpected function declaration in the global scope.
- [ ] **45. `BH-F365632C92257DD2`** `oxlint/eslint(no-implicit-globals)` — `src/server/renderer/client.js:?:?` — Unexpected function declaration in the global scope.
- [ ] **46. `BH-F365632C92257DD2`** `oxlint/eslint(no-implicit-globals)` — `src/server/renderer/client.js:?:?` — Unexpected function declaration in the global scope.
- [ ] **47. `BH-F365632C92257DD2`** `oxlint/eslint(no-implicit-globals)` — `src/server/renderer/client.js:?:?` — Unexpected function declaration in the global scope.
- [ ] **48. `BH-F365632C92257DD2`** `oxlint/eslint(no-implicit-globals)` — `src/server/renderer/client.js:?:?` — Unexpected function declaration in the global scope.
- [ ] **49. `BH-F365632C92257DD2`** `oxlint/eslint(no-implicit-globals)` — `src/server/renderer/client.js:?:?` — Unexpected function declaration in the global scope.
- [ ] **50. `BH-221968370A461DA2`** `oxlint/eslint(no-inline-comments)` — `src/server/renderer/client.js:?:?` — Unexpected comment inline with code
- [ ] **51. `BH-9F840E446F36BA51`** `oxlint/eslint(no-undef)` — `src/server/renderer/client.js:?:?` — 'location' is not defined.
- [ ] **52. `BH-9F840E446F36BA51`** `oxlint/eslint(no-undef)` — `src/server/renderer/client.js:?:?` — 'location' is not defined.
- [ ] **53. `BH-DF140AD0A3B9B32F`** `oxlint/eslint(no-undef)` — `src/server/renderer/client.js:?:?` — 'TextEncoder' is not defined.
- [ ] **54. `BH-E006C52C8E0C3D1A`** `oxlint/eslint(no-undef)` — `src/server/renderer/client.js:?:?` — 'TextDecoder' is not defined.
- [ ] **55. `BH-18F1BD3D731C1FDF`** `oxlint/eslint(no-undef)` — `src/server/renderer/client.js:?:?` — 'WebSocket' is not defined.
- [ ] **56. `BH-18F1BD3D731C1FDF`** `oxlint/eslint(no-undef)` — `src/server/renderer/client.js:?:?` — 'WebSocket' is not defined.
- [ ] **57. `BH-18F1BD3D731C1FDF`** `oxlint/eslint(no-undef)` — `src/server/renderer/client.js:?:?` — 'WebSocket' is not defined.
- [ ] **58. `BH-2A7DAD091CACD33F`** `oxlint/eslint(no-undef)` — `src/server/renderer/client.js:?:?` — 'requestAnimationFrame' is not defined.
- [ ] **59. `BH-069DF221D199C422`** `oxlint/eslint(no-undef)` — `src/server/renderer/client.js:?:?` — 'createImageBitmap' is not defined.
- [ ] **60. `BH-334C06DB21A13A74`** `oxlint/eslint(no-undef)` — `src/server/renderer/client.js:?:?` — 'OffscreenCanvas' is not defined.
- [ ] **61. `BH-52AD747F4C7B0B19`** `oxlint/eslint(no-undef)` — `src/server/renderer/client.js:?:?` — 'document' is not defined.
- [ ] **62. `BH-52AD747F4C7B0B19`** `oxlint/eslint(no-undef)` — `src/server/renderer/client.js:?:?` — 'document' is not defined.
- [ ] **63. `BH-52AD747F4C7B0B19`** `oxlint/eslint(no-undef)` — `src/server/renderer/client.js:?:?` — 'document' is not defined.
- [ ] **64. `BH-473942BA7A693751`** `oxlint/eslint(no-undef)` — `src/server/renderer/client.js:?:?` — 'Blob' is not defined.
- [ ] **65. `BH-7B36C35CD86B21B7`** `oxlint/eslint(no-undef)` — `src/server/renderer/client.js:?:?` — 'ImageData' is not defined.
- [ ] **66. `BH-28BB56463A161668`** `oxlint/eslint(no-undef)` — `src/server/renderer/client.js:?:?` — 'setTimeout' is not defined.
- [ ] **67. `BH-FF29A73AB7B354DB`** `oxlint/eslint(no-undef)` — `src/server/renderer/client.js:?:?` — 'window' is not defined.
- [ ] **68. `BH-9D54B0E8E4846BB4`** `oxlint/eslint(no-plusplus)` — `src/server/renderer/client.js:?:?` — Unary operator '++' used.
- [ ] **69. `BH-775675C146C7E0DA`** `oxlint/eslint(sort-vars)` — `src/server/renderer/client.js:?:?` — Variable declarations should be sorted
- [ ] **70. `BH-9D54B0E8E4846BB4`** `oxlint/eslint(no-plusplus)` — `src/server/renderer/client.js:?:?` — Unary operator '++' used.
- [ ] **71. `BH-EF9A13050E6C505A`** `oxlint/eslint(no-bitwise)` — `src/server/renderer/client.js:?:?` — Unexpected use of `">>"`.
- [ ] **72. `BH-9D54B0E8E4846BB4`** `oxlint/eslint(no-plusplus)` — `src/server/renderer/client.js:?:?` — Unary operator '++' used.
- [ ] **73. `BH-A1DB7EF4C1919AFC`** `oxlint/eslint(no-bitwise)` — `src/server/renderer/client.js:?:?` — Unexpected use of `"|"`.
- [ ] **74. `BH-D4004F3E52B67ACB`** `oxlint/eslint(no-bitwise)` — `src/server/renderer/client.js:?:?` — Unexpected use of `"<<"`.
- [ ] **75. `BH-820E23BFB99D04B8`** `oxlint/eslint(no-bitwise)` — `src/server/renderer/client.js:?:?` — Unexpected use of `"&"`.
- [ ] **76. `BH-9D54B0E8E4846BB4`** `oxlint/eslint(no-plusplus)` — `src/server/renderer/client.js:?:?` — Unary operator '++' used.
- [ ] **77. `BH-9D54B0E8E4846BB4`** `oxlint/eslint(no-plusplus)` — `src/server/renderer/client.js:?:?` — Unary operator '++' used.
- [ ] **78. `BH-9D54B0E8E4846BB4`** `oxlint/eslint(no-plusplus)` — `src/server/renderer/client.js:?:?` — Unary operator '++' used.
- [ ] **79. `BH-9D54B0E8E4846BB4`** `oxlint/eslint(no-plusplus)` — `src/server/renderer/client.js:?:?` — Unary operator '++' used.
- [ ] **80. `BH-2B475FC7B6315C2D`** `oxlint/eslint(no-undefined)` — `src/server/renderer/client.js:?:?` — Unexpected use of `undefined`
- [ ] **81. `BH-348F358994D72353`** `oxlint/oxc(no-async-await)` — `src/server/renderer/client.js:?:?` — async is not allowed
- [ ] **82. `BH-2B475FC7B6315C2D`** `oxlint/eslint(no-undefined)` — `src/server/renderer/client.js:?:?` — Unexpected use of `undefined`
- [ ] **83. `BH-32E00442ABED73B1`** `oxlint/eslint(no-use-before-define)` — `src/server/renderer/client.js:?:?` — 'join' was used before it was defined.
- [ ] **84. `BH-A4B4604200E06339`** `oxlint/oxc(no-rest-spread-properties)` — `src/server/renderer/client.js:?:?` — object spread property are not allowed. 
- [ ] **85. `BH-CA0C4D415D6C58E7`** `oxlint/eslint(no-use-before-define)` — `src/server/renderer/client.js:?:?` — 'blit' was used before it was defined.
- [ ] **86. `BH-607297B9C7FE47B5`** `oxlint/eslint(no-use-before-define)` — `src/server/renderer/client.js:?:?` — 'fill' was used before it was defined.
- [ ] **87. `BH-4C323958A0C748BB`** `oxlint/eslint(no-use-before-define)` — `src/server/renderer/client.js:?:?` — 'blitRegion' was used before it was defined.
- [ ] **88. `BH-9D54B0E8E4846BB4`** `oxlint/eslint(no-plusplus)` — `src/server/renderer/client.js:?:?` — Unary operator '++' used.
- [ ] **89. `BH-9D54B0E8E4846BB4`** `oxlint/eslint(no-plusplus)` — `src/server/renderer/client.js:?:?` — Unary operator '++' used.
- [ ] **90. `BH-9D54B0E8E4846BB4`** `oxlint/eslint(no-plusplus)` — `src/server/renderer/client.js:?:?` — Unary operator '++' used.
- [ ] **91. `BH-9D54B0E8E4846BB4`** `oxlint/eslint(no-plusplus)` — `src/server/renderer/client.js:?:?` — Unary operator '++' used.
- [ ] **92. `BH-E8C2F1A6730C5798`** `oxlint/eslint(max-lines-per-function)` — `src/server/renderer/client.js:?:?` — The function `parseMessage` has too many lines (70). Maximum allowed is 50.
- [ ] **93. `BH-9D54B0E8E4846BB4`** `oxlint/eslint(no-plusplus)` — `src/server/renderer/client.js:?:?` — Unary operator '++' used.
- [ ] **94. `BH-775675C146C7E0DA`** `oxlint/eslint(sort-vars)` — `src/server/renderer/client.js:?:?` — Variable declarations should be sorted
- [ ] **95. `BH-775675C146C7E0DA`** `oxlint/eslint(sort-vars)` — `src/server/renderer/client.js:?:?` — Variable declarations should be sorted
- [ ] **96. `BH-534A1CFE8B2CE037`** `oxlint/unicorn(prefer-query-selector)` — `src/server/renderer/client.js:?:?` — Prefer `.querySelector()` over `.getElementById()`.
- [ ] **97. `BH-2752E00BE3332DF2`** `oxlint/eslint(class-methods-use-this)` — `src/server/renderer/client.js:?:?` — Expected method `status` to have this.
- [ ] **98. `BH-534A1CFE8B2CE037`** `oxlint/unicorn(prefer-query-selector)` — `src/server/renderer/client.js:?:?` — Prefer `.querySelector()` over `.getElementById()`.
- [ ] **99. `BH-F3D8F5765AD53525`** `oxlint/eslint(no-use-before-define)` — `src/server/renderer/client.js:?:?` — 'encodeAck' was used before it was defined.
- [ ] **100. `BH-8A1B62A4EC71713B`** `oxlint/eslint(max-lines-per-function)` — `src/server/renderer/client.js:?:?` — The async method `onMessage` has too many lines (52). Maximum allowed is 50.
- [ ] **101. `BH-348F358994D72353`** `oxlint/oxc(no-async-await)` — `src/server/renderer/client.js:?:?` — async is not allowed
- [ ] **102. `BH-FDB09C23C5592E0B`** `oxlint/eslint(no-use-before-define)` — `src/server/renderer/client.js:?:?` — 'encodeRequestKeyframe' was used before it was defined.
- [ ] **103. `BH-0D65D4FE005D8915`** `hadolint/DL3066` — `Dockerfile:21:1` — Non-numeric user-id may not be resolvable by host system
- [ ] **104. `BH-9869B909107784CB`** `shellcheck/SC1009` — `deploy/relay/provision-oracle.sh:182:15` — The mentioned syntax error was in this double quoted string.
