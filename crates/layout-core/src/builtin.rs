use crate::{
    KeyAssignment, LanguageAssessment, LanguageId, LanguageModel, LanguageProfile, LayoutId,
    LayoutMapping, LayoutSet, LayoutSetError, LexicalEvidence, VersionedProvenance,
};
use std::collections::HashSet;
use std::sync::Arc;

const UPSTREAM_LAYOUT_SOURCE: &str = "OopsLayout KeyMap.swift at a828e79c65a44e1842d3f687bd28ea3af44e0522; MIT; RU/EN physical key mapping adapted";
const FIRST_PARTY_MODEL_SOURCE: &str = "First-party, hand-authored common character bigram sets for English and Russian; no corpus training, FrequencyWords, OpenSubtitles, or imported weights";

/// Build the only shipped profile set: English QWERTY and Russian ЙЦУКЕН.
///
/// The set and its heuristic model are versioned. The keyboard map's MIT
/// provenance is recorded in both mapping metadata and THIRD_PARTY_NOTICES.md.
pub fn russian_english_layout_set() -> Result<LayoutSet, LayoutSetError> {
    let english = LanguageProfile::new(
        LanguageId::new("en"),
        LayoutId::new("en-qwerty"),
        english_mapping(),
        Arc::new(NgramModel::new(
            "abcdefghijklmnopqrstuvwxyz",
            ENGLISH_COMMON_BIGRAMS,
            VersionedProvenance::new("manual-bigrams-v2", FIRST_PARTY_MODEL_SOURCE),
        )),
        VersionedProvenance::new(
            "en-qwerty-v1",
            "First-party profile configuration; English key positions adapted from the MIT-attributed OopsLayout map",
        ),
    );
    let russian = LanguageProfile::new(
        LanguageId::new("ru"),
        LayoutId::new("ru-jcuken"),
        russian_mapping(),
        Arc::new(NgramModel::new(
            "абвгдеёжзийклмнопрстуфхцчшщъыьэюя",
            RUSSIAN_COMMON_BIGRAMS,
            VersionedProvenance::new("manual-bigrams-v2", FIRST_PARTY_MODEL_SOURCE),
        )),
        VersionedProvenance::new(
            "ru-jcuken-v1",
            "First-party profile configuration; Russian key positions adapted from the MIT-attributed OopsLayout map",
        ),
    );
    LayoutSet::new(vec![english, russian])
}

#[derive(Clone, Copy)]
struct PhysicalKeyPair {
    english_unshifted: char,
    english_shifted: char,
    russian_unshifted: char,
    russian_shifted: char,
}

