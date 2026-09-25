//! From a wiki page to dataset entries: each infobox gives an entry, of the kind the
//! infobox itself declares (not the folder the page was listed under), and level-2
//! sections with a recognized title give the text. The preamble is discarded: it's the
//! "X is a passive item…" sentence that `catalog` already covers — **except on a
//! transformation**, where it is the only place the wiki says how you become one (B51), and
//! on a boss, a character or a challenge whose infobox writes no `description`.

use serde::{Deserialize, Serialize};

use crate::blocks::parse_blocks;
use crate::editions::Editions;
use crate::infobox::{
    entry_facts, extract_infoboxes, infobox_from, leading_number, InfoboxKind, RawInfobox,
};
use crate::inline::plain;
use crate::resolver::{is_layout_template, Resolver};
use crate::sections::{section_kind, split_page};
use crate::template::{template_segments, Segment};
use crate::{Block, Diagnostics, Entry, Inline, Section, Style};

/// The page kind, as `index.json` classifies it: decides the folder in `raw/` and the
/// template the page was listed from. It does not decide the entries' kind: each infobox
/// says that for itself (a page can contain infoboxes of different kinds).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PageKind {
    Collectible,
    Trinket,
    Achievement,
    Boss,
    Challenge,
    Character,
    Transformation,
}

impl PageKind {
    pub const ALL: [PageKind; 7] = [
        PageKind::Collectible,
        PageKind::Trinket,
        PageKind::Achievement,
        PageKind::Boss,
        PageKind::Challenge,
        PageKind::Character,
        PageKind::Transformation,
    ];

    /// The subfolder of `raw/pages/`.
    pub fn dir(self) -> &'static str {
        match self {
            PageKind::Collectible => "collectible",
            PageKind::Trinket => "trinket",
            PageKind::Achievement => "achievement",
            PageKind::Boss => "boss",
            PageKind::Challenge => "challenge",
            PageKind::Character => "character",
            PageKind::Transformation => "transformation",
        }
    }

    /// The wiki templates whose transclusions list the pages of this kind. Plural because
    /// the characters need two: `Infobox characters` is a different template, not a
    /// spelling of the first, and it holds the four pages that state two playable forms
    /// (B45). A kind with one template is the ordinary case and reads the same.
    pub fn templates(self) -> &'static [&'static str] {
        match self {
            PageKind::Collectible => &["Template:Infobox collectible"],
            PageKind::Trinket => &["Template:Infobox trinket"],
            PageKind::Achievement => &["Template:Infobox achievement"],
            PageKind::Boss => &["Template:Infobox boss"],
            PageKind::Challenge => &["Template:Infobox challenge"],
            PageKind::Character => &["Template:Infobox character", "Template:Infobox characters"],
            PageKind::Transformation => &["Template:Infobox transformation"],
        }
    }
}

/// The key an entry enters the dataset with. A boss carries the bestiary triple.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryKey {
    Item(u32),
    Trinket(u32),
    Achievement(u32),
    Boss(u32, u32, u32),
    Challenge(u32),
    Character(u32),
    Transformation(u32),
}

/// A numeric infobox parameter: the leading digits, because the wiki can follow the
/// number with a note (`349<br>{{plat|PS4}}&nbsp;340`: the PC id, then the PS4 one).
fn number(ib: &RawInfobox, name: &str) -> Option<u32> {
    leading_number(ib.params.get(name)?)
}

/// The entry key for an infobox; `None` if the id is missing, or if an item or trinket
/// isn't in the current edition's table for this page (Tonsil's collectible 474, which in
/// Repentance+ is Broken Glass Cannon).
fn entry_key(kind: InfoboxKind, title: &str, ib: &RawInfobox, r: &Resolver) -> Option<EntryKey> {
    Some(match kind {
        InfoboxKind::Passive | InfoboxKind::Activated => {
            EntryKey::Item(r.item_id_of_page(title, number(ib, "id")?)?)
        }
        InfoboxKind::Trinket => EntryKey::Trinket(r.trinket_id_of_page(title, number(ib, "id")?)?),
        InfoboxKind::Achievement => EntryKey::Achievement(number(ib, "id")?),
        InfoboxKind::Challenge => EntryKey::Challenge(number(ib, "number")?),
        // Our own map first (by `name`, then by title: Lazarus's page also contains
        // Lazarus Risen), the infobox id as a fallback.
        InfoboxKind::Character => EntryKey::Character(
            character(title, ib, r)
                .map(|(id, _)| id)
                .or_else(|| number(ib, "id"))?,
        ),
        // The entity table knows variant and subtype; the infobox only the id.
        InfoboxKind::Boss => match r.boss_key(title) {
            Some((id, variant, subtype)) => EntryKey::Boss(id, variant, subtype),
            None => EntryKey::Boss(number(ib, "id")?, 0, 0),
        },
        // The Cargo table first, because it is the only source with an id for every page:
        // Super Bum's infobox says `id = n/a` and the table maps that onto 1000. By the
        // infobox alone that page would be dropped as having no id.
        InfoboxKind::Transformation => EntryKey::Transformation(
            r.transformation_of_page(title)
                .or_else(|| number(ib, "id"))?,
        ),
    })
}

