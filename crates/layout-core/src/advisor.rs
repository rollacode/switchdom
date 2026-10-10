use crate::{
    Decision, LayoutId, LexicalEvidence, RankedCandidate, MAX_PROFILE_ID_BYTES,
    MAX_TARGET_EVALUATIONS, MAX_TOKEN_CHARS,
};
use std::time::Instant;

/// Identity carried by any future asynchronous advisor request and response.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RequestIdentity {
    pub request_id: u64,
    pub generation: u64,
}

/// Token text is local-only unless an eventual caller passes an explicit
/// user-controlled opt-in. The default is disabled.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AdvisorTextOptIn {
    #[default]
    Disabled,
    UserEnabled,
}

/// Candidate context suitable for a future advisor request. Raw text is
/// omitted unless the caller explicitly opts in; evidence is bounded counts
/// from the deterministic local model.
#[derive(Clone, Debug, PartialEq)]
pub struct AdvisorCandidate {
    pub target_layout: LayoutId,
    pub confidence: f32,
    pub replacement_text: Option<String>,
    pub lexical_evidence: LexicalEvidence,
}

/// Bounded, generic context for a future advisor. Constructing this value does
/// not call a provider or transmit data. Callers must run any future advisor
/// outside input callbacks, with the associated request deadline and identity.
#[derive(Clone, Debug, PartialEq)]
pub struct AdvisorContext {
    identity: RequestIdentity,
    source_layout: Option<LayoutId>,
    original_text: Option<String>,
    source_confidence: Option<f32>,
    source_lexical_evidence: Option<LexicalEvidence>,
    candidates: Vec<AdvisorCandidate>,
}

impl AdvisorContext {
    /// Build a no-text context from ranked candidates. This path is safe by
    /// default and retains generic lexical evidence only.
    pub fn from_ranked(identity: RequestIdentity, ranked: &[RankedCandidate]) -> Option<Self> {
        Self::from_parts(identity, None, None, None, None, ranked)
    }

    /// Build a bounded request schema from a core decision. Text is retained
    /// only with explicit `UserEnabled` consent; this function remains inert
    /// and performs no provider call or transmission.
    pub fn from_decision(
        identity: RequestIdentity,
        original_text: &str,
        source_layout: &LayoutId,
        decision: &Decision,
        text_opt_in: AdvisorTextOptIn,
    ) -> Option<Self> {
        let include_text = text_opt_in == AdvisorTextOptIn::UserEnabled;
        let bounded_original = if include_text {
            if original_text.chars().count() > MAX_TOKEN_CHARS {
                return None;
            }
            Some(original_text.to_owned())
        } else {
            None
        };
        let candidates = Self::from_parts(
            identity,
            Some(source_layout.clone()),
            bounded_original,
            decision.source_confidence,
            decision.source_lexical_evidence,
            &decision.ranked_candidates,
        )?;
        if include_text
            && candidates
                .candidates
                .iter()
                .any(|candidate| candidate.replacement_text.is_none())
        {
            return None;
        }
        Some(candidates)
    }

    fn from_parts(
        identity: RequestIdentity,
        source_layout: Option<LayoutId>,
        original_text: Option<String>,
        source_confidence: Option<f32>,
        source_lexical_evidence: Option<LexicalEvidence>,
        ranked: &[RankedCandidate],
    ) -> Option<Self> {
        if ranked.len() > MAX_TARGET_EVALUATIONS {
            return None;
        }
        if source_layout
            .as_ref()
            .is_some_and(|layout| layout.as_str().len() > MAX_PROFILE_ID_BYTES)
            || source_confidence.is_some_and(|score| !valid_score(score))
            || source_lexical_evidence.is_some_and(|evidence| !valid_evidence(evidence))
        {
            return None;
        }
        if original_text
            .as_ref()
            .is_some_and(|text| text.chars().count() > MAX_TOKEN_CHARS)
        {
            return None;
        }

        let mut candidates = Vec::with_capacity(ranked.len());
        for candidate in ranked {
            if candidate.target_layout.as_str().len() > MAX_PROFILE_ID_BYTES
                || !valid_score(candidate.confidence)
                || !valid_evidence(candidate.lexical_evidence)
            {
                return None;
            }
            let replacement_text = if original_text.is_some() {
                if candidate.replacement.chars().count() > MAX_TOKEN_CHARS {
                    return None;
                }
                Some(candidate.replacement.clone())
            } else {
                None
            };
            candidates.push(AdvisorCandidate {
                target_layout: candidate.target_layout.clone(),
                confidence: candidate.confidence,
                replacement_text,
                lexical_evidence: candidate.lexical_evidence,
            });
        }
        Some(Self {
            identity,
            source_layout,
            original_text,
            source_confidence,
            source_lexical_evidence,
            candidates,
        })
    }

