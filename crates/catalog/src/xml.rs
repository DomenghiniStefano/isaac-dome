//! The only layer over quick-xml. The per-source modules work on `Element` and never
//! touch the library: if it ever changes, only this file needs to change.

use quick_xml::events::Event;
use quick_xml::{Reader, XmlVersion};

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
                    if let Some(ch) = r.resolve_char_ref().map_err(|_| XmlError)? {
                        out[i].text.push(ch);
                    } else {
                        let resolved = quick_xml::escape::resolve_predefined_entity(r.as_ref())
                            .ok_or(XmlError)?;
                        out[i].text.push_str(resolved);
                    }
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
    let mut attrs = Vec::new();
    for a in e.attributes() {
        let a = a.map_err(|_| XmlError)?;
        let key = a.key.local_name().as_ref().to_owned();
        let value = a
            .normalized_value(XmlVersion::Implicit1_0)
            .map_err(|_| XmlError)?
            .into_owned();
        attrs.push((key, value));
    }
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

#[cfg(test)]
mod tests {
    use super::*;

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
