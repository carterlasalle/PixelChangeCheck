# Task 0001: eslint no-undef

**Occurrences:** 17
**Severity:** error
**Signal key:** `eslint:no-undef`

## Representative message

'document' is not defined.

## Locations

- `BH-442844C5E0F800CC` — `src/server/renderer/client.js:137:21:`
- `BH-E978965F5F67E680` — `src/server/renderer/client.js:188:24:`
- `BH-DEBE143C15DED943` — `src/server/renderer/client.js:189:28:`
- `BH-0EC0BD628C3EB543` — `src/server/renderer/client.js:193:23:`
- `BH-0B6A9C340807CC3A` — `src/server/renderer/client.js:348:32:`
- `BH-A8E315B9664ED7F1` — `src/server/renderer/client.js:358:19:`
- `BH-442844C5E0F800CC` — `src/server/renderer/client.js:383:19:`
- `BH-D7D0A91F9B426F2E` — `src/server/renderer/client.js:391:5:`
- `BH-7CFA34A2865BBB94` — `src/server/renderer/client.js:404:31:`
- `BH-442844C5E0F800CC` — `src/server/renderer/client.js:412:16:`
- `BH-D10B23A3D6562ADE` — `src/server/renderer/client.js:417:19:`
- `BH-D10B23A3D6562ADE` — `src/server/renderer/client.js:418:31:`
- `BH-01E3C22BBBE38E0B` — `src/server/renderer/client.js:419:24:`
- `BH-779EAA82D656D60C` — `src/server/renderer/client.js:435:7:`
- `BH-01E3C22BBBE38E0B` — `src/server/renderer/client.js:441:51:`
- `BH-01E3C22BBBE38E0B` — `src/server/renderer/client.js:500:51:`
- `BH-229054FF4267132E` — `src/server/renderer/client.js:526:1:`

## Repair protocol

Fix the root cause, run the verification command for the affected analyzer, then re-run BugHunt to refresh the queue.

```bash
/Users/rocket/pixelchangecheck/node_modules/.bin/eslint . --format json --config /Users/rocket/pixelchangecheck/.bughunt/configs/eslint.config.mjs
```
