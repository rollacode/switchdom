use layout_core::{
    decide, russian_english_layout_set, LayoutId, NoProposalReason, MAX_TOKEN_CHARS,
};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io::Read;
use std::process::Command;
use std::time::Instant;

// Frozen release budget from the task; do not retune from measured output.
const P99_LIMIT_MICROSECONDS: f64 = 100.0;
const MINIMUM_SAMPLE_COUNT: usize = 10_000;
const MAX_CORPUS_BYTES: u64 = 64 * 1024;
const MAX_FIXTURE_CASES: usize = 32;
const MAX_ITERATIONS_PER_CASE: usize = 100_000;
const MAX_JSON_NESTING: usize = 16;

struct Arguments {
    corpus: String,
    iterations: usize,
    max_token_chars: usize,
    json: bool,
}

struct Case {
    name: String,
    source_layout: LayoutId,
    input: String,
    expected_target: Option<LayoutId>,
    expected_replacement: Option<String>,
    expected_reason: Option<NoProposalReason>,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("decision-perf-oracle: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let Some(arguments) = parse_arguments()? else {
        return Ok(());
    };
    if arguments.max_token_chars == 0 || arguments.max_token_chars > MAX_TOKEN_CHARS {
        return Err(format!("--max-token-chars must be between 1 and {MAX_TOKEN_CHARS}").into());
    }
    if arguments.iterations == 0 || arguments.iterations > MAX_ITERATIONS_PER_CASE {
        return Err(format!(
            "--iterations must be between 1 and {MAX_ITERATIONS_PER_CASE} per case"
        )
        .into());
    }
    let mut corpus_text = String::new();
    fs::File::open(&arguments.corpus)?
        .take(MAX_CORPUS_BYTES + 1)
        .read_to_string(&mut corpus_text)?;
    if corpus_text.len() as u64 > MAX_CORPUS_BYTES {
        return Err(format!("fixture exceeds {MAX_CORPUS_BYTES} bytes").into());
    }
    let cases = parse_fixture(&corpus_text)?;
    if cases.is_empty() {
        return Err("fixture contains no cases".into());
    }
    let fixture_max_chars = cases
        .iter()
        .map(|case| case.input.chars().count())
        .max()
        .unwrap_or(0);
    if fixture_max_chars != MAX_TOKEN_CHARS {
        return Err(format!(
            "fixture must cover exactly {MAX_TOKEN_CHARS} characters; found {fixture_max_chars}"
        )
        .into());
    }
    for case in &cases {
        let length = case.input.chars().count();
        if length > arguments.max_token_chars {
            return Err(format!(
                "fixture case {:?} has {length} characters, over the requested limit {}",
                case.name, arguments.max_token_chars
            )
            .into());
        }
    }
    let sample_count = cases
        .len()
        .checked_mul(arguments.iterations)
        .ok_or("sample count overflow")?;
    if sample_count < MINIMUM_SAMPLE_COUNT {
        return Err(format!(
            "requested {sample_count} samples; at least {MINIMUM_SAMPLE_COUNT} are required"
        )
        .into());
    }

    // Fixture parsing and profile setup stay outside the measured section.
    let layout_set = russian_english_layout_set()?;
    for case in &cases {
        let decision = decide(&layout_set, &case.source_layout, &case.input);
        if !matches_expected(case, &decision) {
            return Err(format!(
                "frozen fixture case {:?} did not match its expected result before measurement",
                case.name
            )
            .into());
        }
    }

    let mut samples_microseconds = Vec::with_capacity(sample_count);
    let mut fixtures_match = true;
    for case in &cases {
        for _ in 0..arguments.iterations {
            let start = Instant::now();
            let decision = decide(&layout_set, &case.source_layout, &case.input);
            let elapsed = start.elapsed();
            samples_microseconds.push(elapsed.as_secs_f64() * 1_000_000.0);
            // Compare after stopping the timer so validation is not detector work.
            fixtures_match &= matches_expected(case, &decision);
        }
    }

