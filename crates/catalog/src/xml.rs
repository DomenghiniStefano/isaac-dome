//! The only layer over quick-xml, and the reading every per-source module shares.
//!
//! The per-source modules work on `Element` and never touch the library: if it ever
//! changes, only this file needs to change. What they had each written for themselves —
//! the unreadable file, the row without a usable id, the root folder, the direct
//! children — lives here once, so nine parsers say the same thing the same way.

use quick_xml::events::Event;
use quick_xml::{Reader, XmlVersion};

use crate::diagnostics::{Diagnostic, SkipReason, Source};

/// An element of the document, in opening order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Element {
    pub name: String,
    pub attrs: Vec<(String, String)>,
    /// The element's direct text, concatenated and trimmed at the edges.
    pub text: String,
    /// The last comment seen before the opening tag, if no element has already claimed it.
    pub comment_before: Option<String>,
    /// 0 for the root.
    pub depth: usize,
}

impl Element {
    pub fn attr(&self, name: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }
}

/// A document quick-xml doesn't accept. No details: only `Diagnostic::SourceUnreadable`
/// crosses the boundary, and a message would carry fragments of the file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct XmlError;

/// All the elements of the document, in opening order, with text and comment.
///
/// One loop over quick-xml's events, with its state kept in three locals: a pull parser is
/// a cursor, and this is the one place in the crate that is written as one on purpose.
pub fn elements(bytes: &[u8]) -> Result<Vec<Element>, XmlError> {
    let mut reader = Reader::from_reader(bytes);
    // No trim_text: quick-xml 0.42 splits entity references (`&amp;`) into `GeneralRef`
    // events separate from the surrounding text fragments, and a per-fragment trim
    // would eat the real space attached to the edge of the reference (e.g. "a &amp; b"
    // -> "a" + "b" instead of "a & b"). We accumulate the raw text and trim it only
    // once, at the edges, when the element closes.

    let mut out: Vec<Element> = Vec::new();
    let mut open: Vec<usize> = Vec::new(); // indices in `out` of the open elements
    let mut pending_comment: Option<String> = None;
    let mut buf = Vec::new();

    loop {
        let event = reader.read_event_into(&mut buf).map_err(|_| XmlError)?;
        match event {
            Event::Start(e) => {
                push_element(&mut out, &mut open, &mut pending_comment, &e, false)?;
            }
            Event::Empty(e) => {
                push_element(&mut out, &mut open, &mut pending_comment, &e, true)?;
            }
            Event::End(_) => {
                if let Some(i) = open.pop() {
                    out[i].text = out[i].text.trim().to_string();
                }
            }
            Event::Text(t) => {
                if let Some(&i) = open.last() {
                    out[i].text.push_str(&t);
                }
            }
            Event::GeneralRef(r) => {
                if let Some(&i) = open.last() {
                    push_reference(&mut out[i].text, &r)?;
                }
            }
            Event::Comment(c) => {
                let s = quick_xml::escape::unescape(&c).map_err(|_| XmlError)?;
                pending_comment = Some(s.trim().to_string());
            }
            Event::Eof => break,
            // `Event` is an external quick-xml enum: the other events (CData,
            // declarations, processing instructions, doctype) are of no use to us.
            Event::CData(_) | Event::Decl(_) | Event::PI(_) | Event::DocType(_) => {}
        }
        buf.clear();
    }
    if !open.is_empty() {
        return Err(XmlError); // elements never closed: truncated document
    }
    Ok(out)
}

/// Appends what an entity reference stands for: a character reference (`&#39;`) or one of
/// the five predefined entities. Any other entity is an error.
fn push_reference(text: &mut String, r: &quick_xml::events::BytesRef<'_>) -> Result<(), XmlError> {
    match r.resolve_char_ref().map_err(|_| XmlError)? {
        Some(ch) => text.push(ch),
        None => {
            text.push_str(quick_xml::escape::resolve_predefined_entity(r).ok_or(XmlError)?);
        }
    }
    Ok(())
}

/// Adds an open element (`Start` or `Empty`) to `out`, claiming any pending comment
/// and pushing the nesting stack unless it's an empty element.
fn push_element(
    out: &mut Vec<Element>,
    open: &mut Vec<usize>,
    pending_comment: &mut Option<String>,
    e: &quick_xml::events::BytesStart<'_>,
    is_empty: bool,
) -> Result<(), XmlError> {
    let name = e.local_name().as_ref().to_owned();
    let attrs = e
        .attributes()
        .map(|a| {
            let a = a.map_err(|_| XmlError)?;
            let value = a
                .normalized_value(XmlVersion::Implicit1_0)
                .map_err(|_| XmlError)?
                .into_owned();
            Ok((a.key.local_name().as_ref().to_owned(), value))
        })
        .collect::<Result<Vec<_>, XmlError>>()?;
    out.push(Element {
        name,
        attrs,
        text: String::new(),
        comment_before: pending_comment.take(),
        depth: open.len(),
    });
    if !is_empty {
        open.push(out.len() - 1);
    }
    Ok(())
}

