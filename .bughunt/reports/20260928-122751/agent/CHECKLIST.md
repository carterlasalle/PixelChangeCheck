# BugHunt Repair Checklist

Use this checklist before weakening any defense.

- [ ] Read `RISK_MAP.md` and start with uncovered/complex/high-agreement code.
- [ ] Read `DEDUPLICATED_QUEUE.md`; fix one logical issue rather than independently chasing duplicate analyzer messages.
- [ ] Apply only deterministic **safe** autofixes first; review unsafe/review fixes.
- [ ] Re-run the narrow verification command after each root-cause fix.
- [ ] Re-run relevant tests under the canonical seed and at least one randomized seed.
- [ ] Treat branch coverage gaps, seam drift, migration/API drift, and mutation survivors as different bug classes.
- [ ] If CI, an analyzer, fuzzing, concurrency, or environment-matrix execution hangs or behaves inconsistently, use the `ci-fix-dont-freeze` skill before weakening, skipping, quarantining, or disabling it.
- [ ] If a real escaped bug is confirmed, add a regression/property/detector and teach Bug Corpus when available.
