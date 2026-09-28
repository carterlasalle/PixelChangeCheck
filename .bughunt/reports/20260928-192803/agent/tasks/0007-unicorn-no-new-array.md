# Task 0007: oxlint unicorn(no-new-array)

**Occurrences:** 1
**Severity:** error
**Signal key:** `oxlint:unicorn(no-new-array)`

## Representative message

Do not use `new Array(singleArgument)`.

## Locations

- `BH-E105D5995218A028` — `src/server/renderer/client.js:?:?:`

## Repair protocol

Fix the root cause, run the verification command for the affected analyzer, then re-run BugHunt to refresh the queue.

```bash
/Users/rocket/pixelchangecheck/node_modules/.bin/oxlint --format=json --deny-warnings --ignore-pattern=node_modules --ignore-pattern=dist --ignore-pattern=build --ignore-pattern=.next --ignore-pattern=.bughunt --ignore-pattern=.venv --ignore-pattern=venv --ignore-pattern=env --ignore-pattern=.direnv --ignore-pattern=.git --ignore-pattern=.hg --ignore-pattern=.svn --ignore-pattern=.tox --ignore-pattern=.nox --ignore-pattern=.mypy_cache --ignore-pattern=.pytest_cache --ignore-pattern=.ruff_cache --ignore-pattern=.pyre --ignore-pattern=.coverage --ignore-pattern=htmlcov --ignore-pattern=mutants --ignore-pattern=__pypackages__ --ignore-pattern=site-packages --ignore-pattern=.agents --ignore-pattern=.claude --ignore-pattern=.codex --ignore-pattern=.pi --ignore-pattern=.omp --ignore-pattern=.hermes --ignore-pattern=.trace --ignore-pattern=.benchmarks --ignore-pattern=.complexipy_cache --ignore-pattern=.import_linter_cache --ignore-pattern=**/dist --ignore-pattern=**/build --ignore-pattern=**/.next --ignore-pattern=**/node_modules --ignore-pattern=**/.venv --ignore-pattern=**/venv --ignore-pattern=**/coverage --ignore-pattern=**/htmlcov --config /Users/rocket/pixelchangecheck/.bughunt/configs/oxlintrc.json
```
