//! Deterministic, platform-neutral layout analysis and proposal policy.
//!
//! Shared entry points take an extensible [`LayoutSet`] and an explicit known
//! source [`LayoutId`]. They do not infer the active operating-system layout.

mod advisor;
mod builtin;
mod platform;

pub use advisor::{
    accept_advice_if_current, Advisor, AdvisorCandidate, AdvisorContext, AdvisorRequest,
    AdvisorResponse, RequestIdentity,
};
pub use builtin::russian_english_layout_set;
pub use platform::{
    ApplicationAdapter, ApplyRequest, LinuxApplicationAdapter, MacOsApplicationAdapter,
};

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::Arc;

/// Maximum number of active language/layout profiles accepted by the core.
pub const MAX_ACTIVE_PROFILES: usize = 8;
/// A known source leaves at most seven target profile evaluations.
pub const MAX_TARGET_EVALUATIONS: usize = MAX_ACTIVE_PROFILES - 1;
/// Maximum retained Unicode scalar values in one analyzed token.
pub const MAX_TOKEN_CHARS: usize = 96;
/// Maximum physical-key assignments retained by a single layout mapping.
pub const MAX_LAYOUT_ASSIGNMENTS: usize = 256;
/// Maximum UTF-8 bytes in either language or layout identifiers retained by a set.
pub const MAX_PROFILE_ID_BYTES: usize = 64;
/// Maximum UTF-8 bytes in a version or provenance source retained by a set.
pub const MAX_PROVENANCE_FIELD_BYTES: usize = 512;

/// Frozen before the first performance measurement. A target score must meet
/// this confidence before the core proposes a change; scores are normalized by
/// each model to the inclusive range `[0, 1]`.
pub const MIN_TARGET_CONFIDENCE: f32 = 0.85;
/// Frozen before the first performance measurement. When another target exists,
/// the winner must exceed it by this score margin.
pub const MIN_RUNNER_UP_MARGIN: f32 = 0.08;

/// Extensible language identifier; it is intentionally not a closed RU/EN enum.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LanguageId(String);

impl LanguageId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for LanguageId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

/// Extensible keyboard layout identifier.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LayoutId(String);

impl LayoutId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for LayoutId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

/// Version and source for a profile, mapping, or language model.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VersionedProvenance {
    pub version: String,
    pub source: String,
}

impl VersionedProvenance {
    pub fn new(version: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            source: source.into(),
        }
    }
}

/// A physical key plus its shift state, shared by the source and target maps.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct KeyStroke {
    pub key_id: u16,
    pub shifted: bool,
}

/// One character assignment in a keyboard layout.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KeyAssignment {
    pub key_id: u16,
    pub shifted: bool,
    pub character: char,
}

impl KeyAssignment {
    pub const fn new(key_id: u16, shifted: bool, character: char) -> Self {
        Self {
            key_id,
            shifted,
            character,
        }
    }
}

/// Mapping data for a single physical keyboard layout.
#[derive(Clone, Debug)]
pub struct LayoutMapping {
    provenance: VersionedProvenance,
    character_to_stroke: HashMap<char, KeyStroke>,
    stroke_to_character: HashMap<KeyStroke, char>,
}

impl LayoutMapping {
    pub fn new(
        provenance: VersionedProvenance,
        assignments: impl IntoIterator<Item = KeyAssignment>,
    ) -> Result<Self, LayoutMappingError> {
        let mut character_to_stroke = HashMap::new();
        let mut stroke_to_character = HashMap::new();
        for (index, assignment) in assignments.into_iter().enumerate() {
            if index == MAX_LAYOUT_ASSIGNMENTS {
                return Err(LayoutMappingError::TooManyAssignments {
                    actual_at_least: index + 1,
                    maximum: MAX_LAYOUT_ASSIGNMENTS,
                });
            }
            let stroke = KeyStroke {
                key_id: assignment.key_id,
                shifted: assignment.shifted,
            };
            if character_to_stroke
                .insert(assignment.character, stroke)
                .is_some()
            {
                return Err(LayoutMappingError::DuplicateCharacter(assignment.character));
            }
            if stroke_to_character
                .insert(stroke, assignment.character)
                .is_some()
            {
                return Err(LayoutMappingError::DuplicateKeyStroke(stroke));
            }
        }
        Ok(Self {
            provenance,
            character_to_stroke,
            stroke_to_character,
        })
    }

    pub fn provenance(&self) -> &VersionedProvenance {
        &self.provenance
    }

    fn stroke_for(&self, character: char) -> Option<KeyStroke> {
        self.character_to_stroke.get(&character).copied()
    }