/// The page's opening sentence, as inline text: what `split_page` puts before the first
/// level-2 heading, with the infobox and the header template gone — `parse_inline` resolves
/// the links and skips the layout templates, and an empty preamble gives an empty list.
///
/// Discarded for items and trinkets, where it is the "X is a passive item…" sentence that
/// `catalog` already covers. B51: on a transformation it carries the requirement; on a boss,
/// a character or a challenge whose infobox has no `description`, it is the summary.
///
/// The **first paragraph** only, with `__TOC__` taken out: 46 boss, character and challenge
/// pages open with several paragraphs — Isaac's runs on into his stats — and 24 carry that
/// magic word, which draws the table of contents and is never prose. No transformation
/// opens with more than one, so B51 reads the same sentence it always did.
fn preamble(text: &str, r: &Resolver, d: &mut Diagnostics) -> Vec<Inline> {
    let (pre, _) = split_page(text);
    let pre = without_the_boxes(&pre).replace("__TOC__", "");
    // The first paragraph that *says* something: on 19 character pages the first one is an
    // image or a layout template, which parses to nothing.
    pre.split("\n\n")
        .map(|p| one_line(parse_blocks(p.trim(), r, d)))
        .find(|inline| !plain(inline).trim().is_empty())
        .unwrap_or_default()
}

/// A paragraph's blocks as one line of inline text. Parsed as blocks, a list's lines are
/// each their own span — so a `{{dlc|…}}` marker ends with its line — and the `*` is gone;
/// then everything is joined by one space, because a summary is drawn as a single line.
fn one_line(blocks: Vec<Block>) -> Vec<Inline> {
    fn walk(blocks: Vec<Block>, out: &mut Vec<Inline>) {
        for block in blocks {
            match block {
                Block::Paragraph { inline } | Block::Heading { inline, .. } => piece(inline, out),
                Block::List { items, .. } => {
                    for item in items {
                        piece(item.inline, out);
                        walk(item.children, out);
                    }
                }
                // A table is not a sentence: nothing in a summary reads one out.
                Block::Table { .. } => {}
            }
        }
    }
    fn piece(inline: Vec<Inline>, out: &mut Vec<Inline>) {
        out.push(Inline::Text {
            text: " ".to_string(),
            style: Style::Plain,
        });
        out.extend(inline);
    }
    let mut out = Vec::new();
    walk(blocks, &mut out);
    let mut space = true;
    squeeze(&mut out, &mut space);
    if let Some(Inline::Text { text, .. }) = out.last_mut() {
        text.truncate(text.trim_end().len());
    }
    out.retain(|i| !matches!(i, Inline::Text { text, .. } if text.is_empty()));
    out
}

/// Every run of whitespace becomes one space, across node boundaries and into editions, and
/// none is left at the start: `space` says whether the text so far ends in one.
fn squeeze(inline: &mut [Inline], space: &mut bool) {
    for node in inline {
        match node {
            Inline::Text { text, .. } => {
                let mut out = String::with_capacity(text.len());
                for ch in text.chars() {
                    if ch.is_whitespace() {
                        if !*space {
                            out.push(' ');
                        }
                        *space = true;
                    } else {
                        out.push(ch);
                        *space = false;
                    }
                }
                *text = out;
            }
            Inline::Edition { inline, .. } => squeeze(inline, space),
            Inline::Ref { .. } | Inline::Concept { .. } => *space = false,
        }
    }
}

/// The preamble text with the infobox and the page header taken out, and **nothing else**:
/// a `{{i|Flip}}` in the same sentence is a reference the prose needs. `template_segments`
/// is what says where a template ends, which a line-by-line pass cannot — an infobox spans
/// a dozen lines and a header one.
fn without_the_boxes(text: &str) -> String {
    template_segments(text)
        .map(|segment| match segment {
            Segment::Text(text) => text,
            Segment::Template { template, .. }
                if template.name.starts_with("infobox") || is_layout_template(&template.name) =>
            {
                ""
            }
            // Any other template is prose: leave it for `parse_inline` to resolve.
            Segment::Template { source, .. } => source,
        })
        .collect()
}

/// The page's edition context: the range the **first** infobox declares, and not each
/// entry's own.
///
/// That is the wiki's own rule rather than a convenience. `{{page dlc}}` carries
/// `{{assert once|page dlc}}`, so the first infobox on a page sets the context every span
/// below is tested against; a later infobox reaches `{{section dlc}}`'s other branch and
/// narrows a section, not the page. Ultra Greed is the page that says so — two bosses in
/// one file, `dlc = a` then `dlc = a+`, and the sections belong to the page, not to either
/// of them. Read as "each entry's own", the note about the ending chest, marked `na+`, has
/// nothing in common with Ultra Greedier's `a+` and is thrown away; it is the one span in
/// 4831 that the counter caught the first time this pass ran.
#[cfg(test)]
fn page_editions(text: &str) -> Editions {
    page_editions_of(&extract_infoboxes(text))
}

/// The page's range from infoboxes already extracted: `parse_page` reads them once and hands
/// the list here, rather than extracting them a second time (card #80, P6).
fn page_editions_of(infoboxes: &[RawInfobox]) -> Editions {
    infoboxes
        .first()
        .and_then(|ib| Editions::parse(ib.params.get("dlc")?))
        .unwrap_or(Editions::ALL)
}

