use dawn::domain::task::unique_id::{UID_LENGTH, UID_PATTERN};
use dawn::domain::task::{Description, Filter, Index, IndexRange, UniqueID};
use regex::Regex;
use std::collections::HashSet;
use std::str::FromStr;
use std::sync::LazyLock;

// Taskwarrior parity: indices reject leading zeros (e.g. "007" is text, not 7).
const INDEX_PATTERN: &str = r"[1-9]\d*";
const RANGE_PATTERN: &str = r"[1-9]\d*-[1-9]\d*";

static INDEX_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(&format!(r"^{INDEX_PATTERN}$")).unwrap());

// e.g. 1-10
static RANGE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(&format!(r"^{RANGE_PATTERN}$")).unwrap());

// e.g. 1,2-5,abcdefghi-_0
static SET_RE: LazyLock<Regex> = LazyLock::new(|| {
    let id_segment = format!(r"(?:{UID_PATTERN}|{INDEX_PATTERN}|{RANGE_PATTERN})");
    Regex::new(&format!(r"^{id_segment}(?:,{id_segment})+$")).unwrap()
});

/// List types to display when no command is present
#[derive(Debug, PartialEq)]
pub(crate) enum DefaultCommand {
    Next(Filter),
    Info(Filter),
}

/// No command word: pre→Filter, with bare-id heuristic for Info/Next routing
pub(crate) fn parse_default(raw_terms: &[String]) -> DefaultCommand {
    let (filter, has_bare_id) = classify(raw_terms);
    if has_bare_id {
        DefaultCommand::Info(filter)
    } else {
        DefaultCommand::Next(filter)
    }
}

/// Read-only report (e.g. `all`): pre and post merge into a single filter pass
pub(crate) fn parse_report(pre: &[String], post: &[String]) -> Filter {
    classify(pre.iter().chain(post.iter())).0
}

/// Mutation (modify/done/delete): pre→filter, post→description or annotation
pub(crate) fn parse_mutation(pre: &[String], post: &[String]) -> (Filter, Option<Description>) {
    let pre_filter = classify(pre).0;
    if pre_filter.is_empty() {
        promote_ids_from_post(post)
    } else {
        let desc: Vec<&str> = post
            .iter()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        let desc_opt = if desc.is_empty() {
            None
        } else {
            // non-empty by guard above; .ok() is None-as-unreachable
            Description::new(&desc.join(" ")).ok()
        };
        (pre_filter, desc_opt)
    }
}

fn classify<S: AsRef<str>>(raw_terms: impl IntoIterator<Item = S>) -> (Filter, bool) {
    let p = process_terms(raw_terms);
    let filter = Filter::default()
        .with_uids(p.uids)
        .with_indices(p.indices)
        .with_index_ranges(p.index_ranges)
        .with_words(p.words);
    (filter, p.has_bare_id)
}

// Promotes IDs from post into the filter when pre filter is empty;
// remaining words become the description or annotation
fn promote_ids_from_post(post: &[String]) -> (Filter, Option<Description>) {
    let parsed = process_terms(post);
    let filter = Filter::default()
        .with_uids(parsed.uids)
        .with_indices(parsed.indices)
        .with_index_ranges(parsed.index_ranges);
    let desc_opt = if parsed.words.is_empty() {
        None
    } else {
        // non-empty by guard above; .ok() is None-as-unreachable
        Description::new(&parsed.words.join(" ")).ok()
    };
    (filter, desc_opt)
}

#[derive(Default)]
struct Parsed {
    uids: HashSet<UniqueID>,
    indices: HashSet<Index>,
    index_ranges: HashSet<IndexRange>,
    words: Vec<String>,
    has_bare_id: bool,
}

fn process_terms<S: AsRef<str>>(raw_terms: impl IntoIterator<Item = S>) -> Parsed {
    let mut out = Parsed::default();

    for raw in raw_terms {
        let fragment = raw.as_ref().trim();
        if fragment.is_empty() {
            continue;
        }

        if SET_RE.is_match(fragment) {
            // Treat as a set of IDs: skip heuristic
            for seg in fragment.split(',') {
                if let Ok(u) = UniqueID::from_str(seg) {
                    out.uids.insert(u);
                } else if let Ok(i) = Index::from_str(seg) {
                    out.indices.insert(i);
                } else if RANGE_RE.is_match(seg) {
                    parse_range_segment(seg, &mut out);
                }
            }
            continue;
        }

        // UID before range: a 12-char digit/hyphen token (e.g. "12345-678901")
        // matches both; favor UID since such ranges need ~10-digit indices.
        if !looks_like_word(fragment) {
            if let Ok(u) = UniqueID::from_str(fragment) {
                out.uids.insert(u);
                out.has_bare_id = true;
                continue;
            }
            if INDEX_RE.is_match(fragment)
                && let Ok(i) = Index::from_str(fragment)
            {
                out.indices.insert(i);
                out.has_bare_id = true;
                continue;
            }
        }

        // Bare range (e.g. "5-10"): treated like a set, does not set has_bare_id.
        if RANGE_RE.is_match(fragment) {
            parse_range_segment(fragment, &mut out);
            continue;
        }

        // Fallback: treat as a search word
        out.words.push(fragment.to_string());
    }

    out
}

fn parse_range_segment(seg: &str, out: &mut Parsed) {
    let Some((lhs, rhs)) = seg.split_once('-') else {
        return;
    };
    let (Ok(start), Ok(end)) = (Index::from_str(lhs), Index::from_str(rhs)) else {
        return;
    };
    match IndexRange::new(start, end) {
        Ok(range) => {
            out.index_ranges.insert(range);
        }
        Err(idx) => {
            // Equal bounds (e.g. "5-5") collapse to a single index
            out.indices.insert(idx);
        }
    }
}

// Word-like heuristic: nanoid collision with this shape occurs at 40ppm (0.004%)
fn looks_like_word(s: &str) -> bool {
    let bytes = s.as_bytes();
    bytes.len() == UID_LENGTH
        && bytes[0].is_ascii_alphabetic()
        && bytes[1..].iter().all(|b| b.is_ascii_lowercase())
}

#[cfg(test)]
mod tests;
