# Task 0004: eslint no-unused-vars

**Occurrences:** 1
**Severity:** error
**Signal key:** `eslint:no-unused-vars`

## Representative message

'FMT_PNG' is assigned a value but never used.

## Locations

- `BH-F9EE3C2A97D02E90` — `src/server/renderer/client.js:33:7:`

## Repair protocol

Fix the root cause, run the verification command for the affected analyzer, then re-run BugHunt to refresh the queue.

```bash
/Users/rocket/pixelchangecheck/node_modules/.bin/eslint . --format json --config /Users/rocket/pixelchangecheck/.bughunt/configs/eslint.config.mjs
```
