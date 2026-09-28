# Task 0012: yamllint

**Occurrences:** 1
**Severity:** error
**Signal key:** `yamllint:<n>:<n> error wrong indentation: expected <n> but found <n> (indentation)`

## Representative message

4:3       error    wrong indentation: expected 4 but found 2  (indentation)

## Locations

- `BH-F02F2F421D4184BE` — `<unknown>:?:?:`

## Repair protocol

Fix the root cause, run the verification command for the affected analyzer, then re-run BugHunt to refresh the queue.

```bash
/opt/homebrew/bin/yamllint -d {extends: default, rules: {line-length: disable, truthy: disable, document-start: disable}} .github/dependabot.yml .github/workflows/ci.yml .github/workflows/release.yml .omp/config.yml .scc/config.yaml
```
