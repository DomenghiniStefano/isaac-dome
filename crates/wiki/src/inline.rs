//! From a line of wikitext to `Vec<Inline>`. Single-pass scan with a stack of open
//! `Edition`s; text accumulates and is emitted on a style or node change.
//!
//! The input is external data: no path may panic. Every malformed construct (an
//! unclosed template, an open link, a tag with no `>`) degrades to text.

use crate::resolver::{Resolution, Resolver};
use crate::template::{parse_template_at, Template};
use crate::{Diagnostics, Dlc, Inline, Style};

/// The output builder: a stack of frames, where the bottom is the top level and every
/// frame above it is an open `Edition` with its own codes.
struct Out {
    frames: Vec<(Vec<Dlc>, Vec<Inline>)>,
    buf: String,
    bold: bool,
    italic: bool,
}

impl Out {
    fn new() -> Out {
        Out {
            frames: vec![(Vec::new(), Vec::new())],
            buf: String::new(),
            bold: false,
            italic: false,
        }
    }

    /// Bold wins if both are open.
    fn style(&self) -> Style {
        if self.bold {
            Style::Bold
        } else if self.italic {
            Style::Italic
        } else {
            Style::Plain
        }
    }

    /// Emits the accumulated text, merging it with the previous `Text` if it has the same style.
    fn flush(&mut self) {
        if self.buf.is_empty() {
            return;
        }
        let style = self.style();
        let text = std::mem::take(&mut self.buf);
        let Some((_, top)) = self.frames.last_mut() else {
            return;
        };
        match top.last_mut() {
            Some(Inline::Text { text: t, style: s }) if *s == style => t.push_str(&text),
            _ => top.push(Inline::Text { text, style }),
        }
    }

    /// Merges with the last node if both are `Text` with the same style: recursion into
    /// unknown templates (`recurse_into_arg`) can return an opening `Text`, and without
    /// this it would split away from the text just accumulated before the template.
    fn push(&mut self, node: Inline) {
        self.flush();
        let Some((_, top)) = self.frames.last_mut() else {
            return;
        };
        match (&node, top.last_mut()) {
            (Inline::Text { text, style }, Some(Inline::Text { text: t, style: s }))
                if style == s =>
            {
                t.push_str(text);
            }
            _ => top.push(node),
        }
    }

    fn open(&mut self, only: Vec<Dlc>) {
        self.flush();
        self.frames.push((only, Vec::new()));
    }

    /// Closes the innermost `Edition`; does nothing without any open frame. An edition with
    /// no content is not emitted — and neither is one that names no **edition**: an empty
    /// `only` means the codes could not be read, and a node declaring its text valid in no
    /// edition at all is worse than the text on its own. Its words go back to the parent,
    /// and `dlc_codes` counts the code that produced it.
    ///
    /// The frame is opened either way: `{{dlc+|…}}` is closed by a later `{{dlc-}}`, so
    /// skipping the open would leave the close popping somebody else's frame.
    fn close(&mut self) {
        self.flush();
        if self.frames.len() > 1 {
            if let Some((only, inline)) = self.frames.pop() {
                if only.is_empty() {
                    for node in inline {
                        self.push(node);
                    }
                } else if !inline.is_empty() {
                    self.push(Inline::Edition { only, inline });
                }
            }
        }
    }

    fn finish(mut self) -> Vec<Inline> {
        while self.frames.len() > 1 {
            self.close();
        }
        self.flush();
        self.frames.pop().map(|f| f.1).unwrap_or_default()
    }
}