    fn character_for(&self, stroke: KeyStroke) -> Option<char> {
        self.stroke_to_character.get(&stroke).copied()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LayoutMappingError {
    DuplicateCharacter(char),
    DuplicateKeyStroke(KeyStroke),
    TooManyAssignments {
        actual_at_least: usize,
        maximum: usize,
    },
}

impl fmt::Display for LayoutMappingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateCharacter(character) => {
                write!(formatter, "duplicate layout character {character:?}")
            }
            Self::DuplicateKeyStroke(stroke) => write!(
                formatter,
                "duplicate physical key assignment ({}, shifted={})",
                stroke.key_id, stroke.shifted
            ),
            Self::TooManyAssignments {
                actual_at_least,
                maximum,
            } => write!(
                formatter,
                "layout mapping has at least {actual_at_least} assignments; maximum is {maximum}"
            ),
        }
    }
}

impl std::error::Error for LayoutMappingError {}

/// A deterministic model that scores text in one language profile.
///
/// Implementations must score in O(text length), use bounded internal state,
/// return a finite confidence score in `[0, 1]`, and provide provenance for any
/// embedded model data. The core clamps invalid model scores to zero as an
/// additional fail-closed measure.
pub trait LanguageModel: Send + Sync {
    fn score(&self, text: &str) -> f32;
    fn provenance(&self) -> &VersionedProvenance;
}

/// One active language/layout profile and its independent mapping/model data.
#[derive(Clone)]
pub struct LanguageProfile {
    language_id: LanguageId,
    layout_id: LayoutId,
    mapping: LayoutMapping,
    model: Arc<dyn LanguageModel>,
    provenance: VersionedProvenance,
}

impl LanguageProfile {
    pub fn new(
        language_id: LanguageId,
        layout_id: LayoutId,
        mapping: LayoutMapping,
        model: Arc<dyn LanguageModel>,
        provenance: VersionedProvenance,
    ) -> Self {
        Self {
            language_id,
            layout_id,
            mapping,
            model,
            provenance,
        }
    }

    pub fn language_id(&self) -> &LanguageId {
        &self.language_id
    }

    pub fn layout_id(&self) -> &LayoutId {
        &self.layout_id
    }

    pub fn provenance(&self) -> &VersionedProvenance {
        &self.provenance
    }

    pub fn mapping_provenance(&self) -> &VersionedProvenance {
        self.mapping.provenance()
    }

    pub fn model_provenance(&self) -> &VersionedProvenance {
        self.model.provenance()
    }
}

impl fmt::Debug for LanguageProfile {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LanguageProfile")
            .field("language_id", &self.language_id)
            .field("layout_id", &self.layout_id)
            .field("mapping_provenance", self.mapping.provenance())
            .field("model_provenance", self.model.provenance())
            .field("provenance", &self.provenance)
            .finish()
    }
}

/// Collection of active profiles. Construction rejects excess work up front.
#[derive(Clone, Debug)]
pub struct LayoutSet {
    active_profiles: Vec<LanguageProfile>,
}

impl LayoutSet {
    pub fn new(active_profiles: Vec<LanguageProfile>) -> Result<Self, LayoutSetError> {
        if active_profiles.len() > MAX_ACTIVE_PROFILES {
            return Err(LayoutSetError::TooManyActiveProfiles {
                actual: active_profiles.len(),
                maximum: MAX_ACTIVE_PROFILES,
            });
        }
        let mut ids = HashSet::with_capacity(active_profiles.len());
        for profile in &active_profiles {
            if profile.language_id.as_str().is_empty() {
                return Err(LayoutSetError::EmptyIdentifier("language id"));
            }
            if profile.layout_id.as_str().is_empty() {
                return Err(LayoutSetError::EmptyIdentifier("layout id"));
            }
            if profile.language_id.as_str().len() > MAX_PROFILE_ID_BYTES {
                return Err(LayoutSetError::IdentifierTooLong {
                    field: "language id",
                    actual: profile.language_id.as_str().len(),
                    maximum: MAX_PROFILE_ID_BYTES,
                });
            }
            if profile.layout_id.as_str().len() > MAX_PROFILE_ID_BYTES {
                return Err(LayoutSetError::IdentifierTooLong {
                    field: "layout id",
                    actual: profile.layout_id.as_str().len(),
                    maximum: MAX_PROFILE_ID_BYTES,
                });
            }
            for (field, value) in [
                ("profile version", profile.provenance.version.as_str()),
                ("profile source", profile.provenance.source.as_str()),
                (
                    "mapping version",
                    profile.mapping.provenance.version.as_str(),
                ),
                ("mapping source", profile.mapping.provenance.source.as_str()),
                ("model version", profile.model.provenance().version.as_str()),
                ("model source", profile.model.provenance().source.as_str()),
            ] {
                if value.len() > MAX_PROVENANCE_FIELD_BYTES {
                    return Err(LayoutSetError::ProvenanceFieldTooLong {
                        field,
                        actual: value.len(),
                        maximum: MAX_PROVENANCE_FIELD_BYTES,
                    });
                }
                if value.trim().is_empty() {
                    return Err(LayoutSetError::MissingProvenance(field));
                }
            }
            if !ids.insert(profile.layout_id.clone()) {
                return Err(LayoutSetError::DuplicateLayoutId(profile.layout_id.clone()));
            }
        }
        Ok(Self { active_profiles })
    }