/// The elements of a source's document, or nothing and one `SourceUnreadable` for it: the
/// first step of every parser, and the only way a whole file is reported unreadable.
pub(crate) fn read(bytes: &[u8], source: Source, d: &mut Vec<Diagnostic>) -> Option<Vec<Element>> {
    match elements(bytes) {
        Ok(els) => Some(els),
        Err(XmlError) => {
            d.push(Diagnostic::SourceUnreadable { source });
            None
        }
    }
}

/// Records that one element of `source` was left out, and yields nothing, so a row reader
/// can end with `return skip(…)` or chain it after an `Option`.
pub(crate) fn skip<T>(
    source: Source,
    id: Option<u32>,
    reason: SkipReason,
    d: &mut Vec<Diagnostic>,
) -> Option<T> {
    d.push(Diagnostic::ElementSkipped { source, id, reason });
    None
}

/// The numeric id a row declares in `attr`. A row without one, or with one that is not a
/// number, is skipped — with no id in the diagnostic, since it has none worth repeating.
pub(crate) fn required_id(
    e: &Element,
    attr: &str,
    source: Source,
    d: &mut Vec<Diagnostic>,
) -> Option<u32> {
    let Some(raw) = e.attr(attr) else {
        return skip(source, None, SkipReason::MissingId, d);
    };
    raw.parse::<u32>()
        .ok()
        .or_else(|| skip(source, None, SkipReason::MalformedId, d))
}

/// An attribute row `id` cannot do without; its absence skips the row for `reason`.
pub(crate) fn required_attr<'e>(
    e: &'e Element,
    attr: &str,
    id: u32,
    reason: SkipReason,
    source: Source,
    d: &mut Vec<Diagnostic>,
) -> Option<&'e str> {
    e.attr(attr).or_else(|| skip(source, Some(id), reason, d))
}

/// The folder the root element `root` declares in `attr`, normalized; `default` when it
/// declares none, or one that normalizes to nothing (`""`, `resources/`).
pub(crate) fn root_attr(els: &[Element], root: &str, attr: &str, default: &str) -> String {
    els.iter()
        .find(|e| e.name == root)
        .and_then(|e| e.attr(attr))
        .map(normalize_root)
        .filter(|r| !r.is_empty())
        .unwrap_or_else(|| default.to_string())
}

/// `resources/gfx/items/` or `gfx/items/` -> `gfx/items`: no archive root or trailing slash.
pub(crate) fn normalize_root(root: &str) -> String {
    let r = root.replace('\\', "/");
    let r = r.strip_prefix("resources/").unwrap_or(&r);
    r.trim_matches('/').to_string()
}

/// Element `root` and everything inside it: the elements that follow it, up to the next one
/// at a depth `<=` its own. Its indices start again at 0, with `root` first.
pub(crate) fn subtree(els: &[Element], root: usize) -> &[Element] {
    let depth = els[root].depth;
    let inside = els[root + 1..]
        .iter()
        .take_while(|e| e.depth > depth)
        .count();
    &els[root..=root + inside]
}

