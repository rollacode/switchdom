use layout_core::{
    decide, KeyAssignment, LanguageAssessment, LanguageId, LanguageModel, LanguageProfile,
    LayoutId, LayoutMapping, LayoutSet, LayoutSetError, LexicalEvidence, NoProposalReason,
    VersionedProvenance, MAX_ACTIVE_PROFILES, MAX_TARGET_EVALUATIONS, MAX_TOKEN_CHARS,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

struct FixedModel {
    confidence: f32,
    calls: Arc<AtomicUsize>,
    provenance: VersionedProvenance,
}

struct ShapeOnlyModel {
    provenance: VersionedProvenance,
}

impl LanguageModel for ShapeOnlyModel {
    fn evaluate(&self, _text: &str) -> LanguageAssessment {
        LanguageAssessment {
            confidence: 0.99,
            lexical_evidence: LexicalEvidence::default(),
        }
    }

    fn provenance(&self) -> &VersionedProvenance {
        &self.provenance
    }
}

impl LanguageModel for FixedModel {
    fn evaluate(&self, text: &str) -> LanguageAssessment {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let alphabetic_characters = text
            .chars()
            .filter(|character| character.is_alphabetic())
            .count();
        let total_bigrams = alphabetic_characters.saturating_sub(1) as u8;
        LanguageAssessment {
            confidence: self.confidence,
            lexical_evidence: LexicalEvidence {
                script_matches: true,
                all_caps: false,
                alphabetic_characters: alphabetic_characters as u8,
                recognized_bigrams: total_bigrams,
                total_bigrams,
            },
        }
    }

    fn provenance(&self) -> &VersionedProvenance {
        &self.provenance
    }
}

fn synthetic_profile(
    language: &str,
    layout: &str,
    character: char,
    confidence: f32,
    calls: Arc<AtomicUsize>,
) -> LanguageProfile {
    let provenance = VersionedProvenance::new(
        "synthetic-v1",
        "in-memory contract test profile; no shipped language data",
    );
    let mapping = LayoutMapping::new(
        provenance.clone(),
        [KeyAssignment::new(1, false, character)],
    )
    .expect("one synthetic key assignment is unique");
    LanguageProfile::new(
        LanguageId::new(language),
        LayoutId::new(layout),
        mapping,
        Arc::new(FixedModel {
            confidence,
            calls,
            provenance: provenance.clone(),
        }),
        provenance,
    )
}

fn three_profiles(first: f32, second: f32) -> LayoutSet {
    let calls = Arc::new(AtomicUsize::new(0));
    LayoutSet::new(vec![
        synthetic_profile("source-language", "source-layout", 'a', 0.0, calls.clone()),
        synthetic_profile(
            "target-one-language",
            "target-one",
            'b',
            first,
            calls.clone(),
        ),
        synthetic_profile("target-two-language", "target-two", 'c', second, calls),
    ])
    .expect("three in-memory profiles fit the active profile bound")
}

#[test]
fn synthetic_three_profile_set_selects_a_unique_winner() {
    let set = three_profiles(0.96, 0.40);
    let result = decide(&set, &LayoutId::from("source-layout"), "aaaa");

    let proposal = result
        .proposal
        .expect("unique, confident target should win");
    assert_eq!(proposal.source_layout, LayoutId::from("source-layout"));
    assert_eq!(proposal.target_layout, LayoutId::from("target-one"));
    assert_eq!(proposal.original, "aaaa");
    assert_eq!(proposal.replacement, "bbbb");
    assert_eq!(result.target_evaluations, 2);
}

#[test]
fn synthetic_tied_targets_preserve_the_input() {
    let set = three_profiles(0.95, 0.95);
    let result = decide(&set, &LayoutId::from("source-layout"), "aaaa");

    assert!(result.proposal.is_none());
    assert_eq!(result.reason, Some(NoProposalReason::Ambiguous));
    assert_eq!(result.ranked_candidates.len(), 2);
}

#[test]
fn confidence_threshold_applies_when_there_is_no_runner_up() {
    let calls = Arc::new(AtomicUsize::new(0));
    let set = LayoutSet::new(vec![
        synthetic_profile("source-language", "source-layout", 'a', 0.0, calls.clone()),
        synthetic_profile("target-language", "only-target", 'b', 0.84, calls),
    ])
    .expect("two in-memory profiles fit the active profile bound");
    let result = decide(&set, &LayoutId::from("source-layout"), "aaaa");

    assert!(result.proposal.is_none());
    assert_eq!(result.reason, Some(NoProposalReason::LowConfidence));
}

#[test]
fn confident_target_must_still_improve_on_source_plausibility() {
    let calls = Arc::new(AtomicUsize::new(0));
    let set = LayoutSet::new(vec![
        synthetic_profile("source-language", "source-layout", 'a', 0.90, calls.clone()),
        synthetic_profile("target-language", "only-target", 'b', 0.95, calls.clone()),
    ])
    .expect("two in-memory profiles fit the active profile bound");
    let result = decide(&set, &LayoutId::from("source-layout"), "aaaa");

    assert!(result.proposal.is_none());
    assert_eq!(result.reason, Some(NoProposalReason::LowConfidence));
    assert_eq!(result.source_confidence, Some(0.90));
    assert_eq!(result.ranked_candidates[0].confidence, 0.95);
    assert_eq!(
        calls.load(Ordering::SeqCst),
        2,
        "source and sole target each evaluated once"
    );
}

#[test]
fn shape_only_target_score_cannot_authorize_a_correction() {
    let calls = Arc::new(AtomicUsize::new(0));
    let provenance = VersionedProvenance::new("synthetic-v1", "in-memory shape-only test profile");
    let source = synthetic_profile("source-language", "source-layout", 'a', 0.0, calls.clone());
    let target_mapping =
        LayoutMapping::new(provenance.clone(), [KeyAssignment::new(1, false, 'b')])
            .expect("one synthetic key assignment is unique");
    let target = LanguageProfile::new(
        LanguageId::from("target-language"),
        LayoutId::from("target-layout"),
        target_mapping,
        Arc::new(ShapeOnlyModel {
            provenance: provenance.clone(),
        }),
        provenance,
    );
    let set = LayoutSet::new(vec![source, target]).expect("two in-memory profiles fit");

    let result = decide(&set, &LayoutId::from("source-layout"), "aaaa");
    assert!(result.proposal.is_none());
    assert_eq!(result.reason, Some(NoProposalReason::LowConfidence));
    assert_eq!(result.ranked_candidates[0].confidence, 0.0);
}

#[test]
fn unknown_source_preserves_the_input_without_target_work() {
    let set = three_profiles(0.96, 0.40);
    let result = decide(&set, &LayoutId::from("not-active"), "a");

    assert!(result.proposal.is_none());
    assert_eq!(result.reason, Some(NoProposalReason::UnknownSource));
    assert_eq!(result.target_evaluations, 0);
    assert!(result.ranked_candidates.is_empty());
}

#[test]
fn overlong_input_is_rejected_before_target_model_work() {
    let calls = Arc::new(AtomicUsize::new(0));
    let set = LayoutSet::new(vec![
        synthetic_profile("source-language", "source-layout", 'a', 0.0, calls.clone()),
        synthetic_profile("target-language", "target-layout", 'b', 0.99, calls.clone()),
    ])
    .expect("two in-memory profiles fit the active profile bound");
    let input = "a".repeat(MAX_TOKEN_CHARS + 1);

    let result = decide(&set, &LayoutId::from("source-layout"), &input);
    assert!(result.proposal.is_none());
    assert_eq!(result.reason, Some(NoProposalReason::InputTooLong));
    assert_eq!(result.target_evaluations, 0);
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

#[test]
fn unsupported_target_is_skipped_and_does_not_propose() {
    let calls = Arc::new(AtomicUsize::new(0));
    let provenance = VersionedProvenance::new("synthetic-v1", "in-memory contract test");
    let source = synthetic_profile("source-language", "source-layout", 'a', 0.0, calls.clone());
    let unsupported_mapping =
        LayoutMapping::new(provenance.clone(), [KeyAssignment::new(2, false, 'b')])
            .expect("one synthetic key assignment is unique");
    let unsupported = LanguageProfile::new(
        LanguageId::from("target-language"),
        LayoutId::from("unsupported-target"),
        unsupported_mapping,
        Arc::new(FixedModel {
            confidence: 1.0,
            calls: calls.clone(),
            provenance: provenance.clone(),
        }),
        provenance,
    );
    let set = LayoutSet::new(vec![source, unsupported]).expect("two profiles are allowed");

    let result = decide(&set, &LayoutId::from("source-layout"), "aaaa");
    assert!(result.proposal.is_none());
    assert_eq!(result.reason, Some(NoProposalReason::UnsupportedTargets));
    assert_eq!(result.target_evaluations, 1);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[test]
fn rejects_excess_profiles_before_any_model_evaluation() {
    let calls = Arc::new(AtomicUsize::new(0));
    let profiles = (0..=MAX_ACTIVE_PROFILES)
        .map(|index| {
            let character = char::from_u32('a' as u32 + index as u32).unwrap();
            synthetic_profile(
                "synthetic-language",
                &format!("layout-{index}"),
                character,
                0.9,
                calls.clone(),
            )
        })
        .collect();

    let result = LayoutSet::new(profiles);
    assert!(matches!(
        result,
        Err(LayoutSetError::TooManyActiveProfiles {
            actual,
            maximum
        }) if actual == MAX_ACTIVE_PROFILES + 1 && maximum == MAX_ACTIVE_PROFILES
    ));
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

#[test]
fn eight_profiles_never_exceed_seven_target_evaluations() {
    let calls = Arc::new(AtomicUsize::new(0));
    let profiles = (0..MAX_ACTIVE_PROFILES)
        .map(|index| {
            let character = char::from_u32('a' as u32 + index as u32).unwrap();
            let confidence = if index == 1 {
                0.99
            } else if index == 2 {
                0.80
            } else {
                0.20
            };
            synthetic_profile(
                "synthetic-language",
                &format!("layout-{index}"),
                character,
                confidence,
                calls.clone(),
            )
        })
        .collect();
    let set = LayoutSet::new(profiles).expect("eight profiles fit the active profile bound");

    let result = decide(&set, &LayoutId::from("layout-0"), "aaaa");
    assert_eq!(result.target_evaluations as usize, MAX_TARGET_EVALUATIONS);
    assert!(result.target_evaluations as usize <= MAX_TARGET_EVALUATIONS);
    assert_eq!(calls.load(Ordering::SeqCst), MAX_TARGET_EVALUATIONS + 1);
    assert_eq!(
        result
            .proposal
            .expect("winner has enough confidence and margin")
            .target_layout,
        LayoutId::from("layout-1")
    );
}
