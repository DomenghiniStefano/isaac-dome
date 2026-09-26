//! From a line of wikitext to `Vec<Inline>`. Single-pass scan with a stack of open
//! `Edition`s; text accumulates and is emitted on a style or node change.
//!
//! The input is external data: no path may panic. Every malformed construct (an
//! unclosed template, an open link, a tag with no `>`) degrades to text.

mod name_list;
mod out;

pub use name_list::name_list_items;

use crate::editions::span_restriction;
use crate::resolver::{Resolution, Resolver};
use crate::template::{parse_template_at, Template};
use crate::{Diagnostics, Inline};
use name_list::NameList;
use out::Out;

/// A closed list, and deliberately not a general HTML-entity decoder: the input is
/// wikitext, not HTML, and a decoder that also ate `&amp;lt;` would be inventing a rule
/// nobody measured. What is outside the list is counted by the caller, not guessed at.
///
/// The last three are the ones the wiki writes to protect **template syntax**: a comma or a
/// colon inside a template argument would be read as a separator, an apostrophe would run
/// into italic markup. MediaWiki decodes them when it renders, and until 2026-09-14 this
/// parser passed them through, so three pickup quotes shipped reading `&comma;`.
fn entity(name: &str) -> Option<&'static str> {
    Some(match name {
        "nbsp" | "#32" => " ",
        "times" => "×",
        "amp" => "&",
        "lt" => "<",
        "gt" => ">",
        "quot" => "\"",
        "ndash" => "–",
        "mdash" => "—",
        "comma" => ",",
        "colon" => ":",
        "apos" => "'",
        _ => return None, // allowed: HTML entity, an open-ended set
    })
}

/// Whether a run between `&` and `;` is shaped like an entity name at all — a letter then
/// letters and digits, or `#` and digits. Without this an ordinary `&` in a sentence
/// ("R&D; more") would be counted as an entity nobody knows, and the counter that exists to
/// show a real gap would fill up with prose.
fn looks_like_entity_name(name: &str) -> bool {
    if let Some(digits) = name.strip_prefix('#') {
        return !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit());
    }
    let mut cs = name.bytes();
    cs.next().is_some_and(|b| b.is_ascii_alphabetic()) && cs.all(|b| b.is_ascii_alphanumeric())
}

/// A reference's label: `text=`, then the second positional argument if it isn't a `k=v`
/// pair that slipped past the parser, otherwise the argument itself.
fn label_of(t: &Template, arg: &str) -> String {
    t.named
        .get("text")
        .cloned()
        .or_else(|| {
            t.args
                .get(1)
                .filter(|a| !a.is_empty() && !a.contains('='))
                .cloned()
        })
        .unwrap_or_else(|| arg.to_string())
}

/// The name of an HTML tag from whatever is between `<` and `>`: without the closing `/`,
/// lowercased, empty if it isn't a tag (`< 10`).
fn tag_name(tag: &str) -> String {
    let name = tag
        .trim_start_matches('/')
        .split(|c: char| c.is_whitespace() || c == '/')
        .next()
        .unwrap_or("");
    if name.starts_with(|c: char| c.is_ascii_alphabetic()) {
        name.to_ascii_lowercase()
    } else {
        String::new()
    }
}

/// Beyond how many nested `{{…}}` recursion is abandoned and the raw text is kept: real
/// nesting in the wiki never goes past a few levels, and without a ceiling a pathological
/// input (a self-containing template) would grow the stack without end.
const MAX_TEMPLATE_DEPTH: u32 = 8;

/// Known templates whose first argument is genuine content, not text (`{{bug|...}}` wraps
/// a sentence that can contain other templates): they behave like an unknown template for
/// recursion purposes, but they are known and must not be counted in `unknown_templates`.
const CONTENT_WRAPPERS: &[&str] = &["bug"];

pub fn parse_inline(src: &str, r: &Resolver, d: &mut Diagnostics) -> Vec<Inline> {
    parse_inline_at(src, r, d, 0)
}

fn parse_inline_at(src: &str, r: &Resolver, d: &mut Diagnostics, depth: u32) -> Vec<Inline> {
    let mut out = Out::new();
    let mut i = 0;
    while i < src.len() {
        i += step(src, i, r, d, depth, &mut out);
    }
    out.finish()
}

/// The construct that starts at `at`, pushed into `out`, and how many bytes it took. The
/// order is the rule: comments first, because the template parser doesn't know about them;
/// a character of plain text when nothing else matches.
fn step(
    src: &str,
    at: usize,
    r: &Resolver,
    d: &mut Diagnostics,
    depth: u32,
    out: &mut Out,
) -> usize {
    let Some(rest) = src.get(at..).filter(|rest| !rest.is_empty()) else {
        return src.len().saturating_sub(at).max(1);
    };
    try_comment(rest)
        .or_else(|| try_style(rest, out))
        .or_else(|| try_template(src, at, r, d, out, depth))
        .or_else(|| try_link(rest, r, out))
        .or_else(|| try_tag(rest, out))
        .or_else(|| try_entity(rest, d, out))
        .unwrap_or_else(|| text_char(rest, out))
}

/// `<!-- … -->`, dropped whole. An unclosed one is text.
fn try_comment(rest: &str) -> Option<usize> {
    let end = rest.strip_prefix("<!--")?.find("-->")?;
    Some("<!--".len() + end + "-->".len())
}

/// Three quotes toggle bold, two italic; the text so far keeps the style it was written in.
fn try_style(rest: &str, out: &mut Out) -> Option<usize> {
    if rest.starts_with("'''") {
        out.flush();
        out.bold = !out.bold;
        return Some(3);
    }
    if rest.starts_with("''") {
        out.flush();
        out.italic = !out.italic;
        return Some(2);
    }
    None
}

