use crate::{
    KeyAssignment, LanguageId, LanguageModel, LanguageProfile, LayoutId, LayoutMapping, LayoutSet,
    LayoutSetError, VersionedProvenance,
};
use std::sync::Arc;

const UPSTREAM_LAYOUT_SOURCE: &str = "OopsLayout KeyMap.swift at a828e79c65a44e1842d3f687bd28ea3af44e0522; MIT; RU/EN physical key mapping adapted";
const FIRST_PARTY_MODEL_SOURCE: &str =
    "First-party shape-v1: vowel-ratio and consonant-run heuristic; no corpus or imported weights";

/// Build the only shipped profile set: English QWERTY and Russian ЙЦУКЕН.
///
/// The set and its heuristic model are versioned. The keyboard map's MIT
/// provenance is recorded in both mapping metadata and THIRD_PARTY_NOTICES.md.
pub fn russian_english_layout_set() -> Result<LayoutSet, LayoutSetError> {
    let english = LanguageProfile::new(
        LanguageId::new("en"),
        LayoutId::new("en-qwerty"),
        english_mapping(),
        Arc::new(ScriptShapeModel::new(
            "abcdefghijklmnopqrstuvwxyz",
            "aeiou",
            0.38,
            VersionedProvenance::new("shape-v1", FIRST_PARTY_MODEL_SOURCE),
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
        Arc::new(ScriptShapeModel::new(
            "абвгдеёжзийклмнопрстуфхцчшщъыьэюя",
            "аеёиоуыэюя",
            0.40,
            VersionedProvenance::new("shape-v1", FIRST_PARTY_MODEL_SOURCE),
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

/// A small deterministic shape heuristic, not a corpus-trained probability.
/// It compares vowel ratio to a hand-authored profile target and penalizes
/// consonant runs longer than two characters.
struct ScriptShapeModel {
    alphabet: &'static str,
    vowels: &'static str,
    expected_vowel_ratio: f32,
    provenance: VersionedProvenance,
}

impl ScriptShapeModel {
    fn new(
        alphabet: &'static str,
        vowels: &'static str,
        expected_vowel_ratio: f32,
        provenance: VersionedProvenance,
    ) -> Self {
        Self {
            alphabet,
            vowels,
            expected_vowel_ratio,
            provenance,
        }
    }
}

impl LanguageModel for ScriptShapeModel {
    fn score(&self, text: &str) -> f32 {
        let mut letter_count = 0_usize;
        let mut vowel_count = 0_usize;
        let mut current_consonants = 0_usize;
        let mut longest_consonant_run = 0_usize;

        for character in text.chars() {
            if character.is_alphabetic() {
                let lower = character.to_lowercase().next().unwrap_or(character);
                if !self.alphabet.contains(lower) {
                    return 0.0;
                }
                letter_count += 1;
                if self.vowels.contains(lower) {
                    vowel_count += 1;
                    current_consonants = 0;
                } else {
                    current_consonants += 1;
                    longest_consonant_run = longest_consonant_run.max(current_consonants);
                }
            } else {
                current_consonants = 0;
            }
        }

        if letter_count == 0 {
            return 0.0;
        }
        let vowel_ratio = vowel_count as f32 / letter_count as f32;
        let excess_cluster = longest_consonant_run.saturating_sub(2) as f32;
        (0.98 - (vowel_ratio - self.expected_vowel_ratio).abs() * 1.6 - excess_cluster * 0.12)
            .clamp(0.0, 1.0)
    }

    fn provenance(&self) -> &VersionedProvenance {
        &self.provenance
    }
}