/// Narrows every `{{dlc|…}}` span on an entry by the page's range, the way the wiki's
/// `{{context test}}` does before it draws an icon.
///
/// A code says what it says on its own; a page says what it says; and the reader is looking
/// at one inside the other. Abyss exists from Repentance and carries a line marked
/// `{{dlc|nr+}}` — "removed in Repentance+" — which read alone names the four editions
/// before Repentance+, three of which never had the item. Narrowed, it names Repentance,
/// which is what the line means. 847 of the snapshot's 4831 spans move this way.
///
/// A page that declares no range narrows nothing, which is half of them. When the two share
/// no edition the span is dropped and counted: the wiki draws its own error there, so it is
/// the wiki contradicting itself and not a code we failed to read.
///
/// **The entry's own fields only.** The sections and the preamble belong to the page and are
/// shared by every entry on it, so they are narrowed once, where they are read
/// (`narrow_sections`): narrowed here, on each entry's copy, one span was counted once per
/// infobox (card #80, P6). The description may carry the preamble; narrowing it again changes
/// nothing and counts nothing, because a narrowed span already fits the page.
fn narrow_to_page(entry: &mut Entry, page: Editions, d: &mut Diagnostics) {
    if page.is_all() {
        return;
    }
    narrow_inline(&mut entry.description, page, d);
    for field in entry.infobox.inlines_mut() {
        narrow_inline(field, page, d);
    }
}

/// The page's preamble, narrowed once like its sections: every entry that takes it shares it.
fn narrowed_preamble(text: &str, page: Editions, r: &Resolver, d: &mut Diagnostics) -> Vec<Inline> {
    let mut read = preamble(text, r, d);
    if !page.is_all() {
        narrow_inline(&mut read, page, d);
    }
    read
}

fn narrow_sections(sections: &mut [Section], page: Editions, d: &mut Diagnostics) {
    if page.is_all() {
        return;
    }
    for section in sections {
        for block in &mut section.blocks {
            narrow_block(block, page, d);
        }
    }
}

fn narrow_block(block: &mut Block, page: Editions, d: &mut Diagnostics) {
    match block {
        Block::Paragraph { inline } | Block::Heading { inline, level: _ } => {
            narrow_inline(inline, page, d)
        }
        Block::List { ordered: _, items } => {
            for item in items {
                narrow_inline(&mut item.inline, page, d);
                for child in &mut item.children {
                    narrow_block(child, page, d);
                }
            }
        }
        Block::Table { header, rows } => {
            for cell in header {
                narrow_inline(cell, page, d);
            }
            for row in rows {
                for cell in row {
                    narrow_inline(cell, page, d);
                }
            }
        }
    }
}

/// An `Edition` whose range survives narrowing keeps it; one left with nothing is unwrapped
/// into its parent, words and all, exactly as an unreadable code is.
fn narrow_inline(inline: &mut Vec<Inline>, page: Editions, d: &mut Diagnostics) {
    let mut out = Vec::with_capacity(inline.len());
    for node in std::mem::take(inline) {
        match node {
            Inline::Edition { only, mut inline } => {
                narrow_inline(&mut inline, page, d);
                let narrowed = Editions::of(&only).intersect(page);
                if narrowed.is_empty() {
                    d.spans_outside_their_page += 1;
                    out.extend(inline);
                } else {
                    out.push(Inline::Edition {
                        only: narrowed.list(),
                        inline,
                    });
                }
            }
            other @ (Inline::Text { .. } | Inline::Ref { .. } | Inline::Concept { .. }) => {
                out.push(other)
            }
        }
    }
    *inline = out;
}

/// The page's kept sections, in the order they appear; ones with an unrecognized title
/// count among the discarded.
fn sections(text: &str, r: &Resolver, d: &mut Diagnostics) -> Vec<Section> {
    let (_preamble, raw) = split_page(text);
    raw.iter()
        .filter_map(|s| match section_kind(&s.title) {
            Some(kind) => Some(Section {
                kind,
                blocks: parse_blocks(&s.body, r, d),
            }),
            None => {
                d.discarded_section(&s.title);
                None
            }
        })
        .collect()
}

/// The infobox's `name`, if it's present and not empty.
fn name_param(ib: &RawInfobox) -> Option<&str> {
    ib.params
        .get("name")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
}

/// The character of an infobox according to our own map: by `name`, otherwise by title.
fn character(title: &str, ib: &RawInfobox, r: &Resolver) -> Option<(u32, String)> {
    name_param(ib)
        .and_then(|n| r.character_of_page(n))
        .or_else(|| r.character_of_page(title))
}

/// The entry's title. An achievement takes the infobox's `name` (a page holds dozens of
/// them); a character its `name`, then the name from our own map ("Lazarus Risen" lives
/// on Lazarus's page, `??? (Character)` is called `???`); everything else the page title,
/// because there `name` carries edition markup.
fn entry_title(kind: InfoboxKind, title: &str, ib: &RawInfobox, r: &Resolver) -> String {
    let name = name_param(ib);
    match kind {
        InfoboxKind::Achievement => name.unwrap_or(title).to_string(),
        InfoboxKind::Character => name
            .map(str::to_string)
            .or_else(|| character(title, ib, r).map(|(_, n)| n))
            .unwrap_or_else(|| title.to_string()),
        InfoboxKind::Passive
        | InfoboxKind::Activated
        | InfoboxKind::Trinket
        | InfoboxKind::Boss
        | InfoboxKind::Challenge
        | InfoboxKind::Transformation => title.to_string(),
    }
}

/// A page as its entries see it: what they all carry (title, revision, text, edition range),
/// and the two readings they share, each made at most once.
///
/// Lazily, because an achievement page takes neither the sections nor the preamble; and at
/// most once, because a page with two infoboxes has one of each, and reading either twice
/// would count its diagnostics twice.
struct Page<'a> {
    title: &'a str,
    revid: u64,
    text: &'a str,
    editions: Editions,
    sections: Option<Vec<Section>>,
    preamble: Option<Vec<Inline>>,
}