/// A template at `at`. A `{{` that closes nothing is text, and the scan resumes after it.
fn try_template(
    src: &str,
    at: usize,
    r: &Resolver,
    d: &mut Diagnostics,
    out: &mut Out,
    depth: u32,
) -> Option<usize> {
    if !src.get(at..)?.starts_with("{{") {
        return None;
    }
    let Some((t, end)) = parse_template_at(src, at) else {
        out.buf.push_str("{{");
        return Some(2);
    };
    template(&t, r, d, out, depth);
    Some(end - at)
}

/// `[[…]]`, up to the first `]]`. A `[[` that closes nothing is text.
fn try_link(rest: &str, r: &Resolver, out: &mut Out) -> Option<usize> {
    if !rest.starts_with("[[") {
        return None;
    }
    let Some((inner, end)) = rest
        .find("]]")
        .and_then(|end| Some((rest.get(2..end)?, end)))
    else {
        out.buf.push_str("[[");
        return Some(2);
    };
    link(inner, r, out);
    Some(end + 2)
}

/// An HTML tag: `<br>` is a space, `<ref>…</ref>` disappears whole (without a closing tag only
/// the tag does), and every other tag is gone while its content stays. Something between `<`
/// and `>` that is not a tag's name (`< 10`) is text.
fn try_tag(rest: &str, out: &mut Out) -> Option<usize> {
    if !rest.starts_with('<') {
        return None;
    }
    let end = rest.find('>')?;
    let tag = rest.get(1..end)?;
    let name = tag_name(tag);
    if name.is_empty() {
        return None;
    }
    let after_tag = end + 1;
    if name == "br" {
        out.buf.push(' ');
        return Some(after_tag);
    }
    if name == "ref" && !tag.ends_with('/') && !tag.starts_with('/') {
        let close = rest.get(after_tag..).and_then(|s| s.find("</ref>"));
        return Some(close.map_or(after_tag, |close| after_tag + close + "</ref>".len()));
    }
    Some(after_tag)
}

/// `&name;` from the closed list, decoded. One shaped like an entity and outside the list is
/// counted, and its text kept: it is the wiki's content, and dropping a run because we do not
/// know one name in it would destroy more than it fixes. The counter is what makes the next
/// one visible instead of shipped — which is how the three at the bottom of the list shipped.
fn try_entity(rest: &str, d: &mut Diagnostics, out: &mut Out) -> Option<usize> {
    let end = rest.strip_prefix('&')?.find(';').filter(|e| *e <= 8)?;
    let name = rest
        .get(1..1 + end)
        .filter(|name| looks_like_entity_name(name))?;
    match entity(name) {
        Some(rep) => {
            out.buf.push_str(rep);
            Some(end + 2)
        }
        None => {
            d.unknown_entity(name);
            None
        }
    }
}

/// One character of plain text.
fn text_char(rest: &str, out: &mut Out) -> usize {
    match rest.chars().next() {
        Some(ch) => {
            out.text_char(ch);
            ch.len_utf8()
        }
        None => rest.len().max(1),
    }
}

/// An already-parsed template: editions, `{{!}}`, or a reference passed to the resolver.
fn template(t: &Template, r: &Resolver, d: &mut Diagnostics, out: &mut Out, depth: u32) {
    // A list of names inside a sentence. A line holding nothing else never gets here:
    // `blocks` makes it a list first.
    if let Some(list) = NameList::of(&t.name) {
        list.push_inline(t, r, d, out);
        return;
    }
    let arg = t.args.first().cloned().unwrap_or_default();
    match t.name.as_str() {
        "!" => out.buf.push('|'),
        // Two shapes share this name. `{{dlc|r}}` is a marker: it opens a scope that runs to
        // the end of the value or to `{{dlc-}}`. `{{dlc|r|text}}` carries its own text and
        // closes itself — and until 2026-09-13 that second argument was never read, so 313
        // spans across the snapshot lost their words without a diagnostic.
        "dlc+" | "dlc" => match t.args.get(1) {
            Some(content) => {
                out.open(span_restriction(&arg, d));
                recurse_into_arg(content, r, d, out, depth);
                out.close();
            }
            None => out.open_marker(span_restriction(&arg, d)),
        },
        "dlc-" => out.close(),
        "dlcalt" => dlcalt(t, &arg, r, d, out, depth),
        "bc" => champion(t, &arg, d, out),
        // Two templates, one shape: the item is in the name and the text is in a `description`
        // parameter, spelled that way in all 197 uses. The label is written out per arm rather
        // than derived from the name, so a third "X synergy" template cannot silently inherit
        // the wrong item.
        "book of virtues synergy" => synergy("Book of Virtues", t, r, d, out, depth),
        // With the article: the item's page is "The Book of Belial", and without it the
        // resolver answers nothing — 33 uses lost the reference, and the ": " that introduces
        // the description with it.
        "book of belial synergy" => synergy("The Book of Belial", t, r, d, out, depth),
        // 204 of the 547 `{{bug|…}}` carry a `dlc`, and until 2026-09-15 this arm recursed
        // into the positional argument and read no named one: a defect that exists in one
        // edition was shown to every reader as theirs. The 130 whose code this parser
        // cannot read open an edition that names none, which `Out::close` unwraps — the
        // sentence keeps every word and the code is counted.
        name if CONTENT_WRAPPERS.contains(&name) => {
            out.open(
                t.named
                    .get("dlc")
                    .map_or_else(Vec::new, |c| span_restriction(c, d)),
            );
            recurse_into_arg(&arg, r, d, out, depth);
            out.close();
        }
        name => reference(name, t, arg, r, d, out, depth),
    }
}

/// `{{dlcalt|…|r=…}}`. Both the positional argument and every per-edition variant are content
/// (the wiki nests other templates in them, `{{p|Soul of Lazarus}}` included): same recursion
/// as unknown templates, not raw text.
fn dlcalt(t: &Template, arg: &str, r: &Resolver, d: &mut Diagnostics, out: &mut Out, depth: u32) {
    recurse_into_arg(arg, r, d, out, depth);
    for (code, text) in &t.named {
        out.open(span_restriction(code, d));
        out.buf.push(' ');
        recurse_into_arg(text, r, d, out, depth);
        out.close();
    }
}

