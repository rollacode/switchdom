use crate::{LayoutId, RankedCandidate, MAX_PROFILE_ID_BYTES, MAX_TARGET_EVALUATIONS};
use std::time::Instant;

/// Identity carried by any future asynchronous advisor request and response.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RequestIdentity {
    pub request_id: u64,
    pub generation: u64,
}

/// Candidate summary exposed to a future advisor; it contains no input text.
#[derive(Clone, Debug, PartialEq)]
pub struct AdvisorCandidate {
    pub target_layout: LayoutId,
    pub confidence: f32,
}

/// Bounded, generic context that contains only candidate identifiers and scores.
#[derive(Clone, Debug, PartialEq)]
pub struct AdvisorContext {
    identity: RequestIdentity,
    candidates: Vec<AdvisorCandidate>,
}

impl AdvisorContext {
    pub fn from_ranked(identity: RequestIdentity, ranked: &[RankedCandidate]) -> Option<Self> {
        if ranked.len() > MAX_TARGET_EVALUATIONS {
            return None;
        }
        if ranked.iter().any(|candidate| {
            candidate.target_layout.as_str().len() > MAX_PROFILE_ID_BYTES
                || !candidate.confidence.is_finite()
                || !(0.0..=1.0).contains(&candidate.confidence)
        }) {
            return None;
        }
        Some(Self {
            identity,
            candidates: ranked
                .iter()
                .map(|candidate| AdvisorCandidate {
                    target_layout: candidate.target_layout.clone(),
                    confidence: candidate.confidence,
                })
                .collect(),
        })
    }

    pub fn identity(&self) -> RequestIdentity {
        self.identity
    }

    pub fn candidates(&self) -> &[AdvisorCandidate] {
        &self.candidates
    }
}

/// Request metadata. The advisor is called by a future caller outside any hook.
#[derive(Clone, Debug)]
pub struct AdvisorRequest {
    pub context: AdvisorContext,
    pub deadline: Instant,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdvisorResponse {
    pub identity: RequestIdentity,
    pub suggested_target: LayoutId,
}

/// Optional extension only. There is no default advisor, transport, or hook call.
/// The trait exposes no text and has no application capability.
pub trait Advisor: Send + Sync {
    fn advise(&self, context: AdvisorContext, deadline: Instant) -> Option<AdvisorResponse>;
}

/// Discard late, stale-generation, mismatched-request, or unknown-target advice.
pub fn accept_advice_if_current(
    request: &AdvisorRequest,
    response: &AdvisorResponse,
    current_generation: u64,
    now: Instant,
) -> Option<LayoutId> {
    if now > request.deadline
        || response.identity != request.context.identity
        || response.identity.generation != current_generation
        || !request
            .context
            .candidates
            .iter()
            .any(|candidate| candidate.target_layout == response.suggested_target)
    {
        return None;
    }
    Some(response.suggested_target.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn rejects_stale_and_late_advice() {
        let now = Instant::now();
        let identity = RequestIdentity {
            request_id: 7,
            generation: 11,
        };
        let request = AdvisorRequest {
            context: AdvisorContext {
                identity,
                candidates: vec![AdvisorCandidate {
                    target_layout: LayoutId::from("target"),
                    confidence: 0.9,
                }],
            },
            deadline: now + Duration::from_secs(1),
        };
        let response = AdvisorResponse {
            identity,
            suggested_target: LayoutId::from("target"),
        };

        assert_eq!(
            accept_advice_if_current(&request, &response, 11, now),
            Some(LayoutId::from("target"))
        );
        assert_eq!(accept_advice_if_current(&request, &response, 12, now), None);
        assert_eq!(
            accept_advice_if_current(&request, &response, 11, now + Duration::from_secs(2)),
            None
        );
    }
}