impl Page<'_> {
    /// The kept sections, narrowed to the page's range.
    fn sections(&mut self, r: &Resolver, d: &mut Diagnostics) -> Vec<Section> {
        let (text, editions) = (self.text, self.editions);
        self.sections
            .get_or_insert_with(|| {
                let mut read = sections(text, r, d);
                narrow_sections(&mut read, editions, d);
                read
            })
            .clone()
    }

    /// The opening paragraph, narrowed to the page's range.
    fn preamble(&mut self, r: &Resolver, d: &mut Diagnostics) -> Vec<Inline> {
        let (text, editions) = (self.text, self.editions);
        self.preamble
            .get_or_insert_with(|| narrowed_preamble(text, editions, r, d))
            .clone()
    }
}

/// The entries of a page, one per recognized infobox, each of its own infobox's kind.
/// The page's sections go to every entry except achievements, which are containers with
/// no text of their own; they're parsed once. An infobox with no key is counted in
/// `pages_without_id` and skipped.
pub fn parse_page(
    title: &str,
    revid: u64,
    text: &str,
    r: &Resolver,
    d: &mut Diagnostics,
) -> Vec<(EntryKey, Entry)> {
    let infoboxes = extract_infoboxes(text);
    let mut page = Page {
        title,
        revid,
        text,
        editions: page_editions_of(&infoboxes),
        sections: None,
        preamble: None,
    };
    infoboxes
        .into_iter()
        .filter_map(|ib| entry_of(&ib, &mut page, r, d))
        .collect()
}

/// The entry one infobox gives, or `None` — counted — for an infobox of no kind, or of a kind
/// whose key the page does not state.
fn entry_of(
    ib: &RawInfobox,
    page: &mut Page,
    r: &Resolver,
    d: &mut Diagnostics,
) -> Option<(EntryKey, Entry)> {
    let Some(kind) = InfoboxKind::of(&ib.name) else {
        d.unknown_infobox(&ib.name);
        return None;
    };
    let Some(key) = entry_key(kind, page.title, ib, r) else {
        d.pages_without_id += 1;
        return None;
    };
    let sections = entry_sections(kind, page, r, d);
    let facts = entry_facts(ib, r, d);
    let description = entry_description(kind, facts.description, page, r, d);
    let mut entry = Entry {
        title: entry_title(kind, page.title, ib, r),
        revid: page.revid,
        description,
        dlc: facts.dlc,
        unlocked_by: facts.unlocked_by,
        infobox: infobox_from(kind, ib, page.text, r, d),
        sections,
    };
    narrow_to_page(&mut entry, page.editions, d);
    Some((key, entry))
}

/// The page's sections, for every kind but an achievement: a row on a storage page, with no
/// text of its own.
fn entry_sections(
    kind: InfoboxKind,
    page: &mut Page,
    r: &Resolver,
    d: &mut Diagnostics,
) -> Vec<Section> {
    match kind {
        InfoboxKind::Achievement => Vec::new(),
        InfoboxKind::Passive
        | InfoboxKind::Activated
        | InfoboxKind::Trinket
        | InfoboxKind::Boss
        | InfoboxKind::Challenge
        | InfoboxKind::Character
        | InfoboxKind::Transformation => page.sections(r, d),
    }
}

/// The entry's summary, from the infobox's own `description` (`own`) and the page's opening
/// paragraph, by kind.
fn entry_description(
    kind: InfoboxKind,
    own: Vec<Inline>,
    page: &mut Page,
    r: &Resolver,
    d: &mut Diagnostics,
) -> Vec<Inline> {
    match kind {
        // B51: a transformation's preamble is the only place the wiki says **how you become
        // one** — Adult's "upon taking three Puberty pills" is there and nowhere else on the
        // page — so for this kind alone the discarded sentence is kept, in front of the
        // infobox's own `description`. That parameter restates the Effects section on all
        // sixteen pages (Guppy's is empty) and on none of them says how the transformation
        // happens, so nothing is dropped by putting it second.
        InfoboxKind::Transformation => {
            let mut out = page.preamble(r, d);
            // Two sentences written in two places meet in one field, and nothing in either
            // carries the space between them.
            if !out.is_empty() && !own.is_empty() {
                out.push(Inline::Text {
                    text: " ".to_string(),
                    style: Style::Plain,
                });
            }
            out.extend(own);
            out
        }
        // Bosses and characters have no `description` parameter, and 27 of 45 challenges
        // leave it out: the opening paragraph is the only summary those pages write. It
        // stands in when the infobox is silent, never alongside it.
        InfoboxKind::Boss | InfoboxKind::Challenge | InfoboxKind::Character if own.is_empty() => {
            page.preamble(r, d)
        }
        // What the wiki files as an achievement's `description` is the unlock paper's line,
        // and it went to the infobox's `quote`: it is not a summary, and read as one it put
        // "???" under 136 titles.
        InfoboxKind::Achievement => Vec::new(),
        InfoboxKind::Passive
        | InfoboxKind::Activated
        | InfoboxKind::Trinket
        | InfoboxKind::Boss
        | InfoboxKind::Challenge
        | InfoboxKind::Character => own,
    }
}