/// The direct children of element `parent` with that name: the elements of its subtree one
/// level below it.
pub(crate) fn children_named<'a>(
    els: &'a [Element],
    parent: usize,
    name: &str,
) -> Vec<&'a Element> {
    let depth = els[parent].depth;
    subtree(els, parent)[1..]
        .iter()
        .filter(|e| e.depth == depth + 1 && e.name == name)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_root_strips_the_archive_prefix_backslashes_and_trailing_slashes() {
        assert_eq!(normalize_root("resources/gfx/items/"), "gfx/items");
        assert_eq!(normalize_root("gfx/items"), "gfx/items");
        assert_eq!(normalize_root("gfx\\items\\"), "gfx/items");
        assert_eq!(normalize_root(""), "");
    }

    #[test]
    fn an_unreadable_document_is_one_diagnostic_for_its_source() {
        let mut d = Vec::new();
        assert!(read(b"<r><x></r>", Source::Players, &mut d).is_none());
        assert_eq!(
            d,
            vec![Diagnostic::SourceUnreadable {
                source: Source::Players
            }]
        );
        assert_eq!(
            read(b"<r/>", Source::Players, &mut d).map(|e| e.len()),
            Some(1)
        );
        assert_eq!(d.len(), 1, "a readable one adds nothing");
    }

    #[test]
    fn a_required_id_tells_missing_from_malformed_and_a_required_attr_names_the_row() {
        let els = elements(b"<r><a id=\"7\"/><b/><c id=\"-1\"/></r>").unwrap();
        let mut d = Vec::new();
        assert_eq!(required_id(&els[1], "id", Source::Items, &mut d), Some(7));
        assert_eq!(required_id(&els[2], "id", Source::Items, &mut d), None);
        assert_eq!(required_id(&els[3], "id", Source::Items, &mut d), None);
        assert_eq!(
            required_attr(
                &els[1],
                "gfx",
                7,
                SkipReason::MissingSprite,
                Source::Items,
                &mut d
            ),
            None
        );
        let skipped = |id, reason| Diagnostic::ElementSkipped {
            source: Source::Items,
            id,
            reason,
        };
        assert_eq!(
            d,
            vec![
                skipped(None, SkipReason::MissingId),
                skipped(None, SkipReason::MalformedId),
                skipped(Some(7), SkipReason::MissingSprite),
            ]
        );
    }

    #[test]
    fn a_subtree_ends_where_its_root_closes_and_children_are_one_level_down() {
        let els = elements(b"<r><a><b/><c><b/></c></a><b/></r>").unwrap();
        let names = |s: &[Element]| s.iter().map(|e| e.name.clone()).collect::<Vec<_>>();
        assert_eq!(names(subtree(&els, 1)), vec!["a", "b", "c", "b"]);
        assert_eq!(
            names(subtree(&els, 2)),
            vec!["b"],
            "an empty element is its own subtree"
        );
        assert_eq!(names(subtree(&els, 0)).len(), els.len());
        assert_eq!(children_named(&els, 1, "b").len(), 1, "the nested b is c's");
        assert_eq!(children_named(&els, 0, "b").len(), 1, "a's b is not r's");
    }

    #[test]
    fn a_root_folder_falls_back_when_absent_or_empty_once_normalized() {
        let els = elements(b"<items gfxroot=\"resources/gfx/x/\"/>").unwrap();
        assert_eq!(root_attr(&els, "items", "gfxroot", "d"), "gfx/x");
        assert_eq!(root_attr(&els, "items", "other", "d"), "d");
        assert_eq!(root_attr(&els, "players", "gfxroot", "d"), "d");
        let empty = elements(b"<items gfxroot=\"resources/\"/>").unwrap();
        assert_eq!(root_attr(&empty, "items", "gfxroot", "d"), "d");
    }

    #[test]
    fn attributes_separated_by_spaces_and_tabs_are_the_same() {
        let a = elements(b"<r><passive id=\"1\" gfx=\"a.png\" /></r>").unwrap();
        let b = elements(b"<r><passive\t\tid=\"1\"\tgfx=\"a.png\" /></r>").unwrap();
        assert_eq!(a[1].attr("id"), Some("1"));
        assert_eq!(b[1].attr("id"), Some("1"));
        assert_eq!(b[1].attr("gfx"), Some("a.png"));
        assert_eq!(b[1].name, "passive");
    }

    #[test]
    fn the_comment_before_an_element_travels_with_it() {
        let els = elements(
            b"<achievements>\n<!-- have 7 hearts -->\n<achievement id=\"1\" />\n<achievement id=\"2\" /></achievements>",
        )
        .unwrap();
        assert_eq!(els[1].comment_before.as_deref(), Some("have 7 hearts"));
        assert_eq!(
            els[2].comment_before, None,
            "the comment applies to a single element only"
        );
    }

    #[test]
    fn two_comments_before_an_element_keep_the_last() {
        let els = elements(b"<r><!-- a --><!-- b --><x /></r>").unwrap();
        assert_eq!(els[1].comment_before.as_deref(), Some("b"));
    }

    #[test]
    fn text_content_and_depth_are_captured() {
        let els = elements(b"<t><category name=\"Items\"><key name=\"K\"><string>Hello</string><string>Ciao</string></key></category></t>").unwrap();
        let strings: Vec<&Element> = els.iter().filter(|e| e.name == "string").collect();
        assert_eq!(strings.len(), 2);
        assert_eq!(strings[0].text, "Hello");
        assert_eq!(strings[1].text, "Ciao");
        assert_eq!(strings[0].depth, 3);
        assert_eq!(els[1].depth, 1, "category");
    }

    #[test]
    fn entities_are_unescaped() {
        let els = elements(b"<r><x name=\"Mom&apos;s Heart\">a &amp; b</x></r>").unwrap();
        assert_eq!(els[1].attr("name"), Some("Mom's Heart"));
        assert_eq!(els[1].text, "a & b");
    }

    #[test]
    fn junk_is_an_error_not_a_panic() {
        assert!(elements(b"<r><x></r>").is_err());
        assert!(elements(&[0xff, 0xfe, 0x00]).is_err());
    }

    #[test]
    fn the_five_predefined_entities_are_resolved_in_text() {
        let els = elements(b"<r>&lt;&gt;&amp;&apos;&quot;</r>").unwrap();
        assert_eq!(els[0].text, "<>&'\"");
    }

    #[test]
    fn numeric_character_references_are_resolved_in_text() {
        let els = elements(b"<r>&#39;&#x27;</r>").unwrap();
        assert_eq!(
            els[0].text, "''",
            "decimal and hexadecimal give the same apostrophe"
        );
    }

    #[test]
    fn an_unknown_entity_in_text_is_an_error() {
        assert!(elements(b"<r>&foo;</r>").is_err());
    }

    #[test]
    fn whitespace_between_child_elements_does_not_leak_into_the_parent_text() {
        let els = elements(b"<a>\n  <b>x</b>\n</a>").unwrap();
        assert_eq!(
            els[0].text, "",
            "whitespace between children is not the parent's text, and the edges stay clean"
        );
        assert_eq!(els[1].text, "x");
    }
}
