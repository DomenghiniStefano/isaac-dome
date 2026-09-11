//! What a row carries in place of a base64 image.
//!
//! An `unlock` payload with the icons embedded is ~7 MB on a real profile, 94% of it
//! base64. Instead each row carries a **reference**: the Tauri crate turns it into a URL
//! its own protocol handler serves, and the webview does the lazy loading, the caching and
//! the de-duplication that the UI would otherwise have to write by hand.
//!
//! Two halves, and both live here so they can't drift: `to_path` renders the reference,
//! `parse` reads it back. This crate stays pure — like `target_sprite`, it says **which
//! file**, never its bytes.

use catalog::{AchievementId, Catalog, ItemId, SpriteRef};

use crate::catalog_view::{item_kind, ItemKindView};
use crate::marks::{character_for, BOSSES, CHARACTERS};

/// The two levels of a mark. The game draws them as two different symbols, not one tinted
/// (DESIGN-BRIEF.md §5.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkTier {
    Normal,
    Hard,
}

/// The things the interface draws an icon for. Bosses and challenges aren't here because
/// `UnlockTarget` carries no image for them: the game has no single picture for a
/// challenge, and the brief asks for a typographic placeholder instead of a guess.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconRef {
    Achievement {
        id: u32,
    },
    /// The kind is **part of the key**, not decoration: 186 ids are shared between a
    /// collectible and a trinket, so an id alone names two different pictures.
    Item {
        kind: ItemKindView,
        id: u32,
    },
    /// A completion-matrix column's mark symbol: a piece of `completion_widget.png` or of
    /// the online lobby's sheet, resolved by `mark_source`, never by the catalog.
    Mark {
        column: usize,
        tier: MarkTier,
    },
    /// The co-op menu head of a completion-matrix row, resolved through `character_for`.
    Head {
        row: usize,
    },
}

fn kind_token(k: ItemKindView) -> &'static str {
    match k {
        ItemKindView::Passive => "passive",
        ItemKindView::Active => "active",
        ItemKindView::Familiar => "familiar",
        ItemKindView::Trinket => "trinket",
    }
}

fn kind_from_token(s: &str) -> Option<ItemKindView> {
    match s {
        "passive" => Some(ItemKindView::Passive),
        "active" => Some(ItemKindView::Active),
        "familiar" => Some(ItemKindView::Familiar),
        "trinket" => Some(ItemKindView::Trinket),
        // Exhaustive by construction above: a new variant of `ItemKindView` breaks
        // `kind_token`, which is the pair of this one, so it can't be forgotten silently.
        _ => None,
    }
}

fn tier_token(t: MarkTier) -> &'static str {
    match t {
        MarkTier::Normal => "normal",
        MarkTier::Hard => "hard",
    }
}

fn tier_from_token(s: &str) -> Option<MarkTier> {
    match s {
        "normal" => Some(MarkTier::Normal),
        "hard" => Some(MarkTier::Hard),
        // A string a webview handed us, paired with `tier_token` like the kinds above.
        _ => None,
    }
}

impl IconRef {
    /// The path half of the URL, without a scheme: `achievement/19`, `item/passive/92`,
    /// `mark/9/hard`, `head/0`.
    pub fn to_path(&self) -> String {
        match self {
            IconRef::Achievement { id } => format!("achievement/{id}"),
            IconRef::Item { kind, id } => format!("item/{}/{id}", kind_token(*kind)),
            IconRef::Mark { column, tier } => format!("mark/{column}/{}", tier_token(*tier)),
            IconRef::Head { row } => format!("head/{row}"),
        }
    }

    /// The inverse. `None` for anything we didn't write: the handler answers "no image"
    /// rather than guessing, and never panics on a string a webview handed it. A mark past
    /// the matrix's columns or a head past its rows isn't ours either, so the handler
    /// refuses it before it opens an archive.
    pub fn parse(path: &str) -> Option<IconRef> {
        let mut parts = path.split('/');
        let out = match (parts.next()?, parts.next()?, parts.next()) {
            ("achievement", id, None) => IconRef::Achievement {
                id: id.parse().ok()?,
            },
            ("item", kind, Some(id)) => IconRef::Item {
                kind: kind_from_token(kind)?,
                id: id.parse().ok()?,
            },
            ("mark", column, Some(tier)) => IconRef::Mark {
                column: column.parse::<usize>().ok().filter(|&c| c < BOSSES.len())?,
                tier: tier_from_token(tier)?,
            },
            ("head", row, None) => IconRef::Head {
                row: row
                    .parse::<usize>()
                    .ok()
                    .filter(|&r| r < CHARACTERS.len())?,
            },
            _ => return None,
        };
        // A trailing segment means the string isn't ours, whatever the prefix said.
        parts.next().is_none().then_some(out)
    }
}

/// The file the catalog names for a reference, if it knows it.
///
/// `None` covers both "no such id" and "the catalog is older than the reference" — the
/// caller draws the placeholder either way, and nothing here invents a path.
pub fn icon_source<'a>(c: &'a Catalog, r: &IconRef) -> Option<&'a SpriteRef> {
    match *r {
        IconRef::Achievement { id } => c.achievement(AchievementId(id)).map(|a| &a.sprite),
        IconRef::Item { kind, id } => c.item(item_kind(kind), ItemId(id)).map(|i| &i.sprite),
        IconRef::Head { row } => character_for(row, c).and_then(|ch| ch.head.as_ref()),
        // Not the catalog's: the symbols are pieces of the widget's sheets, see `mark_source`.
        IconRef::Mark { .. } => None,
    }
}

/// The URI scheme the app registers for these references.
///
/// It lives here, next to `to_path`, so the two halves of a URL are decided in one place —
/// but this crate never builds the URL itself: on Windows the webview sees a rewritten
/// `http://isaac.localhost/…` origin, and knowing that is the Tauri crate's job, not a
/// pure crate's.
pub const ICON_SCHEME: &str = "isaac";