// Tests extract one variant and panic on the rest: the wildcard is the assertion.
#[allow(clippy::wildcard_enum_match_arm)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::fixtures::test_resolver;
    use crate::{Block, Diagnostics, SectionKind};

    /// Measured on 2026-09-13: `Template:Infobox transformation` exists and is transcluded by
    /// exactly the sixteen pages the Cargo table has rows for. The kind is declared like the
    /// other six so that `fetch`, which walks `PageKind::ALL`, picks the pages up without a
    /// special case — until it exists, no transformation page is downloaded at all.
    /// B51: a transformation's preamble is the only place the wiki says **how you become
    /// one** — Adult states "upon taking three Puberty pills" there and nowhere else — and the
    /// parser drops preambles by a rule written for the "X is a passive item…" sentence that
    /// `catalog` already covers. `transformation::requires` even reads that same line for its
    /// digit and throws the sentence away.
    ///
    /// It is kept ahead of the infobox's own `description`, which on all sixteen pages
    /// restates the Effects section (Guppy's is empty) and on none of them says how the
    /// transformation happens.
    #[test]
    fn a_transformations_preamble_is_kept_ahead_of_its_description() {
        let src = "{{header transformations}}
{{infobox transformation
 | id = 1
 | description = Grants flight.
}}

'''Beelzebub''' is a [[transformation]], turning Isaac into a fly after picking up 3 fly items.

== Effects ==
* a
";
        let mut d = Diagnostics::default();
        let v = parse_page("Beelzebub", 7, src, &test_resolver(), &mut d);
        let text = crate::plain(&v[0].1.description);
        assert!(
            text.starts_with("Beelzebub is a transformation, turning Isaac into a fly after picking up 3 fly items."),
            "{text}"
        );
        // The two sentences are written in two places and meet in one field: the space
        // between them belongs to neither, so it is put here.
        assert!(text.ends_with("3 fly items. Grants flight."), "{text}");
    }

    /// Bosses and characters have no `description` parameter at all, and 27 of the 45
    /// challenge pages leave it out: on those pages the only summary the wiki writes is the
    /// opening paragraph, which the parser used to discard, and all 169 of them reached the
    /// screen with no line under the title. The preamble stands in when the infobox is
    /// silent, and only then: a challenge that fills the parameter keeps its own words.
    #[test]
    fn a_boss_character_or_challenge_without_a_description_takes_the_preamble() {
        let desc = |title: &str, src: &str| {
            let v = parse_page(title, 1, src, &test_resolver(), &mut Diagnostics::default());
            crate::plain(&v[0].1.description)
        };
        assert_eq!(
            desc(
                "Mom",
                "{{infobox boss\n | id = 45\n}}\n'''Mom''' is a [[boss]].\n== Behavior ==\nx\n"
            ),
            "Mom is a boss."
        );
        assert_eq!(
            desc(
                "Cain",
                "{{infobox character\n | id = 2\n}}\n'''Cain''' is a character.\n== Notes ==\nn\n"
            ),
            "Cain is a character."
        );
        assert_eq!(
            desc(
                "Pitch Black",
                "{{infobox challenge\n | number = 1\n}}\n'''Pitch Black''' is challenge #1.\n== Difficulty ==\nx\n"
            ),
            "Pitch Black is challenge #1."
        );
        assert_eq!(
            desc(
                "XXXXXXXXL",
                "{{infobox challenge\n | number = 21\n | description = Every floor is XL.\n}}\n'''XXXXXXXXL''' is challenge #21.\n== Difficulty ==\nx\n"
            ),
            "Every floor is XL."
        );
        // A summary is the first paragraph. 46 of those pages open with several — Isaac's
        // runs on into his stats and his tips — and 24 carry `__TOC__`, a magic word that
        // draws the table of contents and is never prose.
        assert_eq!(
            desc(
                "Isaac",
                "{{infobox character\n | id = 0\n}}\n\n\n[[File:Isaac.png]]\n\n'''Isaac''' is a character.\n\nHe starts with the D6.\n__TOC__\n== Notes ==\nn\n"
            ),
            "Isaac is a character."
        );
        assert_eq!(
            desc(
                "Pitch Black",
                "{{infobox challenge\n | number = 1\n}}\n'''Pitch Black''' is challenge #1.\n__TOC__\n\n== Difficulty ==\nx\n"
            ),
            "Pitch Black is challenge #1."
        );
    }

    /// 27 boss pages open with a list — "can appear:" and one line per edition. Read as one
    /// run of inline text the `*` reached the screen and every line's `{{dlc|…}}` marker ran
    /// on into the next line; read as blocks, each line is its own span, joined by a space.
    #[test]
    fn a_list_in_the_preamble_is_one_line_per_edition() {
        let src = "{{infobox boss\n | id = 45\n}}\n'''Mom''' can appear:\n* {{dlc|nr}} In the Depths.\n* {{dlc|r}} Only in the Mausoleum.\n\n== Behavior ==\nx\n";
        let v = parse_page("Mom", 1, src, &test_resolver(), &mut Diagnostics::default());
        let description = &v[0].1.description;
        assert_eq!(
            crate::plain(description),
            "Mom can appear: In the Depths. Only in the Mausoleum."
        );
        let editions: Vec<&Vec<crate::Dlc>> = description
            .iter()
            .filter_map(|i| match i {
                crate::Inline::Edition { only, .. } => Some(only),
                _ => None,
            })
            .collect();
        assert_eq!(editions.len(), 2, "{description:?}");
    }

    #[test]
    fn the_transformation_kind_names_its_template_and_its_folder() {
        assert_eq!(PageKind::Transformation.dir(), "transformation");
        assert_eq!(
            PageKind::Transformation.templates(),
            ["Template:Infobox transformation"]
        );
        assert!(PageKind::ALL.contains(&PageKind::Transformation));
    }

    /// B45: a kind's pages are the transclusions of **its templates**, plural, because the
    /// characters are listed by two — `Infobox character` for the thirty pages that hold
    /// one form, and `Infobox characters` for the four that hold two. Enumerating only the
    /// singular is not a query that *misses* those four, it is a query they are not in,
    /// which is why the fetch reported no error while losing eight playable characters.
    ///
    /// Measured 2026-09-14 against the wiki: `Infobox characters` has exactly 4
    /// transclusions in namespace 0, and they are those four pages.
    #[test]
    fn the_characters_are_listed_by_two_templates_not_one() {
        assert_eq!(
            PageKind::Character.templates(),
            ["Template:Infobox character", "Template:Infobox characters"]
        );
        // Every other kind still has exactly one, so nothing else changed shape.
        for kind in PageKind::ALL {
            let n = kind.templates().len();
            assert_eq!(
                n,
                if kind == PageKind::Character { 2 } else { 1 },
                "{kind:?}"
            );
        }
    }

    /// B45's mechanism, not its symptom. `extract_infoboxes` takes any template whose name
    /// starts with `infobox`, and `InfoboxKind::of` answers `None` for the ones we do not
    /// know — after which the page simply produces nothing, with no error and no counter.
    /// That is how four character pages could have been downloaded and parsed into zero
    /// entries without a word. Counted now, the way `unknown_entities` counts the entities
    /// that shipped undecoded for months.
    #[test]
    fn an_infobox_whose_template_we_do_not_know_is_counted() {
        let src = "{{infobox rune\n | id = 1\n}}\n== Effects ==\n* a\n";
        let mut d = Diagnostics::default();
        let v = parse_page("Rune of Hagalaz", 7, src, &test_resolver(), &mut d);
        assert!(v.is_empty());
        assert_eq!(d.unknown_infoboxes.get("infobox rune"), Some(&1));
    }

    /// The counter has to be able to stay quiet, or it says nothing by saying nothing.
    #[test]
    fn a_template_we_know_is_not_counted_as_unknown() {
        let src = "{{infobox passive collectible\n | id = 25\n}}\n";
        let mut d = Diagnostics::default();
        parse_page("Breakfast", 7, src, &test_resolver(), &mut d);
        assert!(d.unknown_infoboxes.is_empty());
    }

    #[test]
    fn the_three_common_facts_land_on_the_entry_not_the_infobox() {
        let src = "{{infobox passive collectible\n | id = 25\n | dlc = r\n | description = Tears up\n | unlocked by = Epic Fetus\n}}\n== Effects ==\n* a\n";
        let mut d = Diagnostics::default();
        let v = parse_page("Breakfast", 7, src, &test_resolver(), &mut d);
        let e = &v[0].1;
        // `r` is "added in Repentance", so it names Repentance **and** Repentance+.
        assert_eq!(
            e.dlc,
            vec![crate::Dlc::Repentance, crate::Dlc::RepentancePlus]
        );
        assert_eq!(e.unlocked_by, Some(crate::Target::Achievement { id: 62 }));
        assert!(matches!(
            e.description.first(),
            Some(crate::Inline::Text { text, .. }) if text.contains("Tears up")
        ));
    }

    /// The editions of every `Edition` node under an entry, in the order they are found.
    fn edition_nodes(inline: &[crate::Inline]) -> Vec<Vec<crate::Dlc>> {
        let mut out = Vec::new();
        for i in inline {
            if let crate::Inline::Edition { only, inline } = i {
                out.push(only.clone());
                out.extend(edition_nodes(inline));
            }
        }
        out
    }

    fn entry_editions(e: &Entry) -> Vec<Vec<crate::Dlc>> {
        let mut out = edition_nodes(&e.description);
        for s in &e.sections {
            for b in &s.blocks {
                if let crate::Block::List { items, .. } = b {
                    for item in items {
                        out.extend(edition_nodes(&item.inline));
                    }
                }
            }
        }
        out
    }

    /// Abyss is the page that showed this: it exists from Repentance (`dlc = r`) and
    /// carries a line marked `{{dlc|nr+}}`, "removed in Repentance+". Read on its own that
    /// code names the four editions before Repentance+, three of which the item does not
    /// exist in — so the line would be labelled for editions that never had it. The wiki
    /// narrows a span by the page it sits on (`{{context test}}`), and what is left is
    /// Repentance alone, which is what the line means.
    ///
    /// 847 of the 4831 spans in the snapshot are narrowed this way.
    #[test]
    fn a_span_is_narrowed_by_the_editions_its_page_declares() {
        let src = "{{infobox passive collectible\n | id = 25\n | dlc = r\n | quote = {{dlc|nr+|gone later}}\n}}\n== Effects ==\n* {{dlc|nr+|gone later}}\n";
        let mut d = Diagnostics::default();
        let v = parse_page("Breakfast", 7, src, &test_resolver(), &mut d);
        assert_eq!(
            entry_editions(&v[0].1),
            vec![vec![crate::Dlc::Repentance]],
            "the body's span"
        );
        // The infobox's fields go through the same pass: a quote carries editions too.
        let crate::Infobox::Item { quote, .. } = &v[0].1.infobox else {
            panic!("a collectible page carries an item infobox")
        };
        assert_eq!(edition_nodes(quote), vec![vec![crate::Dlc::Repentance]]);
        assert_eq!(d.spans_outside_their_page, 0);
    }

    /// A page with two infoboxes has one context, and it is the **first** one's: the wiki
    /// gives `{{page dlc}}` an `{{assert once}}`, so the second infobox narrows a section
    /// rather than the page. Ultra Greed is the page that forced the rule — two bosses,
    /// `dlc = a` then `dlc = a+`, sharing one set of sections. Read as "each entry's own",
    /// the note marked `na+` shares no edition with Ultra Greedier's `a+` and is thrown
    /// away; it was the one span in 4831 this pass dropped the first time it ran.
    #[test]
    fn a_page_with_two_infoboxes_takes_its_context_from_the_first() {
        let src = "{{infobox boss\n | dlc = a\n | id = 1\n}}\ntext\n{{infobox boss\n | dlc = a+\n | id = 2\n}}\n";
        assert_eq!(
            page_editions(src).list(),
            vec![
                crate::Dlc::Afterbirth,
                crate::Dlc::AfterbirthPlus,
                crate::Dlc::Repentance,
                crate::Dlc::RepentancePlus
            ]
        );
    }

    /// A page that declares no range narrows nothing: 492 of the 1113 pages in the
    /// snapshot, and the reason an absent parameter has to read as "every edition" rather
    /// than "none" — as "none" it would erase every badge on half the wiki.
    #[test]
    fn a_page_that_declares_no_range_leaves_its_spans_alone() {
        let src = "{{infobox passive collectible\n | id = 25\n}}\n== Effects ==\n* {{dlc|r+|only the last}}\n";
        let mut d = Diagnostics::default();
        let v = parse_page("Breakfast", 7, src, &test_resolver(), &mut d);
        assert_eq!(
            entry_editions(&v[0].1),
            vec![vec![crate::Dlc::RepentancePlus]]
        );
    }

    /// When the two ranges share no edition the wiki draws its own error, so the span is a
    /// contradiction on the wiki's side and not a code we failed to read. The words stay,
    /// the badge goes, and the counter says it happened — 0 times on the snapshot of
    /// 2026-09-15.
    #[test]
    fn a_span_its_page_leaves_no_edition_for_keeps_its_words_and_is_counted() {
        let src = "{{infobox passive collectible\n | id = 25\n | dlc = a\n}}\n== Effects ==\n* {{dlc|na|only in Rebirth}}\n";
        let mut d = Diagnostics::default();
        let v = parse_page("Breakfast", 7, src, &test_resolver(), &mut d);
        assert!(
            entry_editions(&v[0].1).is_empty(),
            "a badge naming no edition: {:?}",
            v[0].1.sections
        );
        let crate::Block::List { items, .. } = &v[0].1.sections[0].blocks[0] else {
            panic!("the Effects section holds a list")
        };
        assert!(
            items[0]
                .inline
                .iter()
                .any(|i| matches!(i, crate::Inline::Text { text, .. } if text.contains("only in Rebirth"))),
            "the words went with it: {:?}", items[0]
        );
        assert_eq!(d.spans_outside_their_page, 1);
    }

    /// Card #80, P6: the counter counts spans, not copies. A page with two infoboxes shares
    /// its sections between the two entries, and narrowing each entry's copy counted one span
    /// once per entry.
    #[test]
    fn a_span_in_sections_two_entries_share_is_counted_once() {
        let src = "{{infobox boss\n | id = 1\n | dlc = a\n}}\n{{infobox boss\n | id = 2\n | dlc = a\n}}\n== Behavior ==\n* {{dlc|na|only in Rebirth}}\n";
        let mut d = Diagnostics::default();
        let v = parse_page("Twins", 7, src, &test_resolver(), &mut d);
        assert_eq!(v.len(), 2, "two entries share the page");
        assert_eq!(d.spans_outside_their_page, 1);
    }

    #[test]
    fn collectible_page_yields_one_entry_with_kept_sections() {
        let src = "{{infobox passive collectible\n | id = 25\n | quote = x\n}}\n{{cit|p|r}}\n\n== Effects ==\n* a\n== Trivia ==\n* t\n== Notes ==\nn\n";
        let mut d = Diagnostics::default();
        let v = parse_page("Breakfast", 7, src, &test_resolver(), &mut d);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].0, EntryKey::Item(25));
        let e = &v[0].1;
        assert_eq!(e.title, "Breakfast");
        assert_eq!(e.revid, 7);
        assert_eq!(
            e.sections.iter().map(|s| s.kind).collect::<Vec<_>>(),
            vec![SectionKind::Effects, SectionKind::Notes]
        );
        assert!(matches!(e.sections[0].blocks[0], Block::List { .. }));
        assert_eq!(d.discarded_sections.get("Trivia"), Some(&1));
        assert_eq!(d.pages_without_id, 0);
    }

    #[test]
    fn achievement_storage_page_yields_many() {
        let src = "{{storage page}}\n{{infobox achievement\n | name = Epic Fetus\n | link = Breakfast\n | id = 62\n}} {{infobox achievement\n | name = Cain\n | id = 2\n}} {{infobox achievement\n | name = NoId\n}}";
        let mut d = Diagnostics::default();
        let v = parse_page("Achievements/Rebirth 1", 1, src, &test_resolver(), &mut d);
        assert_eq!(
            v.iter().map(|(k, _)| k.clone()).collect::<Vec<_>>(),
            vec![EntryKey::Achievement(62), EntryKey::Achievement(2)]
        );
        assert_eq!(v[0].1.title, "Epic Fetus");
        assert!(v[0].1.sections.is_empty());
        assert_eq!(d.pages_without_id, 1);
    }

    #[test]
    fn an_id_with_a_platform_note_is_its_leading_number() {
        // Achievements/Afterbirth † 2: the PC id followed by the PS4 one.
        let src = "{{infobox achievement\n | name = Black Hole\n | id = 349<br>{{plat|PS4}}&nbsp;340\n}} {{infobox achievement\n | name = Negative\n | id = -1\n}}";
        let mut d = Diagnostics::default();
        let v = parse_page(
            "Achievements/Afterbirth † 2",
            1,
            src,
            &test_resolver(),
            &mut d,
        );
        assert_eq!(
            v.iter().map(|(k, _)| k.clone()).collect::<Vec<_>>(),
            vec![EntryKey::Achievement(349)]
        );
        assert_eq!(d.pages_without_id, 1);
    }

    #[test]
    fn boss_key_from_entity_table_or_infobox() {
        let src = "{{infobox boss\n | id = 45\n | base hp = 1\n}}\n== Behavior ==\nx\n";
        let v = parse_page("Mom", 1, src, &test_resolver(), &mut Diagnostics::default());
        assert_eq!(v[0].0, EntryKey::Boss(45, 0, 0));
        let v = parse_page(
            "Unknown Boss",
            1,
            "{{infobox boss\n | id = 999\n}}",
            &test_resolver(),
            &mut Diagnostics::default(),
        );
        assert_eq!(v[0].0, EntryKey::Boss(999, 0, 0));
    }

    #[test]
    fn each_infobox_is_dispatched_by_its_own_kind() {
        // Tonsil: the page is listed among collectibles, but trinket 97 is the only entry
        // for the current edition; collectible 474 (Afterbirth+) is discarded and counted.
        let src = "{{infobox trinket\n | id = 97\n}}\nintro\n{{infobox passive collectible\n | id = 474\n}}\n== Effects ==\ne\n";
        let mut d = Diagnostics::default();
        let v = parse_page("Tonsil", 3, src, &test_resolver(), &mut d);
        assert_eq!(
            v.iter().map(|(k, _)| k.clone()).collect::<Vec<_>>(),
            vec![EntryKey::Trinket(97)]
        );
        assert_eq!(v[0].1.title, "Tonsil");
        assert_eq!(v[0].1.sections.len(), 1);
        assert_eq!(d.pages_without_id, 1);
    }

    #[test]
    fn a_page_with_two_entries_gives_the_sections_to_both_and_parses_them_once() {
        let src = "{{infobox activated collectible\n | id = 550\n}}\n== Effects ==\na\n{{infobox passive collectible\n | id = 551\n}}\n== Trivia ==\nt\n";
        let mut d = Diagnostics::default();
        let v = parse_page("Broken Shovel", 1, src, &test_resolver(), &mut d);
        assert_eq!(
            v.iter().map(|(k, _)| k.clone()).collect::<Vec<_>>(),
            vec![EntryKey::Item(550), EntryKey::Item(551)]
        );
        assert_eq!(v[0].1.sections, v[1].1.sections);
        assert_eq!(v[0].1.sections[0].kind, SectionKind::Effects);
        assert_eq!(d.discarded_sections.get("Trivia"), Some(&1));
    }

    #[test]
    fn an_alternate_character_form_takes_its_own_name() {
        let src = "{{infobox character\n | name = Lazarus\n | id = 8\n}}\n{{infobox character\n | name = Lazarus Risen\n | id = 11\n}}\n{{infobox monster\n | name = Dark Esau\n | id = 866\n}}\n== Notes ==\nn\n";
        let mut d = Diagnostics::default();
        let v = parse_page("Lazarus", 1, src, &test_resolver(), &mut d);
        assert_eq!(
            v.iter().map(|(k, _)| k.clone()).collect::<Vec<_>>(),
            vec![EntryKey::Character(8), EntryKey::Character(11)]
        );
        assert_eq!(v[0].1.title, "Lazarus");
        assert_eq!(v[1].1.title, "Lazarus Risen");
        assert_eq!(v[1].1.sections.len(), 1);
        assert_eq!(d.pages_without_id, 0);
    }

    #[test]
    fn a_character_page_keys_from_our_map_before_the_infobox_id() {
        let mut d = Diagnostics::default();
        // The wiki writes 14 for Isaac: the key is the one from our own map.
        let v = parse_page(
            "Isaac",
            1,
            "{{infobox character\n | name = Isaac\n | id = 14\n}}",
            &test_resolver(),
            &mut d,
        );
        assert_eq!(v[0].0, EntryKey::Character(0));
        assert_eq!(v[0].1.title, "Isaac");
        // Without `name`: the page title, without « (Character)», and the map's name.
        let v = parse_page(
            "??? (Character)",
            1,
            "{{infobox character\n | id = 4\n}}",
            &test_resolver(),
            &mut d,
        );
        assert_eq!(v[0].0, EntryKey::Character(4));
        assert_eq!(v[0].1.title, "???");
        // A character the map doesn't know: the infobox id, as before.
        let v = parse_page(
            "Nobody",
            1,
            "{{infobox character\n | id = 99\n}}",
            &test_resolver(),
            &mut d,
        );
        assert_eq!(v[0].0, EntryKey::Character(99));
        assert_eq!(v[0].1.title, "Nobody");
        assert_eq!(d.pages_without_id, 0);
    }

    #[test]
    fn an_item_page_the_table_does_not_know_is_dropped() {
        let mut d = Diagnostics::default();
        let v = parse_page(
            "Nope",
            1,
            "{{infobox passive collectible\n | id = 5\n}}",
            &test_resolver(),
            &mut d,
        );
        assert!(v.is_empty());
        assert_eq!(d.pages_without_id, 1);
        // Same page, an id different from the table's: it's discarded all the same.
        let mut d = Diagnostics::default();
        let v = parse_page(
            "Breakfast",
            1,
            "{{infobox passive collectible\n | id = 26\n}}",
            &test_resolver(),
            &mut d,
        );
        assert!(v.is_empty());
        assert_eq!(d.pages_without_id, 1);
    }
}