const PHYSICAL_KEYS: &[PhysicalKeyPair] = &[
    PhysicalKeyPair {
        english_unshifted: 'q',
        english_shifted: 'Q',
        russian_unshifted: 'й',
        russian_shifted: 'Й',
    },
    PhysicalKeyPair {
        english_unshifted: 'w',
        english_shifted: 'W',
        russian_unshifted: 'ц',
        russian_shifted: 'Ц',
    },
    PhysicalKeyPair {
        english_unshifted: 'e',
        english_shifted: 'E',
        russian_unshifted: 'у',
        russian_shifted: 'У',
    },
    PhysicalKeyPair {
        english_unshifted: 'r',
        english_shifted: 'R',
        russian_unshifted: 'к',
        russian_shifted: 'К',
    },
    PhysicalKeyPair {
        english_unshifted: 't',
        english_shifted: 'T',
        russian_unshifted: 'е',
        russian_shifted: 'Е',
    },
    PhysicalKeyPair {
        english_unshifted: 'y',
        english_shifted: 'Y',
        russian_unshifted: 'н',
        russian_shifted: 'Н',
    },
    PhysicalKeyPair {
        english_unshifted: 'u',
        english_shifted: 'U',
        russian_unshifted: 'г',
        russian_shifted: 'Г',
    },
    PhysicalKeyPair {
        english_unshifted: 'i',
        english_shifted: 'I',
        russian_unshifted: 'ш',
        russian_shifted: 'Ш',
    },
    PhysicalKeyPair {
        english_unshifted: 'o',
        english_shifted: 'O',
        russian_unshifted: 'щ',
        russian_shifted: 'Щ',
    },
    PhysicalKeyPair {
        english_unshifted: 'p',
        english_shifted: 'P',
        russian_unshifted: 'з',
        russian_shifted: 'З',
    },
    PhysicalKeyPair {
        english_unshifted: '[',
        english_shifted: '{',
        russian_unshifted: 'х',
        russian_shifted: 'Х',
    },
    PhysicalKeyPair {
        english_unshifted: ']',
        english_shifted: '}',
        russian_unshifted: 'ъ',
        russian_shifted: 'Ъ',
    },
    PhysicalKeyPair {
        english_unshifted: 'a',
        english_shifted: 'A',
        russian_unshifted: 'ф',
        russian_shifted: 'Ф',
    },
    PhysicalKeyPair {
        english_unshifted: 's',
        english_shifted: 'S',
        russian_unshifted: 'ы',
        russian_shifted: 'Ы',
    },
    PhysicalKeyPair {
        english_unshifted: 'd',
        english_shifted: 'D',
        russian_unshifted: 'в',
        russian_shifted: 'В',
    },
    PhysicalKeyPair {
        english_unshifted: 'f',
        english_shifted: 'F',
        russian_unshifted: 'а',
        russian_shifted: 'А',
    },
    PhysicalKeyPair {
        english_unshifted: 'g',
        english_shifted: 'G',
        russian_unshifted: 'п',
        russian_shifted: 'П',
    },
    PhysicalKeyPair {
        english_unshifted: 'h',
        english_shifted: 'H',
        russian_unshifted: 'р',
        russian_shifted: 'Р',
    },
    PhysicalKeyPair {
        english_unshifted: 'j',
        english_shifted: 'J',
        russian_unshifted: 'о',
        russian_shifted: 'О',
    },
    PhysicalKeyPair {
        english_unshifted: 'k',
        english_shifted: 'K',
        russian_unshifted: 'л',
        russian_shifted: 'Л',
    },
    PhysicalKeyPair {
        english_unshifted: 'l',
        english_shifted: 'L',
        russian_unshifted: 'д',
        russian_shifted: 'Д',
    },
    PhysicalKeyPair {
        english_unshifted: ';',
        english_shifted: ':',
        russian_unshifted: 'ж',
        russian_shifted: 'Ж',
    },
    PhysicalKeyPair {
        english_unshifted: '\'',
        english_shifted: '"',
        russian_unshifted: 'э',
        russian_shifted: 'Э',
    },
    PhysicalKeyPair {
        english_unshifted: 'z',
        english_shifted: 'Z',
        russian_unshifted: 'я',
        russian_shifted: 'Я',
    },
    PhysicalKeyPair {
        english_unshifted: 'x',
        english_shifted: 'X',
        russian_unshifted: 'ч',
        russian_shifted: 'Ч',
    },
    PhysicalKeyPair {
        english_unshifted: 'c',
        english_shifted: 'C',
        russian_unshifted: 'с',
        russian_shifted: 'С',
    },
    PhysicalKeyPair {
        english_unshifted: 'v',
        english_shifted: 'V',
        russian_unshifted: 'м',
        russian_shifted: 'М',
    },
    PhysicalKeyPair {
        english_unshifted: 'b',
        english_shifted: 'B',
        russian_unshifted: 'и',
        russian_shifted: 'И',
    },
    PhysicalKeyPair {
        english_unshifted: 'n',
        english_shifted: 'N',
        russian_unshifted: 'т',
        russian_shifted: 'Т',
    },
    PhysicalKeyPair {
        english_unshifted: 'm',
        english_shifted: 'M',
        russian_unshifted: 'ь',
        russian_shifted: 'Ь',
    },
    PhysicalKeyPair {
        english_unshifted: ',',
        english_shifted: '<',
        russian_unshifted: 'б',
        russian_shifted: 'Б',
    },
    PhysicalKeyPair {
        english_unshifted: '.',
        english_shifted: '>',
        russian_unshifted: 'ю',
        russian_shifted: 'Ю',
    },
];

fn english_mapping() -> LayoutMapping {
    let mut assignments = Vec::with_capacity(PHYSICAL_KEYS.len() * 2);
    for (key_id, key) in PHYSICAL_KEYS.iter().enumerate() {
        assignments.push(KeyAssignment::new(
            key_id as u16,
            false,
            key.english_unshifted,
        ));
        assignments.push(KeyAssignment::new(key_id as u16, true, key.english_shifted));
    }
    LayoutMapping::new(
        VersionedProvenance::new("qwerty-jcuken-v1", UPSTREAM_LAYOUT_SOURCE),
        assignments,
    )
    .expect("the checked-in English key assignments are unique")
}

fn russian_mapping() -> LayoutMapping {
    let mut assignments = Vec::with_capacity(PHYSICAL_KEYS.len() * 2);
    for (key_id, key) in PHYSICAL_KEYS.iter().enumerate() {
        assignments.push(KeyAssignment::new(
            key_id as u16,
            false,
            key.russian_unshifted,
        ));
        assignments.push(KeyAssignment::new(key_id as u16, true, key.russian_shifted));
    }
    LayoutMapping::new(
        VersionedProvenance::new("qwerty-jcuken-v1", UPSTREAM_LAYOUT_SOURCE),
        assignments,
    )
    .expect("the checked-in Russian key assignments are unique")
}

