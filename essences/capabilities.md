# Switchdom capabilities

This is the intended product inventory. `stable` means an accepted bounded core contract, not a released desktop feature. `planned` carries no native support or installation claim. Detailed obligations are in [Product contract](product.md); responsibility boundaries are in [Core](core.md).

## 1. Preserve intended text {#vector:intended-text}

| ID | Capability | What it enables | Entry points | Invariants | Status | Runs on | Feeds |
| --- | --- | --- | --- | --- | --- | --- | --- |
| capability:layout-profiles | Versioned layout profiles | Represent several active languages and physical mappings without a binary language API | Bounded profile collection and known source layout identity | Real data currently cover English QWERTY and Russian ЙЦУКЕН only; synthetic profiles prove extensibility, not language support | stable (portable core only) | core:profiles | capability:correction-decision |
| capability:correction-decision | Conservative correction proposal | Rank possible intended text and abstain when evidence is insufficient | Local token decision using known source and active profiles | Bounded deterministic work; source comparison and discriminative evidence; a proposal grants no native edit authority | stable (portable core only) | core:decision | capability:native-correction, capability:advisor-seam, capability:platform-seams |
| capability:native-correction | Guarded Windows correction and undo | Correct a verified writable field and immediately restore the exact prior insertion | Eligible native field input; immediate undo | Current installed layout and exact target context must be verified; unsupported or stale state passes through; no submission | planned (native acceptance pending) | core:windows-authority | — (user-visible outcome) |

## 2. Keep control local {#vector:local-control}

| ID | Capability | What it enables | Entry points | Invariants | Status | Runs on | Feeds |
| --- | --- | --- | --- | --- | --- | --- | --- |
| capability:tray-control | Visible operation and exclusions | Enable, pause, choose active profiles, manage local exceptions, inspect About and exit | Windows tray and local configuration | Pause, exit and recognized game exclusion remove hook authority and clear state; no universal game detection claim | planned | core:desktop-product | capability:native-correction |
| capability:reversible-install | Reversible per-user installation | Run a portable binary or install and remove it within the user's product directory | Portable launch; optional install, uninstall and ordinary shortcut | No autostart, elevation, security-setting, installed-layout or system-hotkey changes | planned (no installation accepted) | core:desktop-product | capability:tray-control |

## 3. Extend without expanding authority {#vector:bounded-extension}

| ID | Capability | What it enables | Entry points | Invariants | Status | Runs on | Feeds |
| --- | --- | --- | --- | --- | --- | --- | --- |
| capability:platform-seams | Portable application boundary | Add future macOS or Linux adapters around the same proposals | Platform application interfaces | Interfaces confer no working adapter claim; every platform must establish its own native authority and field evidence | stable (interfaces only; adapters planned) | core:extension-boundaries | — (future platform outcome) |
| capability:advisor-seam | Inert optional advisor boundary | Preserve a future bounded advisory context without enabling a provider | Textless context schema and request identity/deadline | No provider, transport or native authority now; future text sharing requires separate user opt-in and authorization | stable (inert schema only) | core:extension-boundaries | — (future advice only) |

## 4. Make claims accountable {#vector:accountable-release}

| ID | Capability | What it enables | Entry points | Invariants | Status | Runs on | Feeds |
| --- | --- | --- | --- | --- | --- | --- | --- |
| capability:evidence-bounded-release | Evidence-bounded public product | Preserve provenance and publish only supported behavior with reversible installation | Release and support claims | Independent security review, frozen synthetic labels, native observations and latency gates; no private artifacts; upstream lineage retained | planned (core evidence accepted; desktop release pending) | core:switchdom | capability:reversible-install |
