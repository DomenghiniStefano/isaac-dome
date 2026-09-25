//! The inline parser's output builder, kept apart from the scan that drives it: the scan
//! decides what a construct is, this decides where its nodes land.

use crate::{Dlc, Inline, Style};

/// The output builder: a stack of frames, where the bottom is the top level and every
/// frame above it is an open `Edition` with its own codes.
pub(super) struct Out {
    frames: Vec<(Vec<Dlc>, Vec<Inline>)>,
    /// One per frame above the bottom: the parenthesis depth a marker was opened at, when it
    /// was opened inside one. The `)` that brings the text back to that depth closes it.
    closes_at: Vec<Option<u32>>,
    /// Parentheses open in the text so far.
    paren: u32,
    pub(super) buf: String,
    pub(super) bold: bool,
    pub(super) italic: bool,
}

impl Out {
    pub(super) fn new() -> Out {
        Out {
            frames: vec![(Vec::new(), Vec::new())],
            closes_at: Vec::new(),
            paren: 0,
            buf: String::new(),
            bold: false,
            italic: false,
        }
    }

    /// A character of plain text. A `)` that closes a parenthesis opened *before* the
    /// innermost marker closes the marker first, so the parenthesis stays outside it.
    pub(super) fn text_char(&mut self, ch: char) {
        match ch {
            '(' => self.paren += 1,
            ')' => {
                if self.paren > 0 && self.closes_at.last() == Some(&Some(self.paren)) {
                    self.close();
                }
                self.paren = self.paren.saturating_sub(1);
            }
            _ => {}
        }
        self.buf.push(ch);
    }

    /// `{{dlc|r}}` with no text of its own: a scope that runs to the end of the value, to
    /// `{{dlc-}}`, or — opened inside a parenthesis — to the `)` that closes it.
    pub(super) fn open_marker(&mut self, only: Vec<Dlc>) {
        self.open(only);
        if let Some(last) = self.closes_at.last_mut() {
            *last = (self.paren > 0).then_some(self.paren);
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
    pub(super) fn flush(&mut self) {
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
            // A different style, another kind of node, or nothing yet: a new text node.
            Some(_) | None => top.push(Inline::Text { text, style }),
        }
    }

    /// Merges with the last node if both are `Text` with the same style: recursion into
    /// unknown templates (`recurse_into_arg`) can return an opening `Text`, and without
    /// this it would split away from the text just accumulated before the template.
    pub(super) fn push(&mut self, node: Inline) {
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
            // Anything else is a new node: two texts of different styles, or not two texts.
            (_, Some(_) | None) => top.push(node),
        }
    }

    pub(super) fn open(&mut self, only: Vec<Dlc>) {
        self.flush();
        self.frames.push((only, Vec::new()));
        self.closes_at.push(None);
    }

    /// Closes the innermost `Edition`; does nothing without any open frame. An edition with
    /// no content is not emitted — and neither is one whose `only` is empty, which is how
    /// `span_restriction` says the code restricts **nothing**: either it named every edition, or it
    /// could not be read and was counted. Either way the words go back to the parent, since
    /// a node declaring its text valid in no edition at all is worse than the text on its
    /// own.
    ///
    /// The frame is opened either way: `{{dlc+|…}}` is closed by a later `{{dlc-}}`, so
    /// skipping the open would leave the close popping somebody else's frame.
    pub(super) fn close(&mut self) {
        self.flush();
        if self.frames.len() > 1 {
            self.closes_at.pop();
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

    pub(super) fn finish(mut self) -> Vec<Inline> {
        while self.frames.len() > 1 {
            self.close();
        }
        self.flush();
        self.frames.pop().map(|f| f.1).unwrap_or_default()
    }
}