    pub fn active_profiles(&self) -> &[LanguageProfile] {
        &self.active_profiles
    }

    pub fn profile(&self, layout_id: &LayoutId) -> Option<&LanguageProfile> {
        self.active_profiles
            .iter()
            .find(|profile| &profile.layout_id == layout_id)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LayoutSetError {
    TooManyActiveProfiles {
        actual: usize,
        maximum: usize,
    },
    DuplicateLayoutId(LayoutId),
    EmptyIdentifier(&'static str),
    MissingProvenance(&'static str),
    IdentifierTooLong {
        field: &'static str,
        actual: usize,
        maximum: usize,
    },
    ProvenanceFieldTooLong {
        field: &'static str,
        actual: usize,
        maximum: usize,
    },
}

impl fmt::Display for LayoutSetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooManyActiveProfiles { actual, maximum } => write!(
                formatter,
                "active profile count {actual} exceeds maximum {maximum}"
            ),
            Self::DuplicateLayoutId(layout_id) => {
                write!(formatter, "duplicate layout id {}", layout_id.as_str())
            }
            Self::EmptyIdentifier(field) => write!(formatter, "{field} cannot be empty"),
            Self::MissingProvenance(field) => {
                write!(formatter, "{field} provenance cannot be empty")
            }
            Self::IdentifierTooLong {
                field,
                actual,
                maximum,
            } => write!(
                formatter,
                "{field} has {actual} UTF-8 bytes; maximum is {maximum}"
            ),
            Self::ProvenanceFieldTooLong {
                field,
                actual,
                maximum,
            } => write!(
                formatter,
                "{field} has {actual} UTF-8 bytes; maximum is {maximum}"
            ),
        }
    }
}

impl std::error::Error for LayoutSetError {}

/// A target after one source-to-target conversion and one model evaluation.
#[derive(Clone, Debug, PartialEq)]
pub struct RankedCandidate {
    pub target_layout: LayoutId,
    pub replacement: String,
    pub confidence: f32,
}

/// A correction proposal returned only after the frozen policy thresholds pass.
#[derive(Clone, Debug, PartialEq)]
pub struct CorrectionProposal {
    pub source_layout: LayoutId,
    pub target_layout: LayoutId,
    pub original: String,
    pub replacement: String,
    pub confidence: f32,
    /// `None` means there was no runner-up; confidence still had to pass.
    pub runner_up_margin: Option<f32>,
}

/// Why the core returned no correction proposal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NoProposalReason {
    UnknownSource,
    InputTooLong,
    NoTargetProfiles,
    UnsupportedTargets,
    LowConfidence,
    Ambiguous,
}

/// Generic result; `proposal == None` means the caller keeps the original input.
#[derive(Clone, Debug, PartialEq)]
pub struct Decision {
    pub proposal: Option<CorrectionProposal>,
    pub ranked_candidates: Vec<RankedCandidate>,
    /// Counts each non-source profile inspected, including unsupported targets.
    pub target_evaluations: u8,
    pub reason: Option<NoProposalReason>,
}

struct AnalyzedInput {
    characters: Vec<char>,
}

impl AnalyzedInput {
    fn from_text(input: &str) -> Result<Self, NoProposalReason> {
        let mut characters = Vec::with_capacity(MAX_TOKEN_CHARS);
        for character in input.chars() {
            if characters.len() == MAX_TOKEN_CHARS {
                return Err(NoProposalReason::InputTooLong);
            }
            characters.push(character);
        }
        Ok(Self { characters })
    }
}

