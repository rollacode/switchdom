# Switchdom responsibility boundaries

Switchdom separates language judgment from permission to edit another application's field. These are product responsibilities, including planned ones, rather than proof of deployed components. [Capabilities](capabilities.md) carries their current status; [Product contract](product.md) owns detailed expectations.

### Switchdom {#core:switchdom}

- Root: true

Owns the local Rust product, its public provenance and the truth of its support claims. Coordinates deterministic proposals, platform authority and visible user control. A successful core test cannot establish native correction, privacy safeguards or installation.

### Profiles and mappings {#core:profiles}

- Part of: core:switchdom

Owns language/layout identity and versioned model/mapping provenance. A profile key identity represents a shared physical position; it is not an OS virtual key, scan code or installed-layout handle. Native adapters must verify the relationship to the actual installed layout. More than two active profiles must require no binary language API or backend rewrite.

### Deterministic decision {#core:decision}

- Part of: core:switchdom

Owns source assessment, target conversion, discriminative evidence, ranking and conservative proposal policy. It neither discovers foreground authority nor performs OS edits. Abstention is a first-class outcome. Native failures must be repaired in the responsible implementation, without silently changing product intent.

### Windows input authority {#core:windows-authority}

- Part of: core:switchdom

Owns eligibility, bounded transient capture, current source-layout verification, invalidation, targeted replacement, layout selection and exact-context undo. It revalidates authority at the last application boundary. Neither a high model score nor a queued request authorizes editing an unknown or changed field. This responsibility is planned; native acceptance is pending.

### Desktop operation and packaging {#core:desktop-product}

- Part of: core:switchdom

Owns visible enable/pause/exit state, bounded generic active-profile configuration, local exceptions, capability disclosure and reversible per-user packaging. User control must agree with actual hook authority. This responsibility is planned; no Switchdom tray or installed binary is currently accepted.

### Extension boundaries {#core:extension-boundaries}

- Part of: core:switchdom

Owns portable platform contracts and the inert advisory seam. Future adapters must prove native behavior independently. Future advice stays outside input callbacks, carries request/generation/deadline identity and cannot apply OS edits. The present interface/schema does not enable another platform or an LLM.
