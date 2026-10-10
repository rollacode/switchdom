# Rust core canonical gate — BLACKSTAR

Reviewed lane: `70e881760d9a2dcbd8eef7d6198af2b2d6268007`.
Real ancestry merge: `git merge --no-ff lane/pl-ujelxh-task-xgsqfj-28bb3a50`.
Verified code revision: `cb3b0332c5c042de0b31667775cafacb57f56ba5`.

Canonical verification on 2026-10-10, Windows x86_64, rustc 1.93.0:

- `cargo fmt -p layout-core -- --check`: exit 0.
- `cargo test -p layout-core -- --nocapture`: exit 0; 6 unit, 10 synthetic-profile and 1 curated-accuracy test passed.
- Curated accuracy: TP=5/5, FP=0/25, TN=25/25, FN=0/5. This is fixture behavior, not real-world calibration.
- `cargo run --release -p layout-core --bin decision-perf-oracle -- --corpus crates/layout-core/tests/fixtures/decision-perf.json --iterations 10000 --max-token-chars 96 --json`: exit 0; all five frozen fixtures matched, 50,000 samples, maximum 96 characters; median 0.700 us, p95 8.700 us, p99 8.800 us, limit 100 us.

Initial compilation exposed an unescaped opening brace in the oracle's JSON format string. The canonical mechanical repair escaped it and removed an unused assignment immediately before a return; focused gates above passed afterward. No correctness policy or fixture was changed. Subsequent ignore rules exclude `/target/` and retain Rust `src/bin/*.rs`, correcting inherited .NET ignore behavior.

Static review verified source-original assessment once, bounded target evaluation, generic multi-profile API, target confidence/source improvement/runner separation, first-party bigram model without per-example branches, MIT mapping provenance, and default textless inert advisor with no provider or OS authority. Benchmark-only `rustc --version` subprocess supplies metadata; the library has no runtime network or shell execution.

Limits: hand-authored bigrams and 30 curated labels are preliminary. Only RU/EN data ship. Three-profile tests prove the contract, not another supported language. Windows hook, undo, focus/password safeguards, tray and installation remain unverified and unimplemented at this milestone. No global keyboard hook was started.
