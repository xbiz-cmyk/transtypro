# Handoff: QA Setup — phase/qa-setup

## Branch

`phase/qa-setup`

## Files created or modified

| File | Action |
|---|---|
| `docs/QA_CHECKLIST.md` | Modified — added Phase 1, Phase 2 backend contracts, Phase 2 storage, and general cross-agent sections |
| `scripts/quality-check.ps1` | Created — PowerShell quality gate script |

No source files (`src/**`, `src-tauri/src/**`) were touched.

---

## How to run the quality-check script

From the repo root on Windows with PowerShell 7 (pwsh):

```powershell
pwsh scripts/quality-check.ps1
```

The script:
- Automatically extends `$env:PATH` with `~/.cargo/bin` if present (handles the common Windows case where cargo is installed but not on the system PATH).
- Runs `npm run build`, `cargo fmt --check`, `cargo clippy`, `cargo test`, and `git status` in order.
- Prints a clearly labeled PASS or FAIL line after each check.
- Prints a final summary listing all failed checks.
- Exits with code 0 if all checks pass, code 1 if any check fails.

**Prerequisite:** `npm install` must have been run at least once before `npm run build`.

---

## Full output of quality-check.ps1 on this branch

```
======================================
  transtypro quality-check.ps1
======================================

==> npm run build
    PASS: npm run build

==> cargo fmt --check
    PASS: cargo fmt --check

==> cargo clippy
    Checking transtypro v0.1.0 (C:\Users\User\Desktop\transtypro-qa\src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 5.57s
    PASS: cargo clippy

==> cargo test
   Compiling transtypro v0.1.0 (C:\Users\User\Desktop\transtypro-qa\src-tauri)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 10.44s
     Running unittests src\lib.rs (...)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src\main.rs (...)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests transtypro_lib

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

    PASS: cargo test

==> git status
    PASS: git status

======================================
  SUMMARY
======================================
  All checks passed.
```

All checks passed: **yes**

---

## Known limitations

- The script uses no unit test framework for the frontend (Vitest is not configured yet). The `npm run build` check only verifies TypeScript compilation and Vite bundling.
- `cargo test` shows `0 tests` because no Rust unit tests exist yet. The test harness is ready; tests will be added in Phase 2 and beyond.
- `npm run lint` is not run because no lint script is configured in `package.json` at this time (no ESLint setup).
- The script targets Windows (`pwsh`). A `.sh` equivalent exists as `scripts/quality-check.example.sh` for reference on macOS/Linux, but the canonical gate for this project is the `.ps1` file.
- First run requires `npm install` to populate `node_modules/`. The script does not run `npm install` automatically to avoid unexpected side effects.

---

## Instructions for other agents

### When to use

Run `pwsh scripts/quality-check.ps1` from the repo root before every handoff:
- Before creating a PR.
- Before marking a phase complete.
- After any significant set of changes.

### Expected results

All four checks (npm build, cargo fmt, cargo clippy, cargo test) must show PASS.

### Failures

- `npm run build` failing: check TypeScript errors in the build output and fix type issues before continuing.
- `cargo fmt --check` failing: run `cargo fmt --manifest-path src-tauri/Cargo.toml` to auto-format, then re-run the check.
- `cargo clippy` failing: read the clippy warnings and address them. Do not suppress warnings with `#[allow(...)]` without a clear justification.
- `cargo test` failing: a Rust test has regressed. Fix the test or the implementation before continuing.

### Adding tests

As Phase 2 storage work proceeds, add Rust unit tests under `src-tauri/src/`. They will be picked up automatically by `cargo test`.

### Adding lint

Once ESLint is configured (a future task), add `npm run lint` as a step in the script by inserting a `Run-Check` call before or after the build step.
