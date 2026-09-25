//! `{{name|arg|k=v}}` with nesting: arguments stay raw, `|` inside nested `{{…}}` and
//! `[[…]]` doesn't split.

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Template {
    pub name: String,
    pub args: Vec<String>,
    pub named: BTreeMap<String, String>,
}

/// `s[at..]` starts with `{{`. Returns the template and the byte index after `}}`.
/// `None` if there's no template at `at`, if the closing is missing, or if `at` doesn't
/// fall on a character boundary: wikitext is external data and must not crash the process.
pub fn parse_template_at(s: &str, at: usize) -> Option<(Template, usize)> {
    if !s.get(at..)?.starts_with("{{") {
        return None;
    }
    let b = s.as_bytes();
    let mut i = at + 2;
    let mut depth_t = 0usize; // nested {{ }}
    let mut depth_l = 0usize; // nested [[ ]]
    let mut parts: Vec<String> = Vec::new();
    let mut cur = String::new();
    while i < b.len() {
        let rest = s.get(i..)?;
        if rest.starts_with("{{") {
            depth_t += 1;
            cur.push_str("{{");
            i += 2;
            continue;
        }
        if rest.starts_with("}}") {
            if depth_t == 0 {
                parts.push(std::mem::take(&mut cur));
                return Some((assemble(parts), i + 2));
            }
            depth_t -= 1;
            cur.push_str("}}");
            i += 2;
            continue;
        }
        if rest.starts_with("[[") {
            depth_l += 1;
            cur.push_str("[[");
            i += 2;
            continue;
        }
        if rest.starts_with("]]") {
            depth_l = depth_l.saturating_sub(1);
            cur.push_str("]]");
            i += 2;
            continue;
        }
        if b[i] == b'|' && depth_t == 0 && depth_l == 0 {
            parts.push(std::mem::take(&mut cur));
            i += 1;
            continue;
        }
        let ch = rest.chars().next()?;
        cur.push(ch);
        i += ch.len_utf8();
    }
    None
}

/// A piece of wikitext as [`template_segments`] cuts it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Segment<'a> {
    /// Text between templates, verbatim. A `{{` that opens nothing is text as well.
    Text(&'a str),
    /// A top-level template: parsed, the source it was parsed from, and everything after it.
    Template {
        template: Template,
        source: &'a str,
        after: &'a str,
    },
}

/// `text` cut into its top-level templates and the text between them, in order. Laid end to
/// end, the segments' `Text` and `source` give `text` back byte for byte.
///
/// It is the one answer to "where does each template begin and end", which a line-by-line pass
/// cannot give: an infobox spans a dozen lines. Four passes each carried their own loop of
/// "find `{{`, try `parse_template_at`, step over a `{{` that closes nothing" — the infobox
/// extraction, the preamble's cleanup, the block wrappers and the heading normalization. A
/// template nested inside another one is part of the outer one's `source`, never a segment.
pub(crate) fn template_segments(text: &str) -> impl Iterator<Item = Segment<'_>> {
    let mut cursor = 0;
    std::iter::from_fn(move || {
        let (segment, len) = segment_at(text, cursor)?;
        cursor += len;
        Some(segment)
    })
}

/// The segment that starts at `at`, and how many bytes it spans; `None` at the end of `text`.
fn segment_at(text: &str, at: usize) -> Option<(Segment<'_>, usize)> {
    let rest = text.get(at..).filter(|rest| !rest.is_empty())?;
    Some(match rest.find("{{") {
        None => (Segment::Text(rest), rest.len()),
        Some(0) => match parse_template_at(text, at) {
            Some((template, end)) => (
                Segment::Template {
                    template,
                    source: text.get(at..end)?,
                    after: text.get(end..)?,
                },
                end - at,
            ),
            // A `{{` with no close: text, and the scan resumes right after it.
            None => (Segment::Text(rest.get(..2)?), 2),
        },
        Some(pos) => (Segment::Text(rest.get(..pos)?), pos),
    })
}

