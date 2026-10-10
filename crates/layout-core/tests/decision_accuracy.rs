use layout_core::{decide, russian_english_layout_set, LayoutId};

#[derive(Debug)]
struct AccuracyCase {
    name: String,
    source_layout: String,
    input: String,
    label: String,
    expected_target: Option<String>,
    expected_replacement: Option<String>,
}

fn accuracy_cases() -> Vec<AccuracyCase> {
    let document = include_str!("fixtures/decision-accuracy.json");
    let mut cases = Vec::new();
    let mut depth = 0_usize;
    let mut in_string = false;
    let mut escaped = false;
    let mut object_start = None;

    for (index, character) in document.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                in_string = false;
            }
            continue;
        }
        match character {
            '"' => in_string = true,
            '{' => {
                if depth == 1 {
                    object_start = Some(index);
                }
                depth += 1;
            }
            '}' => {
                depth -= 1;
                if depth == 1 {
                    if let Some(start) = object_start.take() {
                        cases.push(parse_case(&document[start..=index]));
                    }
                }
            }
            _ => {}
        }
    }
    assert_eq!(depth, 0, "accuracy fixture JSON braces are balanced");
    cases
}

fn parse_case(object: &str) -> AccuracyCase {
    AccuracyCase {
        name: required_string(object, "name"),
        source_layout: required_string(object, "source_layout"),
        input: required_string(object, "input"),
        label: required_string(object, "label"),
        expected_target: string_field(object, "expected_target"),
        expected_replacement: string_field(object, "expected_replacement"),
    }
}

fn required_string(object: &str, field: &str) -> String {
    string_field(object, field).unwrap_or_else(|| panic!("fixture field {field} is missing"))
}

/// The fixture intentionally uses a small flat JSON schema and no escapes in
/// case values. This parser handles JSON string escapes as a guard against
/// later punctuation cases adding them.
fn string_field(object: &str, field: &str) -> Option<String> {
    let key = format!("\"{field}\"");
    let key_start = object
        .find(key.as_str())
        .unwrap_or_else(|| panic!("fixture field {field} is missing"));
    let after_colon = object[key_start + key.len()..]
        .find(':')
        .map(|offset| key_start + key.len() + offset + 1)
        .expect("JSON field has a colon");
    let value = object[after_colon..].trim_start();
    if value.starts_with("null") {
        return None;
    }
    assert!(
        value.starts_with('"'),
        "fixture field {field} must be a string or null"
    );

    let mut output = String::new();
    let mut escaped = false;
    for character in value[1..].chars() {
        if escaped {
            output.push(match character {
                '"' => '"',
                '\\' => '\\',
                '/' => '/',
                'b' => '\u{0008}',
                'f' => '\u{000c}',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                other => panic!("unsupported JSON escape \\{other} in {field}"),
            });
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else if character == '"' {
            return Some(output);
        } else {
            output.push(character);
        }
    }
    panic!("unterminated JSON string in {field}")
}

#[test]
fn curated_accuracy_suite_catches_native_false_positives_and_checks_mistypes() {
    let cases = accuracy_cases();
    assert!(
        !cases.is_empty(),
        "accuracy fixture must contain labeled cases"
    );
    let layouts = russian_english_layout_set().expect("built-in RU/EN profiles are valid");
    let mut true_positives = 0_usize;
    let mut false_positives = 0_usize;
    let mut true_negatives = 0_usize;
    let mut false_negatives = 0_usize;
    let mut positive_total = 0_usize;
    let mut negative_total = 0_usize;
    let mut mismatches = Vec::new();

    for case in &cases {
        let is_positive = match case.label.as_str() {
            "positive" => true,
            "negative" => false,
            unknown => panic!("unknown label {unknown:?} in case {}", case.name),
        };
        let result = decide(
            &layouts,
            &LayoutId::from(case.source_layout.as_str()),
            &case.input,
        );
        match (is_positive, result.proposal.as_ref()) {
            (true, Some(proposal)) => {
                true_positives += 1;
                positive_total += 1;
                if Some(proposal.target_layout.as_str()) != case.expected_target.as_deref()
                    || Some(proposal.replacement.as_str()) != case.expected_replacement.as_deref()
                {
                    mismatches.push(format!(
                        "{} expected {:?} -> {:?}, got {:?} -> {:?}",
                        case.name,
                        case.expected_target,
                        case.expected_replacement,
                        proposal.target_layout.as_str(),
                        proposal.replacement
                    ));
                }
            }
            (true, None) => {
                false_negatives += 1;
                positive_total += 1;
                mismatches.push(format!(
                    "{} expected a correction, got no proposal",
                    case.name
                ));
            }
            (false, Some(proposal)) => {
                false_positives += 1;
                negative_total += 1;
                mismatches.push(format!(
                    "{} expected no change, got {:?} -> {:?}",
                    case.name,
                    proposal.target_layout.as_str(),
                    proposal.replacement
                ));
            }
            (false, None) => {
                true_negatives += 1;
                negative_total += 1;
                if case.expected_target.is_some() || case.expected_replacement.is_some() {
                    mismatches.push(format!(
                        "{} has inconsistent negative fixture labels",
                        case.name
                    ));
                }
            }
        }
    }

    let fpr_denominator = false_positives + true_negatives;
    let false_positive_rate = if fpr_denominator == 0 {
        0.0
    } else {
        false_positives as f64 / fpr_denominator as f64
    };
    println!(
        "curated accuracy suite (not real-world calibration): TP={true_positives}/{positive_total}, FP={false_positives}/{negative_total}, TN={true_negatives}/{negative_total}, FN={false_negatives}/{positive_total}, FPR={false_positives}/{fpr_denominator} ({false_positive_rate:.4})"
    );
    assert_eq!(positive_total + negative_total, cases.len());
    assert!(
        mismatches.is_empty(),
        "accuracy fixture mismatches:\n{}",
        mismatches.join("\n")
    );
}