/// Common English letter pairs selected by hand as a compact first-party
/// language discriminator. These are model features, not word exceptions or a
/// corpus-derived frequency table.
const ENGLISH_COMMON_BIGRAMS: &[&str] = &[
    "th", "he", "in", "er", "an", "re", "on", "at", "en", "nd", "ti", "es", "or", "te", "of", "ed",
    "is", "it", "al", "ar", "st", "to", "nt", "ng", "se", "ha", "as", "ou", "io", "le", "ve", "co",
    "me", "de", "hi", "ri", "ro", "ic", "ne", "ea", "ra", "ce", "li", "ch", "ll", "be", "ma", "si",
    "om", "ur", "ca", "el", "ta", "la", "ns", "wa", "ol", "ge", "go", "no", "un", "wh", "et", "ot",
    "od", "ac", "wi", "sh", "ee", "tt", "ss", "oo", "ie", "we", "ad", "rt", "ly", "ke", "ow", "ng",
    "rs", "ld", "ay", "em", "am", "im", "ir", "tr", "pr", "pl", "bl", "gr", "cr", "br", "dr", "fr",
    "cl", "fl", "gl", "sl", "sp", "st", "sw", "sc", "sk", "sm", "sn", "ch", "ck", "mp", "nd", "nd",
    "lp", "lt", "rm", "rn", "ss", "ry", "cy", "dy", "my", "py", "by", "ty", "sy", "ny", "fy", "gy",
    "hy", "ky", "ey", "ay", "oy", "ey", "ai", "oi", "au", "ue", "ui", "oa", "oe", "eu", "ei", "ia",
    "ua", "io", "eo", "oa", "ee", "oo", "ck", "gh", "ph", "qu", "wr", "kn",
];

/// Common Russian letter pairs selected by hand as a compact first-party
/// language discriminator. No lexical corpus or generated weights are used.
const RUSSIAN_COMMON_BIGRAMS: &[&str] = &[
    "ст", "но", "ен", "то", "на", "ов", "ни", "ра", "во", "ко", "пр", "ет", "ро", "ос", "ал", "го",
    "ог", "ер", "ин", "по", "за", "ло", "ре", "ли", "ка", "де", "ан", "да", "гд", "уд", "ру", "ми",
    "ир", "ве", "ив", "ия", "ти", "сь", "ел", "ок", "от", "ор", "та", "те", "се", "не", "чт", "ль",
    "ий", "мо", "ск", "ва", "сс", "ср", "ов", "тв", "го", "ве", "ем", "ме", "ны", "ый", "ое", "ая",
    "ую", "ри", "ви", "ом", "ед", "от", "до", "ка", "ли", "же", "му", "ма", "об", "бо", "бы", "ся",
    "сь", "ди", "зн", "сл", "ол", "ее", "ии", "ые", "ых", "ей", "ой", "кг", "дн", "жд", "цв", "шк",
    "зд", "вы", "жи", "ки", "ши", "ща", "ще",
];

/// Versioned hand-authored character bigram evidence. Scoring requires at
/// least four letters and two bigrams, then rewards recognized pairs. Script
/// validation and pair coverage provide language evidence; vowel ratios and
/// consonant-run shape do not contribute to confidence.
struct NgramModel {
    alphabet: &'static str,
    common_bigrams: HashSet<(char, char)>,
    provenance: VersionedProvenance,
}

impl NgramModel {
    fn new(alphabet: &'static str, bigrams: &[&str], provenance: VersionedProvenance) -> Self {
        let common_bigrams = bigrams
            .iter()
            .filter_map(|pair| {
                let mut characters = pair.chars();
                Some((characters.next()?, characters.next()?))
            })
            .collect();
        Self {
            alphabet,
            common_bigrams,
            provenance,
        }
    }
}

impl LanguageModel for NgramModel {
    fn evaluate(&self, text: &str) -> LanguageAssessment {
        let mut evidence = LexicalEvidence {
            script_matches: true,
            ..LexicalEvidence::default()
        };
        let mut previous = None;
        let mut saw_uppercase = false;
        let mut saw_lowercase = false;

        for character in text.chars() {
            if !character.is_alphabetic() {
                previous = None;
                continue;
            }
            let lower = character.to_lowercase().next().unwrap_or(character);
            saw_uppercase |= character.is_uppercase();
            saw_lowercase |= character.is_lowercase();
            if !self.alphabet.contains(lower) {
                evidence.script_matches = false;
                evidence.all_caps = saw_uppercase && !saw_lowercase;
                return LanguageAssessment {
                    confidence: 0.0,
                    lexical_evidence: evidence,
                };
            }

            evidence.alphabetic_characters = evidence.alphabetic_characters.saturating_add(1);
            if let Some(prior) = previous {
                evidence.total_bigrams = evidence.total_bigrams.saturating_add(1);
                if self.common_bigrams.contains(&(prior, lower)) {
                    evidence.recognized_bigrams = evidence.recognized_bigrams.saturating_add(1);
                }
            }
            previous = Some(lower);
        }
        evidence.all_caps = saw_uppercase && !saw_lowercase;

        if evidence.alphabetic_characters < 4 || evidence.total_bigrams < 2 {
            return LanguageAssessment {
                confidence: 0.0,
                lexical_evidence: evidence,
            };
        }
        let coverage = f32::from(evidence.recognized_bigrams) / f32::from(evidence.total_bigrams);
        let confidence = 0.55 + 0.45 * coverage;
        LanguageAssessment {
            // Acronym-like all-uppercase tokens lack the lexical context
            // needed for an automatic language decision.
            confidence: if evidence.all_caps {
                confidence.min(0.80)
            } else {
                confidence
            },
            lexical_evidence: evidence,
        }
    }

    fn provenance(&self) -> &VersionedProvenance {
        &self.provenance
    }
}