fn assemble(parts: Vec<String>) -> Template {
    let mut it = parts.into_iter();
    let name = it.next().unwrap_or_default().trim().to_lowercase();
    let mut args = Vec::new();
    let mut named = BTreeMap::new();
    for p in it {
        // `k=v` only if `k` doesn't contain `{{`/`[[`: an `=` inside a link isn't a name.
        match p.split_once('=') {
            Some((k, v)) if !k.contains("{{") && !k.contains("[[") && !k.trim().is_empty() => {
                let k = k.trim();
                let v = v.trim().to_string();
                // A name that is a number is MediaWiki's explicit positional syntax:
                // `{{i|1=Bird's Eye}}` is `{{i|Bird's Eye}}`. Filed under `named` instead,
                // it leaves `args` empty and the resolver with nothing to resolve.
                //
                // Accepted only for the slot right after the last one, or one already
                // filled. The index comes from external wikitext, and a rule that
                // honoured any number would let `{{x|999999999=y}}` ask for a vector of a
                // billion empty slots. An index that leaves a gap keeps its named form,
                // which is what it looks like anyway — degrade, don't allocate.
                match k.parse::<usize>() {
                    Ok(n) if (1..=args.len() + 1).contains(&n) => match args.get_mut(n - 1) {
                        Some(slot) => *slot = v,
                        None => args.push(v),
                    },
                    _ => {
                        named.insert(k.to_lowercase(), v);
                    }
                }
            }
            _ => args.push(p.trim().to_string()),
        }
    }
    Template { name, args, named }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(s: &str) -> Template {
        parse_template_at(s, 0).unwrap().0
    }

    #[test]
    fn name_and_positional_args() {
        let x = t("{{i|Little Baggy|Baggy}}");
        assert_eq!(x.name, "i");
        assert_eq!(x.args, vec!["Little Baggy", "Baggy"]);
        assert!(x.named.is_empty());
    }

    /// MediaWiki's explicit positional syntax: an argument whose *name* is a number is
    /// the positional argument at that index. `{{i|1=Bird's Eye}}` means `{{i|Bird's Eye}}`.
    /// Read as an ordinary `k=v` it lands in `named` and leaves `args` empty, so the
    /// resolver is handed nothing and the reference comes out unresolved — which is where
    /// three of the snapshot's unresolved `{{i|…}}` came from.
    #[test]
    fn a_numeric_name_is_the_positional_argument_at_that_index() {
        assert_eq!(t("{{i|1=Bird's Eye}}").args, vec!["Bird's Eye"]);
        assert!(t("{{i|1=Bird's Eye}}").named.is_empty());
        // It can also follow a plain positional one, and keeps its place.
        let two = t("{{x|first|2=second}}");
        assert_eq!(two.args, vec!["first", "second"]);
        // A name that isn't a number stays a named argument.
        assert_eq!(
            t("{{dlc|dlc=na}}").named.get("dlc").map(String::as_str),
            Some("na")
        );
    }

    #[test]
    fn named_args_and_case() {
        let x = t("{{ I | Little Baggy | text = Baggy }}");
        assert_eq!(x.name, "i");
        assert_eq!(x.args, vec!["Little Baggy"]);
        assert_eq!(x.named.get("text").map(String::as_str), Some("Baggy"));
    }

    #[test]
    fn nested_templates_and_links_are_not_split() {
        let x = t("{{dlc+|a+}} Entrance to {{s|The Void}} [[a|b]] {{dlc-}}");
        // parse_template_at only reads the first template
        assert_eq!(x.name, "dlc+");
        assert_eq!(x.args, vec!["a+"]);
        let (y, end) = parse_template_at("{{bug|{{i|A|x=y}} and [[p|q]]|dlc=r}} tail", 0).unwrap();
        assert_eq!(y.args, vec!["{{i|A|x=y}} and [[p|q]]"]);
        assert_eq!(y.named.get("dlc").map(String::as_str), Some("r"));
        assert_eq!(
            &"{{bug|{{i|A|x=y}} and [[p|q]]|dlc=r}} tail"[end..],
            " tail"
        );
    }

    #[test]
    fn multiline_infobox() {
        let src = "{{infobox boss\n | dlc = a\n | id = 407\n | base hp = 6666\n}}\n'''Hush'''";
        let (x, end) = parse_template_at(src, 0).unwrap();
        assert_eq!(x.name, "infobox boss");
        assert_eq!(x.named.get("base hp").map(String::as_str), Some("6666"));
        assert_eq!(&src[end..], "\n'''Hush'''");
    }

    #[test]
    fn unterminated_returns_none() {
        assert!(parse_template_at("{{i|Breakfast", 0).is_none());
        assert!(parse_template_at("no template", 0).is_none());
    }

    #[test]
    fn parser_functions_keep_the_hash_name() {
        let x = t("{{#ev:youtube|dFwYucBWQ9k}}");
        assert_eq!(x.name, "#ev:youtube");
    }

    #[test]
    fn offset_past_the_end_or_inside_a_char_returns_none() {
        assert!(parse_template_at("{{i|x}}", 99).is_none());
        // `at` in the middle of a multibyte character: must not panic
        assert!(parse_template_at("é{{i|x}}", 1).is_none());
    }

    fn joined(text: &str) -> String {
        template_segments(text)
            .map(|s| match s {
                Segment::Text(t) => t,
                Segment::Template { source, .. } => source,
            })
            .collect()
    }

    /// The segments are a cut, not a rewrite: laid end to end they are the input, whatever
    /// the input — a nested template, an unclosed one, a multibyte character next to a brace.
    #[test]
    fn the_segments_laid_end_to_end_are_the_text() {
        for text in [
            "",
            "plain",
            "a {{i|x}} b {{dlc|r|{{i|y}}}} c",
            "{{i|Breakfast",
            "é{{ {{i|x}}}} }}é",
            "{{infobox boss\n | id = 1\n}}\n'''Hush'''",
        ] {
            assert_eq!(joined(text), text, "{text:?}");
        }
    }

    /// Only top-level templates are segments, each with what follows it; a `{{` that closes
    /// nothing is text, and the template after it is still found.
    #[test]
    fn a_segment_is_a_top_level_template_or_the_text_around_it() {
        let segments: Vec<Segment> = template_segments("a {{bug|{{i|x}}}} {{ b {{i|y}}").collect();
        let names: Vec<&str> = segments
            .iter()
            .filter_map(|s| match s {
                Segment::Template { template, .. } => Some(template.name.as_str()),
                Segment::Text(_) => None,
            })
            .collect();
        assert_eq!(names, vec!["bug", "i"]);
        assert!(matches!(
            &segments[1],
            Segment::Template {
                source: "{{bug|{{i|x}}}}",
                after: " {{ b {{i|y}}",
                ..
            }
        ));
    }

    #[test]
    fn parses_from_a_later_offset() {
        let src = "testo {{i|Gimpy}} coda";
        let (x, end) = parse_template_at(src, 6).unwrap();
        assert_eq!(x.args, vec!["Gimpy"]);
        assert_eq!(&src[end..], " coda");
    }
}
