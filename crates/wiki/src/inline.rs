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

    /// Closes the innermost `Edition`; does nothing without any open frame. An empty
    /// edition is not emitted.
    fn close(&mut self) {
        self.flush();
        if self.frames.len() > 1 {
            if let Some((only, inline)) = self.frames.pop() {
                if !inline.is_empty() {
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

/// `a+` is a whole code: it's tried as such first, then split on `,` and whitespace. An
/// unknown code disappears, and an `Edition` with an empty `only` means "no known edition".
fn dlc_codes(s: &str) -> Vec<Dlc> {
    if let Some(d) = Dlc::from_code(s) {
        return vec![d];
    }
    s.split(|c: char| c == ',' || c.is_whitespace())
        .filter_map(Dlc::from_code)
        .collect()
}

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
        _ => return None, // allowed: HTML entity, an open-ended set
    })
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
            if let Some((rep, end)) = rest
                .get(1..)
                .and_then(|s| s.find(';'))
                .filter(|e| *e <= 8)
                .and_then(|end| Some((entity(rest.get(1..1 + end)?)?, end)))
            {
                out.buf.push_str(rep);
                i += end + 2;
                continue;
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
                out.open(dlc_codes(&arg));
                recurse_into_arg(content, r, d, out, depth);
                out.close();
            }
            None => out.open(dlc_codes(&arg)),
        },
        "dlc-" => out.close(),
        "dlcalt" => {
            // Both the positional argument and every per-edition variant are content
            // (the wiki nests other templates in them, `{{p|Soul of Lazarus}}` included):
            // same recursion as unknown templates, not raw text.
            recurse_into_arg(&arg, r, d, out, depth);
            for (code, text) in &t.named {
                out.open(dlc_codes(code));
                out.buf.push(' ');
                recurse_into_arg(text, r, d, out, depth);
                out.close();
            }
        }
        name if CONTENT_WRAPPERS.contains(&name) => recurse_into_arg(&arg, r, d, out, depth),
        name => match r.resolve(name, &arg) {
            Resolution::Target(target) => {
                let label = label_of(t, &arg);
                out.push(Inline::Ref { target, label });
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
        let (v, d) = p("{{cit|p|r}}x {{m|Donation Machine}} y");
        assert_eq!(v, vec![text("x Donation Machine y", Style::Plain)]);
        assert_eq!(d.unknown_templates.get("m"), Some(&1));
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
        let (v, _) = p("{{dlc+|zz}}unknown code");
        assert_eq!(
            v,
            vec![Inline::Edition {
                only: vec![],
                inline: vec![text("unknown code", Style::Plain)]
            }]
        );
    }
}