/// `{{bc|…}}`, a boss's champion variant, always under `== Champion Versions ==`. The number is
/// the variant's index; which colour each index is lives in the wiki's own template and nowhere
/// we can read, and `catalog` has no champion table — so the index is kept verbatim and nothing
/// is invented around it. `dlc=` makes the variant belong to one edition, which is what
/// `Inline::Edition` already says.
fn champion(t: &Template, arg: &str, d: &mut Diagnostics, out: &mut Out) {
    let index = arg.trim();
    with_optional_edition(out, d, t.named.get("dlc"), |out, _| {
        out.push(Inline::Concept {
            page: "Champion".to_string(),
            label: format!("Champion {index}"),
        })
    });
}

/// `{{Book of Virtues synergy|description=…}}` and its twin. The text is in a *named*
/// parameter, which neither `resolve` nor the positional recursion reaches: 157 of these
/// sentences used to arrive empty. The item is named too, because the template's meaning is
/// "with Book of Virtues, this happens" and a section shown on its own would otherwise lose the
/// half that says with what.
fn synergy(item: &str, t: &Template, r: &Resolver, d: &mut Diagnostics, out: &mut Out, depth: u32) {
    if let Resolution::Target(target) = r.resolve("i", item) {
        out.push(Inline::Ref {
            target,
            label: item.to_string(),
        });
        out.buf.push_str(": ");
    }
    if let Some(description) = t.named.get("description") {
        recurse_into_arg(description, r, d, out, depth);
    }
}

/// `body` inside the edition `code` names, or on its own when there is no code.
fn with_optional_edition(
    out: &mut Out,
    d: &mut Diagnostics,
    code: Option<&String>,
    body: impl FnOnce(&mut Out, &mut Diagnostics),
) {
    let Some(code) = code else {
        body(out, d);
        return;
    };
    out.open(span_restriction(code, d));
    body(out, d);
    out.close();
}

/// Any other template, handed to the resolver: a reference, a concept, a miss that is counted,
/// or an unknown template whose argument is read as content.
fn reference(
    name: &str,
    t: &Template,
    arg: String,
    r: &Resolver,
    d: &mut Diagnostics,
    out: &mut Out,
    depth: u32,
) {
    match r.resolve(name, &arg) {
        Resolution::Target(target) => {
            let label = label_of(t, &arg);
            out.push(Inline::Ref { target, label });
        }
        Resolution::Concept => {
            let label = label_of(t, &arg);
            out.push(Inline::Concept { page: arg, label });
        }
        Resolution::Unresolved => {
            d.unresolved(name);
            let label = label_of(t, &arg);
            out.push(Inline::Concept { page: arg, label });
        }
        Resolution::Unknown => {
            d.unknown_template(name);
            recurse_into_arg(&arg, r, d, out, depth);
        }
        Resolution::Ignore => {}
    }
}

/// The first argument of an unknown template (or of a known wrapper like `{{bug}}`) is
/// content, not raw text: parsing restarts on it from scratch, with the same resolver and
/// the same diagnostics, instead of leaving it as a literal `{{…}}`. Beyond
/// `MAX_TEMPLATE_DEPTH` levels this is abandoned and the text stays raw.
fn recurse_into_arg(arg: &str, r: &Resolver, d: &mut Diagnostics, out: &mut Out, depth: u32) {
    if depth >= MAX_TEMPLATE_DEPTH {
        out.buf.push_str(arg);
        return;
    }
    for node in parse_inline_at(arg, r, d, depth + 1) {
        out.push(node);
    }
}

/// The content of `[[…]]`: `File:`, `Image:` and `Category:` disappear without leaving
/// any text; `[[:Page]]` is a concept like any other.
fn link(inner: &str, r: &Resolver, out: &mut Out) {
    let (page, label) = match inner.split_once('|') {
        Some((p, l)) => (p, l),
        None => (inner, inner),
    };
    let lower = page.to_ascii_lowercase();
    if lower.starts_with("file:") || lower.starts_with("category:") || lower.starts_with("image:") {
        return;
    }
    let page = page.trim_start_matches(':');
    let page = page.split('#').next().unwrap_or(page).trim().to_string();
    let label = label.split('|').next().unwrap_or(label).trim().to_string();
    if page.is_empty() {
        out.buf.push_str(&label);
        return;
    }
    // A wikilink to the page of something the game knows by id is a reference just like
    // `{{i|…}}`; a concept page (`[[Shot Speed]]`) stays a concept.
    match r.by_page_title(&page) {
        Some(target) => out.push(Inline::Ref { target, label }),
        None => out.push(Inline::Concept { page, label }),
    }
}

/// An inline run as **the reader would read it**: a reference and a concept become their
/// label, an edition wrapper is unwrapped and its words kept.
///
/// It is the one place that answers "what does this say, as a sentence". `infobox::tags`
/// used a private copy; a caller outside this crate needs the same answer for an
/// achievement's requirement, and two copies of a rule are wrong within a release.
///
/// What it deliberately loses: style, which page a reference points at, and which editions a
/// wrapper declared. Anything that needs those reads the tree.
pub fn plain(inline: &[Inline]) -> String {
    let mut out = String::new();
    write_plain(inline, &mut out);
    out
}

fn write_plain(inline: &[Inline], out: &mut String) {
    for i in inline {
        match i {
            Inline::Text { text, .. } => out.push_str(text),
            Inline::Ref { label, .. } | Inline::Concept { label, .. } => out.push_str(label),
            Inline::Edition { inline, .. } => write_plain(inline, out),
        }
    }
}

