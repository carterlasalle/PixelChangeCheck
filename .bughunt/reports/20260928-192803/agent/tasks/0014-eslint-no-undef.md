# Task 0014: oxlint eslint(no-undef)

**Occurrences:** 17
**Severity:** warning
**Signal key:** `oxlint:eslint(no-undef)`

## Representative message

'location' is not defined.

## Locations

- `BH-9F840E446F36BA51` — `src/server/renderer/client.js:?:?:`
- `BH-9F840E446F36BA51` — `src/server/renderer/client.js:?:?:`
- `BH-DF140AD0A3B9B32F` — `src/server/renderer/client.js:?:?:`
- `BH-E006C52C8E0C3D1A` — `src/server/renderer/client.js:?:?:`
- `BH-18F1BD3D731C1FDF` — `src/server/renderer/client.js:?:?:`
- `BH-18F1BD3D731C1FDF` — `src/server/renderer/client.js:?:?:`
- `BH-18F1BD3D731C1FDF` — `src/server/renderer/client.js:?:?:`
- `BH-2A7DAD091CACD33F` — `src/server/renderer/client.js:?:?:`
- `BH-069DF221D199C422` — `src/server/renderer/client.js:?:?:`
- `BH-334C06DB21A13A74` — `src/server/renderer/client.js:?:?:`
- `BH-52AD747F4C7B0B19` — `src/server/renderer/client.js:?:?:`
- `BH-52AD747F4C7B0B19` — `src/server/renderer/client.js:?:?:`
- `BH-52AD747F4C7B0B19` — `src/server/renderer/client.js:?:?:`
- `BH-473942BA7A693751` — `src/server/renderer/client.js:?:?:`
- `BH-7B36C35CD86B21B7` — `src/server/renderer/client.js:?:?:`
- `BH-28BB56463A161668` — `src/server/renderer/client.js:?:?:`
- `BH-FF29A73AB7B354DB` — `src/server/renderer/client.js:?:?:`

## Repair protocol

Fix the root cause, run the verification command for the affected analyzer, then re-run BugHunt to refresh the queue.

```bash
/Users/rocket/pixelchangecheck/node_modules/.bin/oxlint --format=json --deny-warnings --ignore-pattern=node_modules --ignore-pattern=dist --ignore-pattern=build --ignore-pattern=.next --ignore-pattern=.bughunt --ignore-pattern=.venv --ignore-pattern=venv --ignore-pattern=env --ignore-pattern=.direnv --ignore-pattern=.git --ignore-pattern=.hg --ignore-pattern=.svn --ignore-pattern=.tox --ignore-pattern=.nox --ignore-pattern=.mypy_cache --ignore-pattern=.pytest_cache --ignore-pattern=.ruff_cache --ignore-pattern=.pyre --ignore-pattern=.coverage --ignore-pattern=htmlcov --ignore-pattern=mutants --ignore-pattern=__pypackages__ --ignore-pattern=site-packages --ignore-pattern=.agents --ignore-pattern=.claude --ignore-pattern=.codex --ignore-pattern=.pi --ignore-pattern=.omp --ignore-pattern=.hermes --ignore-pattern=.trace --ignore-pattern=.benchmarks --ignore-pattern=.complexipy_cache --ignore-pattern=.import_linter_cache --ignore-pattern=**/dist --ignore-pattern=**/build --ignore-pattern=**/.next --ignore-pattern=**/node_modules --ignore-pattern=**/.venv --ignore-pattern=**/venv --ignore-pattern=**/coverage --ignore-pattern=**/htmlcov --config /Users/rocket/pixelchangecheck/.bughunt/configs/oxlintrc.json
```