/// `a+` is a whole code: it's tried as such first, then split on `,` and whitespace. What
/// is left unread produces an empty list, which `Out::close` unwraps instead of emitting —
/// a span valid in no edition is not a span.
fn dlc_codes(s: &str, d: &mut Diagnostics) -> Vec<Dlc> {
    let codes = match Dlc::from_code(s) {
        Some(one) => vec![one],
        None => s
            .split(|c: char| c == ',' || c.is_whitespace())
            .filter_map(Dlc::from_code)
            .collect(),
    };
    // What this cannot read is counted, never dropped in silence: **1734 of the 4168**
    // `{{dlc|…}}` uses in the wikitext concatenate their codes, and each one used to open a
    // span valid in no edition at all. The build counts **1832**, not 1734, and the gap is
    // not a disagreement: a page that belongs to two forms (Tainted Lazarus and its dead
    // half) is read once per entry.
    //
    // **`Dlc::parse_codes` is not the answer here, and reading them as a set would ship
    // 1734 wrong labels.** It splits the *infobox* parameter, where the codes are the
    // editions an entry exists in. In this family every code appears both bare and with a
    // leading `n` — `r` 1687 / `nr` 1155, `r+` 446 / `nr+` 207, `a+` 165 / `na+` 149,
    // `a` 134 / `na` 69 — and a bare `n` appears **zero** times in 4168 uses, which is not
    // what a set of editions looks like. Abyss settles it: the item exists only in
    // Repentance (`dlc = r` in its own infobox) and carries a line marked `{{dlc|nr+}}`,
    // which as a set would be valid in an edition where the item is not. So `n` modifies
    // the code beside it; what it means is unmeasured, and one query against the wiki's
    // own `Template:Dlc` answers it (B52).
    if codes.is_empty() && !s.trim().is_empty() {
        d.unknown_dlc_code(s.trim());
    }
    codes
}

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
    while let Some(rest) = src.get(i..).filter(|s| !s.is_empty()) {
        // Comments first: the template parser doesn't know about them.
        if let Some(end) = rest.strip_prefix("<!--").and_then(|t| t.find("-->")) {
            i += "<!--".len() + end + "-->".len();
            continue;
        }
        if rest.starts_with("'''") {
            out.flush();
            out.bold = !out.bold;
            i += 3;
            continue;
        }
        if rest.starts_with("''") {
            out.flush();
            out.italic = !out.italic;
            i += 2;
            continue;
        }
        if rest.starts_with("{{") {
            if let Some((t, end)) = parse_template_at(src, i) {
                i = end;
                template(&t, r, d, &mut out, depth);
                continue;
            }
            out.buf.push_str("{{");
            i += 2;
            continue;
        }
        if rest.starts_with("[[") {
            if let Some((inner, end)) = rest
                .find("]]")
                .and_then(|end| Some((rest.get(2..end)?, end)))
            {
                i += end + 2;
                link(inner, r, &mut out);
                continue;
            }
            out.buf.push_str("[[");
            i += 2;
            continue;
        }
        if rest.starts_with('<') {
            if let Some((tag, end)) = rest
                .find('>')
                .and_then(|end| Some((rest.get(1..end)?, end)))
            {
                let name = tag_name(tag);
                if !name.is_empty() {
                    i += end + 1;
                    if name == "br" {
                        out.buf.push(' ');
                    } else if name == "ref" && !tag.ends_with('/') && !tag.starts_with('/') {
                        // `<ref>…</ref>` disappears whole; without a closing tag only the tag disappears.
                        if let Some(close) = src.get(i..).and_then(|s| s.find("</ref>")) {
                            i += close + "</ref>".len();
                        }
                    }
                    // every other tag: gone, the content stays
                    continue;
                }
            }
        }
        if rest.starts_with('&') {
            if let Some((name, end)) = rest
                .get(1..)
                .and_then(|s| s.find(';'))
                .filter(|e| *e <= 8)
                .and_then(|end| Some((rest.get(1..1 + end)?, end)))
                .filter(|(name, _)| looks_like_entity_name(name))
            {
                match entity(name) {
                    Some(rep) => {
                        out.buf.push_str(rep);
                        i += end + 2;
                        continue;
                    }
                    // Counted, and the text kept: it is the wiki's content, and dropping a
                    // run because we do not know one name in it would destroy more than it
                    // fixes. The counter is what makes the next one visible instead of
                    // shipped — which is how the three below it shipped.
                    None => d.unknown_entity(name),
                }
            }
        }
        let Some(ch) = rest.chars().next() else {
            break;
        };
        out.buf.push(ch);
        i += ch.len_utf8();
    }
    out.finish()
}