    samples_microseconds.sort_by(f64::total_cmp);
    let median = percentile(&samples_microseconds, 0.50);
    let p95 = percentile(&samples_microseconds, 0.95);
    let p99 = percentile(&samples_microseconds, 0.99);
    let rustc_version = Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned())
        .unwrap_or_else(|| "unavailable".to_owned());
    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };

    if arguments.json {
        println!(
            concat!(
                "{\"oracle\":\"decision-perf-oracle\",",
                "\"crate_version\":\"{}\",",
                "\"os\":\"{}\",\"arch\":\"{}\",\"profile\":\"{}\",",
                "\"rustc\":\"{}\",\"fixture_cases\":{},",
                "\"iterations_per_case\":{},\"samples\":{},",
                "\"fixture_max_chars\":{},\"median_us\":{:.3},",
                "\"p95_us\":{:.3},\"p99_us\":{:.3},",
                "\"p99_limit_us\":{:.1},\"fixtures_match\":{}}}"
            ),
            env!("CARGO_PKG_VERSION"),
            std::env::consts::OS,
            std::env::consts::ARCH,
            profile,
            escape_json(&rustc_version),
            cases.len(),
            arguments.iterations,
            samples_microseconds.len(),
            fixture_max_chars,
            median,
            p95,
            p99,
            P99_LIMIT_MICROSECONDS,
            fixtures_match
        );
    } else {
        println!("oracle: decision-perf-oracle");
        println!("crate: layout-core {}", env!("CARGO_PKG_VERSION"));
        println!(
            "build: {} {} ({profile}; {rustc_version})",
            std::env::consts::OS,
            std::env::consts::ARCH
        );
        println!(
            "corpus: {} cases, {} characters maximum, {} samples",
            cases.len(),
            fixture_max_chars,
            samples_microseconds.len()
        );
        println!(
            "latency: median {median:.3} us, p95 {p95:.3} us, p99 {p99:.3} us (limit {P99_LIMIT_MICROSECONDS:.1} us)"
        );
        println!("fixtures_match: {fixtures_match}");
    }

    if !fixtures_match {
        return Err("one or more measured outputs differ from the frozen fixture".into());
    }
    if samples_microseconds.len() < MINIMUM_SAMPLE_COUNT {
        return Err(format!(
            "only {} samples were measured; at least {MINIMUM_SAMPLE_COUNT} are required",
            samples_microseconds.len()
        )
        .into());
    }
    if p99 > P99_LIMIT_MICROSECONDS {
        return Err(
            format!("p99 {p99:.3} us exceeds the {P99_LIMIT_MICROSECONDS:.1} us limit").into(),
        );
    }
    Ok(())
}

fn matches_expected(case: &Case, decision: &layout_core::Decision) -> bool {
    match (
        &case.expected_target,
        &case.expected_replacement,
        case.expected_reason,
        &decision.proposal,
    ) {
        (Some(expected_target), Some(expected_replacement), None, Some(proposal)) => {
            &proposal.target_layout == expected_target
                && &proposal.replacement == expected_replacement
        }
        (None, None, Some(expected_reason), None) => decision.reason == Some(expected_reason),
        _ => false,
    }
}

fn percentile(sorted_samples: &[f64], percentile: f64) -> f64 {
    if sorted_samples.is_empty() {
        return 0.0;
    }
    let one_based_rank = (percentile * sorted_samples.len() as f64).ceil() as usize;
    sorted_samples[one_based_rank
        .saturating_sub(1)
        .min(sorted_samples.len() - 1)]
}

fn parse_arguments() -> Result<Option<Arguments>, Box<dyn Error>> {
    let mut corpus = None;
    let mut iterations = None;
    let mut max_token_chars = None;
    let mut json = false;
    let mut args = std::env::args().skip(1);
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--corpus" => corpus = Some(args.next().ok_or("--corpus requires a path")?),
            "--iterations" => {
                let value = args.next().ok_or("--iterations requires a number")?;
                iterations = Some(value.parse::<usize>()?);
            }
            "--max-token-chars" => {
                let value = args.next().ok_or("--max-token-chars requires a number")?;
                max_token_chars = Some(value.parse::<usize>()?);
            }
            "--json" => json = true,
            "--help" | "-h" => {
                println!("decision-perf-oracle --corpus PATH --iterations N --max-token-chars N [--json]");
                println!("Iterations are run per fixture case; the fixture and profiles load outside timing.");
                return Ok(None);
            }
            other => return Err(format!("unknown argument {other:?}").into()),
        }
    }
    Ok(Some(Arguments {
        corpus: corpus.ok_or("missing --corpus")?,
        iterations: iterations.ok_or("missing --iterations")?,
        max_token_chars: max_token_chars.ok_or("missing --max-token-chars")?,
        json,
    }))
}

