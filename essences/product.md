# Switchdom product contract

```map
anchors:
  - core:switchdom
  - vector:intended-text
  - vector:local-control
  - vector:bounded-extension
  - vector:accountable-release
```

## Purpose and authority

Switchdom is a local Rust automatic keyboard-layout correction product for people typing Russian and English on Windows. It should repair confidently recognized wrong-layout text while preserving correct text and user control. Its public derivative is rollacode/switchdom. The owner determines intended behavior and support scope; implementation, test criteria and inherited documentation do not independently change that intent.

These essences are the normative product home. [Capabilities](capabilities.md) owns the inventory and status; [Core](core.md) owns enduring responsibility boundaries; this page owns detailed product obligations. Essence links remain within essences. Implementation references, executed evidence, procedures and decision lineage stay in the project's existing review and decision records outside this compact contract. The generated map is a retrieval aid; its journal records map changes rather than all product decisions.

The original need was to compare open Windows switcher sources, retain useful ideas and licensed history, remove demonstrated unsafe behavior, and deliver a working reversible local application. The owner's refinement makes the product Rust-only, emphasizes measured latency, preserves a generic language/layout architecture, and reserves future platform/advisor seams. Local operation, privacy, provenance and a working Windows installation remain required. Record comparison findings in the existing evidence record; support and privacy claims must distinguish observed findings from unverified assumptions and must not imply an absolute cleanliness guarantee.

## Correction policy

Use a known source layout and a generic active-profile collection, not a closed RU/EN enum or pairwise direction API. Profiles, mappings and models carry version and provenance. Only English QWERTY and Russian ЙЦУКЕН have concrete product data now. A synthetic third profile establishes API behavior only; it does not support another language or arbitrary installed layouts.

Retain at most 96 Unicode scalar values per token, accept at most eight active profiles and perform at most seven target evaluations. Assess the source once and each eligible target at most once. A correction requires discriminative lexical/bigram evidence, target score at least 0.85, improvement over source score at least 0.08, and separation from a runner-up of at least 0.08 when one exists. Scores are deterministic heuristics, not calibrated probabilities. Shape or vowel resemblance alone cannot authorize correction. Ties, insufficient evidence, unknown profiles, unsupported mappings and budget violations preserve input. Do not hide accuracy failures with case-specific blacklists, relabeling or weaker thresholds. User-authored local exceptions remain a separate user control.

The native adapter must establish that profile mappings correspond to real installed layouts and select the core-chosen target layout. Generic profile key IDs are not native keycodes. IME, dead-key, non-bijective or complex-script capabilities remain unsupported until explicitly authorized and evidenced.

## Native authority and field boundary

The first Windows support claim is limited to independently verified writable standard native single-line Edit controls, known safe capabilities and collapsed selection. Password, unknown/stale, multiline, Rich Edit, custom/browser/unverified controls and non-collapsed selections pass through with no retained token, analysis or application. CodeTerm, IDE/editors, terminals, games and anti-cheat contexts are excluded. A recognized game removes the hook; a game list cannot universally detect anti-cheat. Users must exit before games/anti-cheat.

Pending work loses authority on changed target, focus, caret, selection, source layout, intervening input, timeout or exceeded bounds. Clear transient state when context changes. Revalidate the exact field and insertion context immediately before replacement. Immediate undo restores only the replaced insertion and prior layout in unchanged context. Preserve punctuation and delimiters; newline passes through without synthesizing Enter or submitting a field.

Normal operation ignores injected events. Only explicit owned probe/self-test mode may accept private-tagged synthetic events after verifying its own process/window. Pause, exit and recognized game exclusion unhook and clear state. Input callbacks perform no disk/network/advisor/UIA work, logging, blocking lock or wait. Native authority must fail closed; unavailable knowledge grants no permission to capture or edit.

## Privacy and product scope

Runtime operation has no telemetry, updater, networking, clipboard access, shell execution or raw key/text logs. Token state is local, transient and bounded. Diagnostics contain reason/request/generation/event counts, never token payloads. Password state has zero retained characters and analyzer inputs. No validation captures real passwords or ordinary user typing.

