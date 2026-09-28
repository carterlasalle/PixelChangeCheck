# BugHunt Blind Spots

These defenses did not provide a trusted clean result. Fix execution errors first; configure/install skipped defenses when they are relevant to this repository.

If a defense is hanging, flaky, environment-dependent, or failing for unclear CI/infrastructure reasons, use the `ci-fix-dont-freeze` skill before weakening or disabling it.

## ERROR: knip

- Category: `js-ts-dead-contract`
- Reason: no trusted result
- Command: `/Users/rocket/pixelchangecheck/node_modules/.bin/knip --strict --config /Users/rocket/pixelchangecheck/.bughunt/configs/knip.json`

## SKIPPED: mutmut

- Category: `mutation`
- Reason: explicitly skipped by user

## SKIPPED: lizard

- Category: `cross-language-complexity`
- Reason: lizard not installed

## SKIPPED: semgrep

- Category: `semantic-static`
- Reason: semgrep not installed

## SKIPPED: schemathesis

- Category: `api-fuzz`
- Reason: no safe runnable API target discovered/configured

## SKIPPED: custom

- Category: `custom`
- Reason: no high-confidence repository-specific semantic campaign could be inferred

## SKIPPED: clippy

- Category: `rust-correctness`
- Reason: Rust detected but cargo/clippy is not installed

## SKIPPED: tsc

- Category: `ts-types`
- Reason: TypeScript detected but no root tsconfig.json project exists

## SKIPPED: publint

- Category: `package-correctness`
- Reason: publint is installed but package.json is missing name/version; add both fields to make the package publishable
