# Task 0031: scc scc:unresolved-references

**Occurrences:** 1
**Severity:** warning
**Signal key:** `scc:scc:unresolved-references`

## Representative message

scc left 1788 likely-internal call/reference edges unresolved (resolved=895); graph-based seam/contract conclusions are partial over these edges

## Locations

- `BH-2A62F0022F311771` — `<unknown>:?:?:`

## Repair protocol

Fix the root cause, run the verification command for the affected analyzer, then re-run BugHunt to refresh the queue.

```bash
/Users/rocket/.local/share/uv/tools/bughunt/bin/python -m bughunt.system_ir_adapter /Users/rocket/pixelchangecheck
```