fn parse_fixture(source: &str) -> Result<Vec<Case>, Box<dyn Error>> {
    let mut root = as_object(JsonParser::new(source).parse()?, "root")?;
    match root.remove("version") {
        Some(JsonValue::Number(1)) => {}
        _ => return Err("fixture version must be 1".into()),
    }
    let cases = match root.remove("cases") {
        Some(JsonValue::Array(cases)) => cases,
        _ => return Err("fixture must contain a cases array".into()),
    };
    if cases.len() > MAX_FIXTURE_CASES {
        return Err(format!(
            "fixture has {} cases; maximum is {MAX_FIXTURE_CASES}",
            cases.len()
        )
        .into());
    }
    if !root.is_empty() {
        return Err("fixture contains unknown root fields".into());
    }

    let mut parsed = Vec::with_capacity(cases.len());
    for (index, value) in cases.into_iter().enumerate() {
        let mut fields = as_object(value, "case")?;
        let name = take_string(&mut fields, "name")?;
        let source_layout = LayoutId::new(take_string(&mut fields, "source_layout")?);
        let input = take_string(&mut fields, "input")?;
        let expected_target =
            take_optional_string(&mut fields, "expected_target")?.map(LayoutId::new);
        let expected_replacement = take_optional_string(&mut fields, "expected_replacement")?;
        let expected_reason = take_optional_string(&mut fields, "expected_reason")?
            .map(|reason| parse_reason(&reason))
            .transpose()?;
        if expected_target.is_some() != expected_replacement.is_some() {
            return Err(format!(
                "fixture case {index} must specify both expected outputs or neither"
            )
            .into());
        }
        if expected_target.is_some() == expected_reason.is_some() {
            return Err(format!(
                "fixture case {index} must expect either a proposal or a no-proposal reason"
            )
            .into());
        }
        if !fields.is_empty() {
            return Err(format!("fixture case {name:?} has unknown fields").into());
        }
        parsed.push(Case {
            name,
            source_layout,
            input,
            expected_target,
            expected_replacement,
            expected_reason,
        });
    }
    Ok(parsed)
}

fn parse_reason(value: &str) -> Result<NoProposalReason, String> {
    match value {
        "unknown_source" => Ok(NoProposalReason::UnknownSource),
        "input_too_long" => Ok(NoProposalReason::InputTooLong),
        "no_target_profiles" => Ok(NoProposalReason::NoTargetProfiles),
        "unsupported_targets" => Ok(NoProposalReason::UnsupportedTargets),
        "low_confidence" => Ok(NoProposalReason::LowConfidence),
        "ambiguous" => Ok(NoProposalReason::Ambiguous),
        _ => Err(format!("unknown no-proposal reason {value:?}")),
    }
}

fn take_string(fields: &mut HashMap<String, JsonValue>, key: &str) -> Result<String, String> {
    match fields.remove(key) {
        Some(JsonValue::String(value)) => Ok(value),
        _ => Err(format!("fixture field {key:?} must be a string")),
    }
}

fn take_optional_string(
    fields: &mut HashMap<String, JsonValue>,
    key: &str,
) -> Result<Option<String>, String> {
    match fields.remove(key) {
        Some(JsonValue::String(value)) => Ok(Some(value)),
        Some(JsonValue::Null) => Ok(None),
        _ => Err(format!("fixture field {key:?} must be a string or null")),
    }
}

fn as_object(value: JsonValue, name: &str) -> Result<HashMap<String, JsonValue>, String> {
    match value {
        JsonValue::Object(value) => Ok(value),
        _ => Err(format!("fixture {name} must be an object")),
    }
}

#[derive(Debug)]
enum JsonValue {
    Object(HashMap<String, JsonValue>),
    Array(Vec<JsonValue>),
    String(String),
    Number(u64),
    Null,
}

struct JsonParser<'a> {
    source: &'a str,
    offset: usize,
}

impl<'a> JsonParser<'a> {
    fn new(source: &'a str) -> Self {
        Self { source, offset: 0 }
    }

    fn parse(mut self) -> Result<JsonValue, String> {
        let value = self.parse_value(0)?;
        self.skip_whitespace();
        if self.offset != self.source.len() {
            return Err("unexpected trailing data in fixture".to_owned());
        }
        Ok(value)
    }

    fn parse_value(&mut self, depth: usize) -> Result<JsonValue, String> {
        if depth > MAX_JSON_NESTING {
            return Err(format!("fixture JSON nesting exceeds {MAX_JSON_NESTING}"));
        }
        self.skip_whitespace();
        match self.peek_byte() {
            Some(b'{') => self.parse_object(depth),
            Some(b'[') => self.parse_array(depth),
            Some(b'"') => self.parse_string().map(JsonValue::String),
            Some(b'n') => {
                self.expect_bytes(b"null")?;
                Ok(JsonValue::Null)
            }
            Some(byte) if byte.is_ascii_digit() => self.parse_number().map(JsonValue::Number),
            _ => Err(format!("invalid JSON value at byte {}", self.offset)),
        }
    }

