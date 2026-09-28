# Task 0003: actionlint

**Occurrences:** 1
**Severity:** error
**Signal key:** `actionlint:[{"message":"label <path> is unknown. available labels are <path> <path> <path> <path> <path> <path> <path> <path> <path> <path> <path> <path> <path> <path> <path> <path> <path> <p`

## Representative message

[{"message":"label \"macos-13\" is unknown. available labels are \"windows-latest\", \"windows-latest-8-cores\", \"windows-2025\", \"windows-2025-vs2026\", \"windows-2022\", \"windows-11-arm\", \"ubuntu-slim\", \"ubuntu-latest\", \"ubuntu-latest-4-cores\", \"ubuntu-latest-8-cores\", \"ubuntu-latest-16-cores\", \"ubuntu-24.04\", \"ubuntu-24.04-arm\", \"ubuntu-22.04\", \"ubuntu-22.04-arm\", \"macos-latest\", \"macos-latest-xlarge\", \"macos-latest-large\", \"macos-26-intel\", \"macos-26-xlarge\", \"macos-26-large\", \"macos-26\", \"macos-15-intel\", \"macos-15-xlarge\", \"macos-15-large\", \"macos-15\", \"macos-14-xlarge\", \"macos-14-large\", \"macos-14\", \"self-hosted\", \"x64\", \"arm\", \"arm64\", \"linux\", \"macos\", \"windows\". if it is a custom label for self-hosted runner, set list of labels in actionlint.yaml config file","filepath":".github/workflows/release.yml","line":37,"column":43,"kind":"runner-label","snippet":"        os: [ubuntu-latest, macos-latest, macos-13, window

## Locations

- `BH-E691EB215F2994FF` — `<unknown>:?:?:`

## Repair protocol

Fix the root cause, run the verification command for the affected analyzer, then re-run BugHunt to refresh the queue.

```bash
/opt/homebrew/bin/actionlint -format {{json .}} .github/workflows/ci.yml .github/workflows/release.yml
```
