use crate::CorrectionProposal;

/// Guard data a future platform adapter must revalidate immediately before
/// replacing text. This crate provides the interface only, with no adapter.
#[derive(Clone, Debug, PartialEq)]
pub struct ApplyRequest {
    pub proposal: CorrectionProposal,
    pub request_id: u64,
    pub generation: u64,
    /// Opaque identity supplied by the platform for the focused target/caret.
    pub target_identity: u64,
}

/// Extension point for a future platform integration.
///
/// An implementation must reject stale target/caret identities and must never
/// apply a proposal after intervening input. The core itself has no OS calls.
pub trait ApplicationAdapter: Send + Sync {
    type Error;

    fn apply_if_current(&self, request: ApplyRequest) -> Result<(), Self::Error>;
}

/// Future macOS adapter contract. No implementation is included in this crate.
pub trait MacOsApplicationAdapter: ApplicationAdapter {}

/// Future Linux adapter contract. No implementation is included in this crate.
pub trait LinuxApplicationAdapter: ApplicationAdapter {}