// Tests extract one variant and panic on the rest: the wildcard is the assertion.
#[allow(clippy::wildcard_enum_match_arm)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::fixtures::test_resolver;
    use crate::{Diagnostics, Dlc, Inline, Style, Target};

    fn p(s: &str) -> (Vec<Inline>, Diagnostics) {
        let r = test_resolver();
        let mut d = Diagnostics::default();
        (parse_inline(s, &r, &mut d), d)
    }

    fn text(t: &str, style: Style) -> Inline {
        Inline::Text {
            text: t.into(),
            style,
        }
    }

    /// The three entities that reach `dataset/wiki.json`, measured on the snapshot of
    /// 2026-09-13: item 469 reads `&colon;(`, item 601 "Tears up&comma; you feel forgiven",
    /// trinket 138 "t&apos;s broken". All three are pickup quotes, and the wiki writes them
    /// to protect a comma or a colon inside a template argument and an apostrophe against
    /// italic markup. MediaWiki decodes them when it renders; this parser passed them
    /// through, so they reached the screen as `&comma;`.
    #[test]
    fn the_entities_that_protect_template_syntax_are_decoded() {
        let (v, d) = p("&colon;( Tears up&comma; you feel forgiven t&apos;s broken");
        assert_eq!(
            v,
            vec![text(
                ":( Tears up, you feel forgiven t's broken",
                Style::Plain
            )]
        );
        assert!(d.unknown_entities.is_empty(), "{:?}", d.unknown_entities);
    }

    /// An entity the closed list does not cover is **kept and counted**, never dropped: the
    /// text is the wiki's content and `&` is an ordinary character in it. Counting is what
    /// makes the next one visible instead of shipped, which is the whole of B38.
    #[test]
    fn an_entity_outside_the_closed_list_is_counted_and_kept() {
        let (v, d) = p("a &hearts; b");
        assert_eq!(v, vec![text("a &hearts; b", Style::Plain)]);
        assert_eq!(d.unknown_entities.get("hearts"), Some(&1));
    }

    #[test]
    fn plain_bold_italic() {
        let (v, _) = p("a '''b''' c ''d''");
        assert_eq!(
            v,
            vec![
                text("a ", Style::Plain),
                text("b", Style::Bold),
                text(" c ", Style::Plain),
                text("d", Style::Italic)
            ]
        );
    }

    #[test]
    fn refs_concepts_and_labels() {
        let (v, d) = p("{{i|Breakfast}}: +0.2 [[Shot Speed]], see {{i|Tonsil|the tonsil}} and [[Pills#Horse Pills|horse pill]] {{I|Nope}}");
        assert_eq!(
            v[0],
            Inline::Ref {
                target: Target::Item { id: 25 },
                label: "Breakfast".into()
            }
        );
        assert_eq!(v[1], text(": +0.2 ", Style::Plain));
        assert_eq!(
            v[2],
            Inline::Concept {
                page: "Shot Speed".into(),
                label: "Shot Speed".into()
            }
        );
        // Tonsil is a trinket: as an item it doesn't exist in Repentance+, so
        // `{{i|Tonsil}}` stays a concept, with its own unresolved count.
        assert_eq!(
            v[4],
            Inline::Concept {
                page: "Tonsil".into(),
                label: "the tonsil".into()
            }
        );
        assert_eq!(
            v[6],
            Inline::Concept {
                page: "Pills".into(),
                label: "horse pill".into()
            }
        );
        assert_eq!(
            v[8],
            Inline::Concept {
                page: "Nope".into(),
                label: "Nope".into()
            }
        );
        assert_eq!(d.unresolved.get("i"), Some(&2));
    }

    #[test]
    fn wikilinks_to_known_pages_are_refs() {
        let (v, d) = p("[[Breakfast]] [[:Breakfast#Notes|brekkie]] [[The Family Man|challenge #19]] [[Shot Speed]]");
        assert_eq!(
            v[0],
            Inline::Ref {
                target: Target::Item { id: 25 },
                label: "Breakfast".into()
            }
        );
        assert_eq!(
            v[2],
            Inline::Ref {
                target: Target::Item { id: 25 },
                label: "brekkie".into()
            }
        );
        assert_eq!(
            v[4],
            Inline::Ref {
                target: Target::Challenge { number: 19 },
                label: "challenge #19".into()
            }
        );
        assert_eq!(
            v[6],
            Inline::Concept {
                page: "Shot Speed".into(),
                label: "Shot Speed".into()
            }
        );
        assert!(d.unresolved.is_empty());
    }

    #[test]
    fn unknown_and_layout_templates() {
        // A name no template will ever have: this test is about what happens to a template
        // we do not know, and using a real one means it breaks the day we learn that one.
        // It used to use `{{m|…}}`, which is exactly what happened on 2026-09-13.
        let (v, d) = p("{{cit|p|r}}x {{notatemplate|Donation Machine}} y");
        assert_eq!(v, vec![text("x Donation Machine y", Style::Plain)]);
        assert_eq!(d.unknown_templates.get("notatemplate"), Some(&1));
        assert!(d.unresolved.is_empty());
    }

    #[test]
    fn unknown_template_argument_is_parsed_recursively() {
        // `column list` is unknown to the resolver: its first positional argument is
        // genuine content and must be parsed, not left as raw text.
        let (v, d) = p("{{column list|* a {{i|Breakfast}}}}");
        assert_eq!(
            v,
            vec![
                text("* a ", Style::Plain),
                Inline::Ref {
                    target: Target::Item { id: 25 },
                    label: "Breakfast".into()
                }
            ]
        );
        assert_eq!(d.unknown_templates.get("column list"), Some(&1));
    }

    #[test]
    fn bug_wrapper_is_parsed_recursively_and_not_counted_as_unknown() {
        let (v, d) = p("{{bug|Using {{i|Breakfast}} here}}");
        assert_eq!(
            v,
            vec![
                text("Using ", Style::Plain),
                Inline::Ref {
                    target: Target::Item { id: 25 },
                    label: "Breakfast".into()
                },
                text(" here", Style::Plain),
            ]
        );
        assert!(!d.unknown_templates.contains_key("bug"));
        assert!(d.unresolved.is_empty());
    }

    #[test]
    fn recursion_depth_is_capped() {
        // A self-containing unknown template must not grow the stack: beyond
        // `MAX_TEMPLATE_DEPTH` levels the text stays raw instead of recursing further.
        let mut src = "{{bug|x}}".to_string();
        for _ in 0..20 {
            src = format!("{{{{bug|{src}}}}}");
        }
        let (v, _) = p(&src);
        // Must not panic or loop: it's enough that it produces something.
        assert!(!v.is_empty());
    }

    #[test]
    fn editions() {
        let (v, _) = p("A {{dlc+|r}}only rep {{i|Breakfast}}{{dlc-}} B");
        assert_eq!(
            v,
            vec![
                text("A ", Style::Plain),
                Inline::Edition {
                    only: vec![Dlc::Repentance, Dlc::RepentancePlus],
                    inline: vec![
                        text("only rep ", Style::Plain),
                        Inline::Ref {
                            target: Target::Item { id: 25 },
                            label: "Breakfast".into()
                        },
                    ]
                },
                text(" B", Style::Plain),
            ]
        );
        let (v, _) = p("{{dlc|a+}} Isaac also starts with X");
        assert_eq!(
            v,
            vec![Inline::Edition {
                only: vec![Dlc::AfterbirthPlus, Dlc::Repentance, Dlc::RepentancePlus],
                inline: vec![text(" Isaac also starts with X", Style::Plain)]
            }]
        );
        let (v, _) = p("{{dlcalt|17.75|r=4.5}}");
        assert_eq!(
            v,
            vec![
                text("17.75", Style::Plain),
                Inline::Edition {
                    only: vec![Dlc::Repentance, Dlc::RepentancePlus],
                    inline: vec![text(" 4.5", Style::Plain)]
                }
            ]
        );
    }

    /// `{{m|Donation Machine}}` and `{{machine|Greed Donation Machine}}` name a machine or a
    /// beggar: a wiki page the game gives no id, which is exactly `Inline::Concept`. Together
    /// they were the largest entry in `unknownTemplates` — 373 and 52 — so their sentences
    /// reached the frontend with the name in place but no link and a diagnostic against them.
    ///
    /// They must not be counted as *unresolved*: that word means "we looked for an id and did
    /// not find one", and here there was never an id to find.
    #[test]
    fn machine_templates_become_concept_links() {
        let (v, d) = p("Use the {{m|Donation Machine}} here");
        assert!(v.iter().any(|i| matches!(
            i,
            Inline::Concept { page, label }
                if page == "Donation Machine" && label == "Donation Machine"
        )));
        assert!(d.unknown_templates.is_empty(), "{:?}", d.unknown_templates);
        assert!(d.unresolved.is_empty(), "a concept is not a failed lookup");

        let (v, d) = p("{{machine|Greed Donation Machine}}");
        assert!(v.iter().any(
            |i| matches!(i, Inline::Concept { page, .. } if page == "Greed Donation Machine")
        ));
        assert!(d.unknown_templates.is_empty());

        // The label still wins when the page is written under another name.
        let (v, _) = p("{{m|Blood Donation Machine|the machine}}");
        assert!(v.iter().any(|i| matches!(
            i,
            Inline::Concept { page, label }
                if page == "Blood Donation Machine" && label == "the machine"
        )));
    }

    /// `{{bc|7}}` marks a boss's champion variant. All 59 uses sit under
    /// `== Champion Versions ==`, and the number is the variant's index — the wiki renders it
    /// as a coloured swatch, and **which colour each index is cannot be read from anything we
    /// have**: it lives in the wiki's own template, and `catalog` has no champion table.
    ///
    /// So the number is kept as the wiki wrote it and the concept is named, which is the
    /// whole of what is known. Giving index 7 a colour name would be the kind of guess this
    /// repo keeps paying for.
    #[test]
    fn the_champion_template_names_the_variant_without_naming_its_colour() {
        let (v, d) = p("{{bc|7}}: 15% larger and slower");
        assert!(
            v.iter().any(|i| matches!(
                i,
                Inline::Concept { page, label } if page == "Champion" && label == "Champion 7"
            )),
            "got {v:?}"
        );
        assert!(d.unknown_templates.is_empty(), "{:?}", d.unknown_templates);

        // `{{bc|18|dlc=a+}}` says the variant belongs to an edition range: that is
        // `Inline::Edition`, the same node every other per-edition span uses. `a+` is
        // "added in Afterbirth †", so the range runs from there to the end.
        let (v, _) = p("{{bc|18|dlc=a+}}");
        assert!(
            v.iter().any(|i| matches!(
                i,
                Inline::Edition { only, .. } if only == &vec![Dlc::AfterbirthPlus, Dlc::Repentance, Dlc::RepentancePlus]
            )),
            "got {v:?}"
        );
    }

    /// A code names a range, and `n` is the half of it that says *removed*. `nr` is the
    /// commonest code in the corpus — 1371 of **4831** uses — and it means "removed in
    /// Repentance", so it names the three editions before it.
    ///
    /// 4831 counts both spellings: 4168 `{{dlc` and 663 `{{Dlc`, which `template.rs`
    /// lowercases before matching, so the parser sees them alike. B52's own figures are the
    /// lowercase subset and say 4168; the two are not a disagreement, and this is the note
    /// that says which is which.
    ///
    /// It named none of them until 2026-09-15: this parser read whole codes one at a time,
    /// so 1734 uses opened a frame with an empty `only`, and **1690 of those reached
    /// `wiki.json`**, where `WikiInline.vue` draws no badge (`v-if="token.only.length"`)
    /// and the text arrives saying nothing about which edition it belongs to.
    #[test]
    fn a_removal_code_names_the_editions_before_it() {
        let (v, d) = p("{{dlc|nr|Gone in Repentance}}");
        assert!(
            v.iter().any(|i| matches!(
                i,
                Inline::Edition { only, .. }
                    if only == &vec![Dlc::Rebirth, Dlc::Afterbirth, Dlc::AfterbirthPlus]
            )),
            "got {v:?}"
        );
        assert!(d.unknown_dlc_codes.is_empty());
    }

    /// `{{dlc|n}}` is the wiki's code for "no restriction" — row 31, beside `x` and the
    /// empty string — and the wiki draws no icon for it. A span restricted to all five
    /// editions is a span restricted to none, so the words come back out of the frame and
    /// no badge is emitted. Zero uses in the corpus: this is the guard, not a fix.
    ///
    /// The same shape carries what could **not** be read: an unreadable code restricts
    /// nothing either, and is counted, because a span declaring itself valid nowhere is
    /// worse than the same span unwrapped.
    #[test]
    fn a_span_restricted_to_every_edition_draws_no_badge_and_an_unreadable_one_is_counted() {
        let (v, d) = p("{{dlc|n|Everywhere}}");
        assert!(
            !v.iter().any(|i| matches!(i, Inline::Edition { .. })),
            "a badge naming all five: {v:?}"
        );
        assert!(
            v.iter()
                .any(|i| matches!(i, Inline::Text { text, .. } if text.contains("Everywhere"))),
            "the words went with it: {v:?}"
        );
        assert!(d.unknown_dlc_codes.is_empty());

        let (v, d) = p("{{dlc|zz|Unreadable}}");
        assert!(
            !v.iter().any(|i| matches!(i, Inline::Edition { .. })),
            "got {v:?}"
        );
        assert!(
            v.iter()
                .any(|i| matches!(i, Inline::Text { text, .. } if text.contains("Unreadable"))),
            "the words went with it: {v:?}"
        );
        assert_eq!(d.unknown_dlc_codes.get("zz"), Some(&1));
    }

    /// `{{bug|dlc=r|…}}` says the defect belongs to the editions from Repentance on, and
    /// **204 of the 547 uses** carry a `dlc`. The arm recursed into the positional argument
    /// and never looked at a named one, so every one of those sentences reached the reader
    /// as if it applied to their edition. All 204 spell a code the switch has: 130 of them
    /// were unreadable here only while a code was read one letter at a time.
    #[test]
    fn a_bug_that_belongs_to_one_edition_says_so() {
        let (v, _) = p("{{bug|dlc=r|Only in Repentance}}");
        assert!(
            v.iter().any(|i| matches!(
                i,
                Inline::Edition { only, .. } if only == &vec![Dlc::Repentance, Dlc::RepentancePlus]
            )),
            "got {v:?}"
        );
    }

    /// `{{Book of Virtues synergy|description=…}}` carries its text in a **named** parameter,
    /// so neither `resolve` nor the positional recursion reaches it: 157 sentences arrived
    /// empty. All 164 uses spell the parameter `description`.
    ///
    /// The item reference is emitted too. The template's whole content is "with Book of
    /// Virtues, this happens", and the app shows a section on its own — a bare description
    /// would leave the reader without the half that says *with what*.
    #[test]
    fn the_book_of_virtues_synergy_keeps_its_description() {
        let (v, d) = p("{{Book of Virtues synergy|description=Spawns {{i|Breakfast}} wisps}}");
        let mut flat = String::new();
        for i in &v {
            if let Inline::Text { text, .. } = i {
                flat.push_str(text);
            }
        }
        assert!(flat.contains("Spawns"), "the description is lost: {v:?}");
        assert!(flat.contains("wisps"));
        // The item the synergy is with, named rather than left implicit.
        assert!(v.iter().any(|i| matches!(
            i,
            Inline::Ref {
                target: Target::Item { id: 584 },
                ..
            }
        )));
        // Links inside the description keep working: it is parsed, not pasted.
        assert!(v.iter().any(|i| matches!(
            i,
            Inline::Ref {
                target: Target::Item { id: 25 },
                ..
            }
        )));
        assert!(d.unknown_templates.is_empty(), "{:?}", d.unknown_templates);
    }

    /// The same arm, the same half of the sentence, lost for the other book: the item is
    /// **The Book of Belial**, with the article, and the arm resolved `Book of Belial`,
    /// which is not an item. All **33 uses** dropped the reference and with it the ": "
    /// that introduces the description — while the Virtues arm right beside it worked,
    /// which is why nobody saw it. The two names are written out per arm precisely so a
    /// third template could not inherit the wrong item; one of the two *was* the wrong item.
    #[test]
    fn the_book_of_belial_synergy_names_the_item() {
        let (v, _) = p("{{Book of Belial synergy|description=The axe glows}}");
        assert!(
            v.iter().any(|i| matches!(
                i,
                Inline::Ref {
                    target: Target::Item { id: 34 },
                    ..
                }
            )),
            "the item the synergy is with is missing: {v:?}"
        );
    }

    /// `{{achievement text | I RULE!, Backasswards, Ultra Hard}}` is the odd one out: it
    /// carries a **comma-separated list**, so it cannot go through `resolve`, which answers
    /// with one `Resolution`. It sits on boss pages, naming the achievements that boss
    /// unlocks — 82 of the 143 uses name exactly one, but the longest names seventeen.
    #[test]
    fn achievement_text_lists_every_achievement_it_names() {
        let (v, d) = p("{{achievement text | Epic Fetus, Cain}}");
        let refs: Vec<&Inline> = v
            .iter()
            .filter(|i| {
                matches!(
                    i,
                    Inline::Ref {
                        target: Target::Achievement { .. },
                        ..
                    }
                )
            })
            .collect();
        assert_eq!(refs.len(), 2, "both names must resolve, got {v:?}");
        assert!(d.unknown_templates.is_empty(), "{:?}", d.unknown_templates);

        // A name we cannot resolve keeps its words rather than disappearing.
        let (v, _) = p("{{achievement text | Epic Fetus, Not An Achievement}}");
        let mut flat = String::new();
        for i in &v {
            if let Inline::Text { text, .. } = i {
                flat.push_str(text);
            }
        }
        assert!(
            flat.contains("Not An Achievement"),
            "an unresolved name must stay readable, got {v:?}"
        );
    }

    /// `{{transformation contribution|Beelzebub}}` is a whole sentence on the page ("counts
    /// toward Beelzebub"), but the only part of it we can resolve is the transformation —
    /// which `{{tf|…}}` already resolves. 186 occurrences, every one of them an item's
    /// Effects section saying which transformation it feeds.
    #[test]
    fn transformation_contribution_resolves_like_tf() {
        let (v, d) = p("{{transformation contribution|Beelzebub}}");
        assert!(v.iter().any(|i| matches!(
            i,
            Inline::Ref {
                target: Target::Transformation { .. },
                ..
            }
        )));
        assert!(d.unknown_templates.is_empty(), "{:?}", d.unknown_templates);
    }

    /// A transformation page states what counts toward it as two tables, each one template
    /// whose argument is a comma-separated list of names — the same shape as
    /// `achievement text`, and the same reason it cannot go through `resolve`: that answers
    /// with one target, and this needs one per name.
    #[test]
    fn the_two_tables_push_one_reference_per_name() {
        let (v, d) = p("{{collectible table | Breakfast, Book of Virtues }} {{trinket table | Swallowed Penny }}");
        let targets: Vec<&Target> = v
            .iter()
            .filter_map(|i| match i {
                Inline::Ref { target, .. } => Some(target),
                _ => None,
            })
            .collect();
        assert_eq!(targets.len(), 3, "{v:?}");
        assert!(matches!(targets[0], Target::Item { id: 25 }));
        assert!(matches!(targets[1], Target::Item { id: 584 }));
        assert!(matches!(targets[2], Target::Trinket { id: 1 }));
        assert!(d.unknown_templates.is_empty(), "{:?}", d.unknown_templates);
    }

    /// Censused on the sixteen transformation pages, 2026-09-13: the item lists come in two
    /// shapes, and `rows` is the **more common** one — 25 occurrences against 14 for
    /// `table`, because a page that splits its list by edition uses one `rows` per edition
    /// under a shared header.
    #[test]
    fn the_rows_form_of_the_two_tables_resolves_the_same_way() {
        let (v, d) = p("{{collectible rows | Breakfast, Book of Virtues }} {{trinket rows | Swallowed Penny }}");
        let refs = v.iter().filter(|i| matches!(i, Inline::Ref { .. })).count();
        assert_eq!(refs, 3, "{v:?}");
        assert!(d.unknown_templates.is_empty(), "{:?}", d.unknown_templates);
    }

    /// `{{collectible rows | dlc = r | … }}` is Conjoined's real markup: those items count
    /// toward the transformation **only in that edition**, which is what `Inline::Edition`
    /// already says everywhere else. Flattening it away would claim they always count.
    #[test]
    fn a_rows_list_qualified_by_edition_stays_qualified() {
        let (v, _) = p("{{collectible rows | dlc = r | Breakfast }}");
        assert!(
            v.iter().any(|i| matches!(
                i,
                Inline::Edition { only, inline }
                    if only == &vec![Dlc::Repentance, Dlc::RepentancePlus]
                        && inline.iter().any(|n| matches!(n, Inline::Ref { .. }))
            )),
            "{v:?}"
        );
    }

    /// The header half of the same markup draws a table head and names nothing. It has to be
    /// layout, not unknown: an unknown template recurses into its argument, and these pages
    /// carry twelve of them.
    #[test]
    fn the_table_headers_are_layout_and_not_unknown() {
        let (v, d) =
            p("{{Collectible table/header}}{{trinket table/header}}{{header transformations}}");
        assert!(v.is_empty(), "{v:?}");
        assert!(d.unknown_templates.is_empty(), "{:?}", d.unknown_templates);
    }

    /// A name the resolver does not know stays on the page as text — the reader still sees
    /// what the wiki said — and the miss is counted rather than swallowed.
    #[test]
    fn an_unknown_name_in_a_table_is_text_and_is_counted() {
        let (v, d) = p("{{collectible table | Nope }}");
        assert!(v.iter().all(|i| !matches!(i, Inline::Ref { .. })), "{v:?}");
        assert_eq!(d.unresolved.get("i"), Some(&1));
    }

    /// `{{ip|Boss}}` names an item pool — "Boss" 81 times, then the rooms and the chests.
    /// The game keys its pools by name in `itempools.xml` and gives them no id, so they are
    /// concepts for the same reason machines are.
    #[test]
    fn the_item_pool_template_becomes_a_concept() {
        let (v, d) = p("Found in the {{ip|Boss}} pool");
        assert!(v
            .iter()
            .any(|i| matches!(i, Inline::Concept { page, .. } if page == "Boss")));
        assert!(d.unknown_templates.is_empty(), "{:?}", d.unknown_templates);
        assert!(d.unresolved.is_empty());
    }

    /// `{{dlc|r}}` is a marker: it opens an edition scope that runs to the end of the value
    /// or to `{{dlc-}}`. But the template also has a **three-argument** form that carries its
    /// own text, and until 2026-09-13 the parser opened the scope and dropped the text on the
    /// floor — 313 times across the snapshot, silently. The item quotes are where it showed:
    /// `Boomerang tears {{dlc|r|+ DMG up + luck down}}` arrived as "Boomerang tears".
    #[test]
    fn the_three_argument_dlc_form_keeps_its_own_text() {
        let (v, _) = p("Boomerang tears{{dlc|r|+ DMG up}}");
        assert_eq!(
            v,
            vec![
                text("Boomerang tears", Style::Plain),
                Inline::Edition {
                    only: vec![Dlc::Repentance, Dlc::RepentancePlus],
                    inline: vec![text("+ DMG up", Style::Plain)]
                }
            ]
        );

        // The span closes itself: text after it is not swallowed into the edition.
        let (v, _) = p("{{dlc|r|only in Repentance}} and after");
        assert_eq!(
            v,
            vec![
                Inline::Edition {
                    only: vec![Dlc::Repentance, Dlc::RepentancePlus],
                    inline: vec![text("only in Repentance", Style::Plain)]
                },
                text(" and after", Style::Plain)
            ]
        );

        // The two-argument form still opens a scope that runs on: that is the common use,
        // and this half of the test is what stops the fix from breaking it.
        let (v, _) = p("{{dlc|r}}everything after this");
        assert_eq!(
            v,
            vec![Inline::Edition {
                only: vec![Dlc::Repentance, Dlc::RepentancePlus],
                inline: vec![text("everything after this", Style::Plain)]
            }]
        );
    }

    /// A marker written **inside a parenthesis** qualifies the parenthesis, not the rest of
    /// the value: `({{dlc|r+}} including the 4 Repentance+ achievements)` is Dead God's
    /// requirement, and the wiki draws the icon in front of the aside only. Until 2026-09-23
    /// the scope ran to the end of the value, 57 times across the snapshot, and Dead God's
    /// "and collect every item in the game" read as a Repentance+-only clause.
    #[test]
    fn a_marker_inside_a_parenthesis_closes_with_it() {
        let rplus = || vec![Dlc::RepentancePlus];
        let (v, _) = p("all ({{dlc|r+}} including the 4) and every item");
        assert_eq!(
            v,
            vec![
                text("all (", Style::Plain),
                Inline::Edition {
                    only: rplus(),
                    inline: vec![text(" including the 4", Style::Plain)]
                },
                text(") and every item", Style::Plain)
            ]
        );

        // A parenthesis opened *after* the marker is part of its span, and closing it does
        // not close the scope.
        let (v, _) = p("a ({{dlc|r+}} b (c) d) e");
        assert_eq!(
            v,
            vec![
                text("a (", Style::Plain),
                Inline::Edition {
                    only: rplus(),
                    inline: vec![text(" b (c) d", Style::Plain)]
                },
                text(") e", Style::Plain)
            ]
        );

        // Outside any parenthesis the marker still runs on, through a balanced aside.
        let (v, _) = p("{{dlc|r+}}x (y) z");
        assert_eq!(
            v,
            vec![Inline::Edition {
                only: rplus(),
                inline: vec![text("x (y) z", Style::Plain)]
            }]
        );
    }

    #[test]
    fn dlcalt_arguments_are_parsed_recursively() {
        // The wiki nests other templates both in the positional argument and in the
        // per-edition variants (`{{dlcalt|{{i|Dead Cat}}, …|r=…}}`, seen on Guppy's Collar).
        let (v, _) = p("{{dlcalt|{{i|Breakfast}}|r={{i|Breakfast}} r}}");
        assert_eq!(
            v,
            vec![
                Inline::Ref {
                    target: Target::Item { id: 25 },
                    label: "Breakfast".into()
                },
                Inline::Edition {
                    only: vec![Dlc::Repentance, Dlc::RepentancePlus],
                    inline: vec![
                        text(" ", Style::Plain),
                        Inline::Ref {
                            target: Target::Item { id: 25 },
                            label: "Breakfast".into()
                        },
                        text(" r", Style::Plain),
                    ]
                }
            ]
        );
    }

    #[test]
    fn html_and_entities() {
        let (v, _) = p("2&times;2 <u>Boss</u><br>next<ref>cite</ref> <span class=\"x\">in</span> done<!-- c -->");
        assert_eq!(v, vec![text("2×2 Boss next in done", Style::Plain)]);
        let (v, _) = p("[[File:Boss Hush.png|x]][[Category:Modified]] keep");
        assert_eq!(v, vec![text(" keep", Style::Plain)]);
    }

    #[test]
    fn malformed_input_degrades_to_text() {
        let (v, _) = p("{{i|open [[link é &bogus; a < 10 and > 5 {{dlc-}} tail");
        assert_eq!(
            v,
            vec![text(
                "{{i|open [[link é &bogus; a < 10 and > 5  tail",
                Style::Plain
            )]
        );
        let (v, _) = p("x<ref>no close");
        assert_eq!(v, vec![text("xno close", Style::Plain)]);
        // A code nobody can read used to degrade to an `Edition` that named no edition,
        // which is not a degradation but a claim — the span says it is valid nowhere, and
        // the app draws no badge for it, so the reader is told nothing and cannot tell that
        // from text with no edition at all. Since 2026-09-15 the words come through plain
        // and the code is counted: an unreadable code restricts nothing, which is the same
        // shape as the wiki's own "no restriction" and a different reason for it.
        let (v, d) = p("{{dlc+|zz}}unknown code");
        assert_eq!(v, vec![text("unknown code", Style::Plain)]);
        assert_eq!(d.unknown_dlc_codes.get("zz"), Some(&1));
    }

    #[test]
    fn plain_reads_a_run_the_way_a_reader_would() {
        assert_eq!(
            plain(&[
                text("Defeat ", Style::Plain),
                Inline::Ref {
                    target: Target::Achievement { id: 1 },
                    label: "Mom's Heart".into()
                },
                text(" as ", Style::Plain),
                Inline::Concept {
                    page: "The Lost".into(),
                    label: "The Lost".into()
                },
            ]),
            "Defeat Mom's Heart as The Lost",
            "a reference and a concept are their label, never their target"
        );
    }

    #[test]
    fn plain_keeps_the_words_inside_an_edition_wrapper() {
        assert_eq!(
            plain(&[Inline::Edition {
                only: vec![Dlc::Repentance, Dlc::RepentancePlus],
                inline: vec![text("use the Red Key", Style::Plain)],
            }]),
            "use the Red Key",
            "the wrapper says which edition, not something the sentence needs to lose"
        );
    }

    #[test]
    fn plain_of_nothing_is_nothing() {
        assert_eq!(plain(&[]), "");
    }
}