    fn parse_object(&mut self, depth: usize) -> Result<JsonValue, String> {
        self.expect_byte(b'{')?;
        let mut fields = HashMap::new();
        self.skip_whitespace();
        if self.consume_byte(b'}') {
            return Ok(JsonValue::Object(fields));
        }
        loop {
            self.skip_whitespace();
            let key = self.parse_string()?;
            self.skip_whitespace();
            self.expect_byte(b':')?;
            let value = self.parse_value(depth + 1)?;
            if fields.insert(key.clone(), value).is_some() {
                return Err(format!("duplicate JSON object key {key:?}"));
            }
            self.skip_whitespace();
            if self.consume_byte(b'}') {
                return Ok(JsonValue::Object(fields));
            }
            self.expect_byte(b',')?;
        }
    }

    fn parse_array(&mut self, depth: usize) -> Result<JsonValue, String> {
        self.expect_byte(b'[')?;
        let mut values = Vec::new();
        self.skip_whitespace();
        if self.consume_byte(b']') {
            return Ok(JsonValue::Array(values));
        }
        loop {
            values.push(self.parse_value(depth + 1)?);
            self.skip_whitespace();
            if self.consume_byte(b']') {
                return Ok(JsonValue::Array(values));
            }
            self.expect_byte(b',')?;
        }
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.expect_byte(b'"')?;
        let mut value = String::new();
        loop {
            let character = self
                .source
                .get(self.offset..)
                .and_then(|rest| rest.chars().next())
                .ok_or_else(|| "unterminated JSON string".to_owned())?;
            self.offset += character.len_utf8();
            match character {
                '"' => return Ok(value),
                '\\' => value.push(self.parse_escape()?),
                character if (character as u32) < 0x20 => {
                    return Err("unescaped control character in JSON string".to_owned())
                }
                character => value.push(character),
            }
        }
    }

    fn parse_escape(&mut self) -> Result<char, String> {
        let escaped = self
            .peek_byte()
            .ok_or_else(|| "incomplete JSON escape".to_owned())?;
        self.offset += 1;
        match escaped {
            b'"' => Ok('"'),
            b'\\' => Ok('\\'),
            b'/' => Ok('/'),
            b'b' => Ok('\u{0008}'),
            b'f' => Ok('\u{000c}'),
            b'n' => Ok('\n'),
            b'r' => Ok('\r'),
            b't' => Ok('\t'),
            b'u' => {
                let digits = self
                    .source
                    .get(self.offset..self.offset + 4)
                    .ok_or_else(|| "incomplete Unicode escape".to_owned())?;
                let codepoint = u16::from_str_radix(digits, 16)
                    .map_err(|_| "invalid Unicode escape".to_owned())?;
                self.offset += 4;
                char::from_u32(u32::from(codepoint))
                    .ok_or_else(|| "unsupported surrogate in Unicode escape".to_owned())
            }
            _ => Err("invalid JSON escape".to_owned()),
        }
    }

    fn parse_number(&mut self) -> Result<u64, String> {
        let start = self.offset;
        while self.peek_byte().is_some_and(|byte| byte.is_ascii_digit()) {
            self.offset += 1;
        }
        self.source[start..self.offset]
            .parse::<u64>()
            .map_err(|_| "invalid fixture number".to_owned())
    }

    fn skip_whitespace(&mut self) {
        while self
            .peek_byte()
            .is_some_and(|byte| matches!(byte, b' ' | b'\n' | b'\r' | b'\t'))
        {
            self.offset += 1;
        }
    }

    fn peek_byte(&self) -> Option<u8> {
        self.source.as_bytes().get(self.offset).copied()
    }

    fn consume_byte(&mut self, expected: u8) -> bool {
        if self.peek_byte() == Some(expected) {
            self.offset += 1;
            true
        } else {
            false
        }
    }

    fn expect_byte(&mut self, expected: u8) -> Result<(), String> {
        if self.consume_byte(expected) {
            Ok(())
        } else {
            Err(format!(
                "expected byte {:?} at {}",
                expected as char, self.offset
            ))
        }
    }

    fn expect_bytes(&mut self, expected: &[u8]) -> Result<(), String> {
        if self
            .source
            .as_bytes()
            .get(self.offset..self.offset + expected.len())
            == Some(expected)
        {
            self.offset += expected.len();
            Ok(())
        } else {
            Err(format!("expected JSON null at byte {}", self.offset))
        }
    }
}

fn escape_json(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            character if (character as u32) < 0x20 => {
                escaped.push_str(&format!("\\u{:04x}", character as u32));
            }
            character => escaped.push(character),
        }
    }
    escaped
}
