# Task 0030: oxlint import(unambiguous)

**Occurrences:** 1
**Severity:** warning
**Signal key:** `oxlint:import(unambiguous)`

## Representative message

This module could be mistakenly parsed as script instead of module

## Locations

- `BH-9881F18300301C12` — `src/server/renderer/client.js:?:?:`

## Repair protocol

Fix the root cause, run the verification command for the affected analyzer, then re-run BugHunt to refresh the queue.

```bash
/Users/rocket/pixelchangecheck/node_modules/.bin/oxlint --format=json --deny-warnings --ignore-pattern=node_modules --ignore-pattern=dist --ignore-pattern=build --ignore-pattern=.next --ignore-pattern=.bughunt --ignore-pattern=.venv --ignore-pattern=venv --ignore-pattern=env --ignore-pattern=.direnv --ignore-pattern=.git --ignore-pattern=.hg --ignore-pattern=.svn --ignore-pattern=.tox --ignore-pattern=.nox --ignore-pattern=.mypy_cache --ignore-pattern=.pytest_cache --ignore-pattern=.ruff_cache --ignore-pattern=.pyre --ignore-pattern=.coverage --ignore-pattern=htmlcov --ignore-pattern=mutants --ignore-pattern=__pypackages__ --ignore-pattern=site-packages --ignore-pattern=.agents --ignore-pattern=.claude --ignore-pattern=.codex --ignore-pattern=.pi --ignore-pattern=.omp --ignore-pattern=.hermes --ignore-pattern=.trace --ignore-pattern=.benchmarks --ignore-pattern=.complexipy_cache --ignore-pattern=.import_linter_cache --ignore-pattern=**/dist --ignore-pattern=**/build --ignore-pattern=**/.next --ignore-pattern=**/node_modules --ignore-pattern=**/.venv --ignore-pattern=**/venv --ignore-pattern=**/coverage --ignore-pattern=**/htmlcov --config /Users/rocket/pixelchangecheck/.bughunt/configs/oxlintrc.json
```