/// Analyze the original text once and evaluate each target profile at most once.
///
/// Candidate conversion and scoring each visit at most `MAX_TOKEN_CHARS`
/// characters. The fixed target limit makes the total work O(N) with a maximum
/// factor of seven; source-target pairs are never enumerated.
pub fn decide(layout_set: &LayoutSet, known_source_layout: &LayoutId, input: &str) -> Decision {
    let source = match layout_set.profile(known_source_layout) {
        Some(source) => source,
        None => return empty_decision(NoProposalReason::UnknownSource, 0),
    };
    let analyzed = match AnalyzedInput::from_text(input) {
        Ok(analyzed) => analyzed,
        Err(reason) => return empty_decision(reason, 0),
    };

    let mut target_evaluations = 0_u8;
    let mut ranked_candidates = Vec::with_capacity(MAX_TARGET_EVALUATIONS);
    for target in layout_set.active_profiles() {
        if target.layout_id == source.layout_id {
            continue;
        }
        if usize::from(target_evaluations) == MAX_TARGET_EVALUATIONS {
            break;
        }
        target_evaluations += 1;
        let Some(replacement) = convert_candidate(&source.mapping, &target.mapping, &analyzed)
        else {
            continue;
        };
        let confidence = normalized_score(target.model.score(&replacement));
        ranked_candidates.push(RankedCandidate {
            target_layout: target.layout_id.clone(),
            replacement,
            confidence,
        });
    }

    ranked_candidates.sort_by(|left, right| {
        right
            .confidence
            .total_cmp(&left.confidence)
            .then_with(|| left.target_layout.cmp(&right.target_layout))
    });

    if ranked_candidates.is_empty() {
        let reason = if target_evaluations == 0 {
            NoProposalReason::NoTargetProfiles
        } else {
            NoProposalReason::UnsupportedTargets
        };
        return Decision {
            proposal: None,
            ranked_candidates,
            target_evaluations,
            reason: Some(reason),
        };
    }

    let winner = &ranked_candidates[0];
    if winner.confidence < MIN_TARGET_CONFIDENCE {
        return Decision {
            proposal: None,
            ranked_candidates,
            target_evaluations,
            reason: Some(NoProposalReason::LowConfidence),
        };
    }

    let runner_up_margin = ranked_candidates
        .get(1)
        .map(|runner_up| winner.confidence - runner_up.confidence);
    if runner_up_margin.is_some_and(|margin| margin < MIN_RUNNER_UP_MARGIN) {
        return Decision {
            proposal: None,
            ranked_candidates,
            target_evaluations,
            reason: Some(NoProposalReason::Ambiguous),
        };
    }

    Decision {
        proposal: Some(CorrectionProposal {
            source_layout: source.layout_id.clone(),
            target_layout: winner.target_layout.clone(),
            original: input.to_owned(),
            replacement: winner.replacement.clone(),
            confidence: winner.confidence,
            runner_up_margin,
        }),
        ranked_candidates,
        target_evaluations,
        reason: None,
    }
}

fn empty_decision(reason: NoProposalReason, target_evaluations: u8) -> Decision {
    Decision {
        proposal: None,
        ranked_candidates: Vec::new(),
        target_evaluations,
        reason: Some(reason),
    }
}

fn normalized_score(score: f32) -> f32 {
    if score.is_finite() {
        score.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

fn convert_candidate(
    source: &LayoutMapping,
    target: &LayoutMapping,
    input: &AnalyzedInput,
) -> Option<String> {
    let mut converted = String::with_capacity(input.characters.len());
    let mut mapped_letters = 0_usize;
    for &character in &input.characters {
        if let Some(stroke) = source.stroke_for(character) {
            match target.character_for(stroke) {
                Some(target_character) => {
                    converted.push(target_character);
                    if character.is_alphabetic() {
                        mapped_letters += 1;
                    }
                }
                None if character.is_alphabetic() => return None,
                None => converted.push(character),
            }
        } else if character.is_alphabetic() {
            return None;
        } else {
            converted.push(character);
        }
    }
    (mapped_letters > 0).then_some(converted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_duplicate_mapping_entries() {
        let provenance = VersionedProvenance::new("test-v1", "synthetic test mapping");
        let entries = [
            KeyAssignment::new(1, false, 'a'),
            KeyAssignment::new(1, false, 'b'),
        ];
        assert!(matches!(
            LayoutMapping::new(provenance, entries),
            Err(LayoutMappingError::DuplicateKeyStroke(_))
        ));
    }

    #[test]
    fn rejects_oversized_mapping_after_a_bounded_prefix() {
        let provenance = VersionedProvenance::new("test-v1", "synthetic test mapping");
        let assignments = (0..=MAX_LAYOUT_ASSIGNMENTS)
            .map(|index| {
                KeyAssignment::new(
                    index as u16,
                    false,
                    char::from_u32(0x100 + index as u32).expect("test codepoint is valid"),
                )
            })
            .collect::<Vec<_>>();

        assert!(matches!(
            LayoutMapping::new(provenance, assignments),
            Err(LayoutMappingError::TooManyAssignments {
                actual_at_least,
                maximum
            }) if actual_at_least == MAX_LAYOUT_ASSIGNMENTS + 1
                && maximum == MAX_LAYOUT_ASSIGNMENTS
        ));
    }
}