/// An already-parsed template: editions, `{{!}}`, or a reference passed to the resolver.
fn template(t: &Template, r: &Resolver, d: &mut Diagnostics, out: &mut Out, depth: u32) {
    let arg = t.args.first().cloned().unwrap_or_default();
    match t.name.as_str() {
        "!" => out.buf.push('|'),
        // Two shapes share this name. `{{dlc|r}}` is a marker: it opens a scope that runs to
        // the end of the value or to `{{dlc-}}`. `{{dlc|r|text}}` carries its own text and
        // closes itself — and until 2026-09-13 that second argument was never read, so 313
        // spans across the snapshot lost their words without a diagnostic.
        "dlc+" | "dlc" => match t.args.get(1) {
            Some(content) => {
                out.open(dlc_codes(&arg, d));
                recurse_into_arg(content, r, d, out, depth);
                out.close();
            }
            None => out.open(dlc_codes(&arg, d)),
        },
        "dlc-" => out.close(),
        "dlcalt" => {
            // Both the positional argument and every per-edition variant are content
            // (the wiki nests other templates in them, `{{p|Soul of Lazarus}}` included):
            // same recursion as unknown templates, not raw text.
            recurse_into_arg(&arg, r, d, out, depth);
            for (code, text) in &t.named {
                out.open(dlc_codes(code, d));
                out.buf.push(' ');
                recurse_into_arg(text, r, d, out, depth);
                out.close();
            }
        }
        // The text is in a *named* parameter, which neither `resolve` nor the positional
        // recursion reaches: 157 of these sentences used to arrive empty. The item is named
        // too, because the template's meaning is "with Book of Virtues, this happens" and a
        // section shown on its own would otherwise lose the half that says with what.
        // A boss's champion variant, always under `== Champion Versions ==`. The number is
        // the variant's index; which colour each index is lives in the wiki's own template
        // and nowhere we can read, and `catalog` has no champion table — so the index is
        // kept verbatim and nothing is invented around it. `dlc=` makes the variant belong
        // to one edition, which is what `Inline::Edition` already says.
        "bc" => {
            let edition = t.named.get("dlc").map(|c| dlc_codes(c, d));
            if let Some(only) = edition.clone() {
                out.open(only);
            }
            let index = arg.trim();
            out.push(Inline::Concept {
                page: "Champion".to_string(),
                label: format!("Champion {index}"),
            });
            if edition.is_some() {
                out.close();
            }
        }
        // Two templates, one shape: the item is in the name and the text is in a `description`
        // parameter, spelled that way in all 197 uses. The label is written out per arm rather
        // than derived from the name, so a third "X synergy" template cannot silently inherit
        // the wrong item.
        name @ ("book of virtues synergy" | "book of belial synergy") => {
            let label = if name == "book of virtues synergy" {
                "Book of Virtues"
            } else {
                // With the article: the item's page is "The Book of Belial", and without it
                // the resolver answers nothing — 33 uses lost the reference, and the ": "
                // that introduces the description with it.
                "The Book of Belial"
            };
            if let Resolution::Target(target) = r.resolve("i", label) {
                out.push(Inline::Ref {
                    target,
                    label: label.to_string(),
                });
                out.buf.push_str(": ");
            }
            if let Some(description) = t.named.get("description") {
                recurse_into_arg(description, r, d, out, depth);
            }
        }
        // The only template whose argument is a *list*: a boss page names the achievements
        // that boss unlocks, comma-separated. `resolve` answers with one `Resolution`, so
        // this cannot go through it — it has to push a node per name.
        "achievement text" => {
            for (n, name) in arg.split(',').enumerate() {
                let name = name.trim();
                if name.is_empty() {
                    continue;
                }
                if n > 0 {
                    out.buf.push_str(", ");
                }
                match r.achievement_by_name(name) {
                    Some(target) => out.push(Inline::Ref {
                        target,
                        label: name.to_string(),
                    }),
                    // Not dropped: a name we cannot resolve is still what the page says.
                    None => out.buf.push_str(name),
                }
            }
        }
        // Two more list templates, and the same reason as `achievement text`: the argument
        // is a comma-separated list of names, and `resolve` answers with one target. They
        // are how a transformation page states what counts toward it, which is the only
        // complete statement of that set — the infobox's `items` misses Guppy's trinket.
        k @ ("collectible table" | "collectible rows" | "trinket table" | "trinket rows") => {
            let kind = if k.starts_with("collectible") {
                "i"
            } else {
                "t"
            };
            // `rows` takes an optional `dlc =`: Conjoined splits its list by edition, one
            // `rows` each under a shared header, and those items count only in that
            // edition — which is what `Inline::Edition` says everywhere else.
            let edition = t.named.get("dlc").map(|c| dlc_codes(c, d));
            if let Some(only) = edition.clone() {
                out.open(only);
            }
            for (n, item) in arg.split(',').enumerate() {
                let item = item.trim();
                if item.is_empty() {
                    continue;
                }
                if n > 0 {
                    out.buf.push_str(", ");
                }
                match r.resolve(kind, item) {
                    Resolution::Target(target) => out.push(Inline::Ref {
                        target,
                        label: item.to_string(),
                    }),
                    // Not dropped: a name we cannot resolve is still what the page says,
                    // and the miss is counted where every other failed lookup is counted.
                    Resolution::Concept
                    | Resolution::Unresolved
                    | Resolution::Ignore
                    | Resolution::Unknown => {
                        d.unresolved(kind);
                        out.buf.push_str(item);
                    }
                }
            }
            if edition.is_some() {
                out.close();
            }
        }
        // 204 of the 547 `{{bug|…}}` carry a `dlc`, and until 2026-09-15 this arm recursed
        // into the positional argument and read no named one: a defect that exists in one
        // edition was shown to every reader as theirs. The 130 whose code this parser
        // cannot read open an edition that names none, which `Out::close` unwraps — the
        // sentence keeps every word and the code is counted.
        name if CONTENT_WRAPPERS.contains(&name) => {
            out.open(
                t.named
                    .get("dlc")
                    .map_or_else(Vec::new, |c| dlc_codes(c, d)),
            );
            recurse_into_arg(&arg, r, d, out, depth);
            out.close();
        }
        name => match r.resolve(name, &arg) {
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
        },
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
                    only: vec![Dlc::Repentance],
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
                only: vec![Dlc::AfterbirthPlus],
                inline: vec![text(" Isaac also starts with X", Style::Plain)]
            }]
        );
        let (v, _) = p("{{dlcalt|17.75|r=4.5}}");
        assert_eq!(
            v,
            vec![
                text("17.75", Style::Plain),
                Inline::Edition {
                    only: vec![Dlc::Repentance],
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

        // `{{bc|18|dlc=a+}}` says the variant is of one edition: that is `Inline::Edition`,
        // the same node every other per-edition span uses.
        let (v, _) = p("{{bc|18|dlc=a+}}");
        assert!(
            v.iter().any(|i| matches!(
                i,
                Inline::Edition { only, .. } if only == &vec![Dlc::AfterbirthPlus]
            )),
            "got {v:?}"
        );
    }

    /// An edition that names no edition is not an edition. 1734 of the 4168 `{{dlc|…}}`
    /// uses concatenate their codes — `nr` alone 1155 — and this parser reads whole codes
    /// only, so every one of them opened a frame with an empty `only`: **1690 of those
    /// reached `wiki.json`**, where `WikiInline.vue` draws no badge (`v-if="token.only.length"`)
    /// and the text arrives saying nothing about which edition it belongs to. They are
    /// **0** since, and `meta.diagnostics.unknownDlcCodes` carries 1832 of them by code.
    ///
    /// The words are kept and the node is not: a span that declares itself valid in no
    /// edition is worse than the same span unwrapped. What could not be read is counted,
    /// which is what `unknown_dlc_codes` exists for.
    #[test]
    fn an_edition_that_names_no_edition_keeps_its_words_and_is_counted() {
        let (v, d) = p("{{dlc|nr|Only since some edition}}");
        assert!(
            !v.iter().any(|i| matches!(i, Inline::Edition { .. })),
            "an edition valid nowhere: {v:?}"
        );
        assert!(
            v.iter().any(|i| matches!(
                i,
                Inline::Text { text, .. } if text.contains("Only since some edition")
            )),
            "the words went with it: {v:?}"
        );
        assert_eq!(d.unknown_dlc_codes.get("nr"), Some(&1));
    }

    /// `{{bug|dlc=r|…}}` says the defect exists only in Repentance, and **204 of the 547
    /// uses** carry a `dlc`. The arm recursed into the positional argument and never looked
    /// at a named one, so every one of those sentences reached the reader as if it applied
    /// to their edition. 74 of the 204 spell a code this parser can read; the other 130 are
    /// concatenated, and they are counted by the test above rather than guessed at.
    #[test]
    fn a_bug_that_belongs_to_one_edition_says_so() {
        let (v, _) = p("{{bug|dlc=r|Only in Repentance}}");
        assert!(
            v.iter().any(|i| matches!(
                i,
                Inline::Edition { only, .. } if only == &vec![Dlc::Repentance]
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
                    if only == &vec![Dlc::Repentance]
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
                    only: vec![Dlc::Repentance],
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
                    only: vec![Dlc::Repentance],
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
                only: vec![Dlc::Repentance],
                inline: vec![text("everything after this", Style::Plain)]
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
                    only: vec![Dlc::Repentance],
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
        // and the code is counted, the way `Dlc::parse_codes` has always treated its own
        // leftovers.
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
                only: vec![Dlc::Repentance],
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