Future advisory schemas are bounded and textless by default. No provider, transport or LLM integration is enabled now. Any future text sharing requires a separately authorized feature and explicit user-controlled opt-in. Advice runs outside callbacks, stale/late/mismatched responses are discarded, and advice has no OS application authority.

The intended Windows product offers a portable binary and optional reversible per-user install/uninstall under `%LOCALAPPDATA%/Programs/Switchdom`, with an optional ordinary shortcut. It changes no autostart, elevation, security settings, installed layouts or system hotkeys. Tray controls include enable/pause, active profiles, local exceptions, About and exit. Rust macOS and Linux adapters are future work; inherited non-Rust material does not establish Switchdom platform support.

## Provenance and public artifacts

Retain full upstream Git lineage, the upstream remote and applicable notices. The physical RU/EN mapping is adapted from MIT OopsLayout at a828e79c65a44e1842d3f687bd28ea3af44e0522. The accepted discriminator is first-party hand-authored bigram logic. FrequencyWords at 525f9b560de45753a5ea01069454e72e9aa541c6 distinguishes MIT code from CC-BY-SA-4.0 content; a code license cannot license copied lists or derived weights. No such lists or weights are included in the current Rust product. Future reuse requires provenance and the content's applicable obligations.

Public artifacts exclude credentials, real typing, private clipboard, personal settings/dictionaries, screenshots, full machine reports and personally identifying paths. Public evidence is reduced to synthetic outcomes, tool versions, platform class and measurements. Exact introduced commits/files receive release-owner review before a public push. A bounded hygiene scan is not an absolute guarantee. Publishing source is distinct from declaring a supported desktop release.

## Evidence and release acceptance

Accuracy and performance use separate frozen fixtures. The supplemental first-party synthetic corpus uses seed 20261010 and labels derived from intended text plus independently verified physical mapping, frozen before scoring. Required supported positives remain required; abstention counts as a false negative. Safety negatives must remain unchanged. Unsupported cases remain distinct. Report direction/category TP/FP/TN/FN with denominators, false-positive rate, recall and abstention reasons. Finite curated results do not claim real-world calibration. Preserve accepted core accuracy, performance and native case contracts; never tune them after observing failures.

Native evidence must exercise the production OS event path with private-tagged synthetic input in owned normal/secure controls and inspect actual displayed field text. Direct detector/apply calls or internal-state-only reports cannot prove native success. Cover both correction directions, correct-text preservation, punctuation/newline, rapid input/backspace, exact undo, caret/selection, same/cross-window focus, source-layout changes, password zero counts, unsupported/timeout pass-through, injected-input rejection and known-game/pause/exit unhook. A separate capability matrix discloses support limits. The coordinator is bounded to 30 seconds and always closes owned windows and removes hooks, including failures/timeouts.

Target core decision p99 is at most 100 microseconds on the designated Windows release machine. Real callback entry-to-return p99 is at most 2 milliseconds under rapid input through the 96-scalar bound. Core timing cannot prove callback latency. Release requires native cases, independent security review with no failed case or unresolved critical/high finding, final focused and whole-product verification, reversible packaging evidence, binary identity and installed owned-field self-test. Gate structure, commands and raw evidence belong outside essences.

## Actual versus planned

The accepted product baseline contains the bounded Rust decision core, RU/EN data, synthetic multi-profile coverage, platform interfaces, an inert advisory schema and read-only bounded mapping metadata. Existing canonical evidence establishes curated core correctness and core latency only. Separately reported cross-platform core results do not establish a native Mac adapter.

Windows hook authority, actual layout verification, targeted replacement/undo and their native evidence remain incomplete. Tray/configuration, packaging, installation and desktop release are planned. No installed Switchdom binary or native callback latency claim is established by this contract. Rust macOS/Linux adapters and LLM integration are not implemented. These limits remain until accepted source and executed evidence justify changing capability status; planned criteria alone never do.
