# Task 0013: lizard LIZARD_CCN

**Occurrences:** 30
**Severity:** warning
**Signal key:** `lizard:LIZARD_CCN`

## Representative message

receive_once has 177 NLOC, 33 CCN, 1139 token, 3 PARAM, 204 length, 0 ND

## Locations

- `BH-161A4B78F1E08411` — `src/app/share.rs:148:?:`
- `BH-2039419E579FD5E6` — `src/app/share.rs:319:?:`
- `BH-FFEFB93B834E9D10` — `src/app/share.rs:490:?:`
- `BH-B0598EFCA2F69C30` — `src/app/share.rs:761:?:`
- `BH-93903C0364AF2C55` — `src/app/view.rs:185:?:`
- `BH-204E753A08C85D13` — `src/audio/playout.rs:116:?:`
- `BH-75B6EA5BF08776C7` — `src/encoder/mod.rs:81:?:`
- `BH-26ACBEAC796CD5C6` — `src/main.rs:184:?:`
- `BH-284D4343F6622EC7` — `src/network/protocol.rs:185:?:`
- `BH-A19266937FFA9FB5` — `src/network/protocol.rs:413:?:`
- `BH-F7A620F417529B9B` — `src/network/protocol.rs:522:?:`
- `BH-A11AF4AAD82EFCFA` — `src/network/wire.rs:129:?:`
- `BH-D1A116DCF78BD907` — `src/pcc/compositor.rs:212:?:`
- `BH-15A5192DAD0A0BE6` — `src/pcc/compositor.rs:379:?:`
- `BH-2CCFDA0D0D8F2B1A` — `src/pcc/detector.rs:147:?:`
- `BH-B45136FA8482B143` — `src/pcc/detector.rs:465:?:`
- `BH-4B47EFEB3FCEB2C1` — `src/pcc/planner.rs:132:?:`
- `BH-AC2617C20C0C131C` — `src/pcc/planner.rs:289:?:`
- `BH-FC3B755BB0255BB4` — `src/pcc/planner.rs:365:?:`
- `BH-7F5239537BBBF2EE` — `src/pcc/types.rs:203:?:`
- `BH-11B7A076EA0F0E09` — `src/reach/mod.rs:138:?:`
- `BH-60F986CB594AE8C9` — `src/reach/mod.rs:327:?:`
- `BH-817E2399756FFDE2` — `src/relay.rs:119:?:`
- `BH-F375F7D38217BE9C` — `src/relay.rs:293:?:`
- `BH-7D5EB110F217DE1D` — `src/server/renderer/client.js:79:?:`
- `BH-857556D0A10D53F4` — `src/server/renderer/client.js:212:?:`
- `BH-8D59C9E0CB94EC5A` — `src/server/renderer/client.js:267:?:`
- `BH-D61193B1CADB1F8D` — `src/server/renderer/client.js:286:?:`
- `BH-FED69D0D537CCE8A` — `src/server/renderer/web.rs:456:?:`
- `BH-DBC7B4C3909800A3` — `src/telemetry/mod.rs:160:?:`

## Repair protocol

Fix the root cause, run the verification command for the affected analyzer, then re-run BugHunt to refresh the queue.

```bash
/Users/rocket/bughunt/.venv/bin/lizard -w -C 10 -L 80 -a 8 -t 6 src
```