    pub fn identity(&self) -> RequestIdentity {
        self.identity
    }

    pub fn source_layout(&self) -> Option<&LayoutId> {
        self.source_layout.as_ref()
    }

    pub fn original_text(&self) -> Option<&str> {
        self.original_text.as_deref()
    }

    pub fn source_confidence(&self) -> Option<f32> {
        self.source_confidence
    }

    pub fn source_lexical_evidence(&self) -> Option<LexicalEvidence> {
        self.source_lexical_evidence
    }

    pub fn candidates(&self) -> &[AdvisorCandidate] {
        &self.candidates
    }
}

fn valid_score(score: f32) -> bool {
    score.is_finite() && (0.0..=1.0).contains(&score)
}

fn valid_evidence(evidence: LexicalEvidence) -> bool {
    evidence.alphabetic_characters as usize <= MAX_TOKEN_CHARS
        && evidence.recognized_bigrams <= evidence.total_bigrams
        && evidence.total_bigrams as usize <= MAX_TOKEN_CHARS.saturating_sub(1)
}

/// Request metadata for a future advisor invocation. There is no default
/// advisor, transport, or hook callback in this crate.
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

/// Optional extension only. It has no OS application capability and receives
/// context only if a future caller explicitly invokes it.
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

    fn candidate() -> RankedCandidate {
        RankedCandidate {
            target_layout: LayoutId::from("target"),
            replacement: "hello".to_owned(),
            confidence: 0.9,
            lexical_evidence: LexicalEvidence {
                script_matches: true,
                all_caps: false,
                alphabetic_characters: 5,
                recognized_bigrams: 4,
                total_bigrams: 4,
            },
        }
    }

    #[test]
    fn ranked_context_is_textless_by_default_and_keeps_lexical_evidence() {
        assert_eq!(AdvisorTextOptIn::default(), AdvisorTextOptIn::Disabled);
        let context = AdvisorContext::from_ranked(
            RequestIdentity {
                request_id: 7,
                generation: 11,
            },
            &[candidate()],
        )
        .expect("one bounded target candidate fits");
        assert_eq!(context.original_text(), None);
        assert_eq!(context.candidates()[0].replacement_text, None);
        assert_eq!(
            context.candidates()[0].lexical_evidence.recognized_bigrams,
            4
        );
    }

    #[test]
    fn decision_context_carries_text_only_with_explicit_user_opt_in() {
        let ranked = vec![candidate()];
        let decision = Decision {
            proposal: None,
            ranked_candidates: ranked,
            source_confidence: Some(0.4),
            source_lexical_evidence: Some(LexicalEvidence {
                script_matches: true,
                all_caps: false,
                alphabetic_characters: 5,
                recognized_bigrams: 1,
                total_bigrams: 4,
            }),
            target_evaluations: 1,
            reason: None,
        };
        let identity = RequestIdentity {
            request_id: 8,
            generation: 12,
        };
        let source = LayoutId::from("source");

        let disabled = AdvisorContext::from_decision(
            identity,
            "ghbdtn",
            &source,
            &decision,
            AdvisorTextOptIn::Disabled,
        )
        .expect("no-text context is bounded");
        assert_eq!(disabled.original_text(), None);
        assert_eq!(disabled.candidates()[0].replacement_text, None);

        let opted_in = AdvisorContext::from_decision(
            identity,
            "ghbdtn",
            &source,
            &decision,
            AdvisorTextOptIn::UserEnabled,
        )
        .expect("explicitly opted-in text is bounded");
        assert_eq!(opted_in.original_text(), Some("ghbdtn"));
        assert_eq!(
            opted_in.candidates()[0].replacement_text.as_deref(),
            Some("hello")
        );
        assert_eq!(
            opted_in.candidates()[0].lexical_evidence.recognized_bigrams,
            4
        );
    }

    #[test]
    fn rejects_oversized_opted_in_context_text() {
        let decision = Decision {
            proposal: None,
            ranked_candidates: vec![candidate()],
            source_confidence: None,
            source_lexical_evidence: None,
            target_evaluations: 1,
            reason: None,
        };
        let long_text = "x".repeat(MAX_TOKEN_CHARS + 1);
        assert!(AdvisorContext::from_decision(
            RequestIdentity {
                request_id: 9,
                generation: 13,
            },
            &long_text,
            &LayoutId::from("source"),
            &decision,
            AdvisorTextOptIn::UserEnabled,
        )
        .is_none());
    }

    #[test]
    fn rejects_stale_and_late_advice() {
        let now = Instant::now();
        let identity = RequestIdentity {
            request_id: 7,
            generation: 11,
        };
        let context = AdvisorContext::from_ranked(identity, &[candidate()]).unwrap();
        let request = AdvisorRequest {
            context,
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
