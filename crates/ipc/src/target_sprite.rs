//! From a wiki reference to the image the game actually has.
//!
//! The wiki dataset carries **text**: sections, and typed references (`Target`). The
//! images live in the game's archives and `catalog` is the one that knows them. Neither
//! crate can make the connection on its own — `catalog` doesn't know the wiki, `wiki`
//! doesn't know the installed game — so it lives here, the crate whose job is exactly
//! this.
//!
//! Like everything in `ipc`: pure logic. It returns **which file** to show, not its
//! bytes: reading them is I/O, done by the caller, with the same `ResourceSet` as
//! everything else.

use std::collections::HashMap;

use catalog::{AchievementId, Catalog, ChallengeId, CharacterId, ItemId, ItemKind, SpriteRef};
use wiki::Target;

/// The outcome of the resolution. Three cases, not an `Option`, because the two ways of
/// having no image are meant to be drawn differently: brief §5.6 asks for one placeholder
/// for "I don't know this" and another for "there is none".
#[derive(Debug)]
pub enum TargetSprite<'a> {
    /// The catalog knows which file to show.
    Found(&'a SpriteRef),
    /// The target exists, but the game doesn't draw an image for that kind of thing
    /// (transformations, rooms), or we have no way to name it (pickups and floors: the
    /// art is in the archives, but the name-to-file map isn't).
    NoArt,
    /// The catalog doesn't know this id. Happens when the wiki dataset is newer than the
    /// installed game, or on an id the game doesn't use.
    Unknown,
}

/// The image for a wiki reference, if the catalog knows which one it is.
///
/// Exhaustive over `Target`: a new variant added to the wiki must break the build here,
/// not silently turn into a page with no figure.
pub fn target_sprite<'a>(c: &'a Catalog, t: &Target) -> TargetSprite<'a> {
    match t {
        // The wiki doesn't distinguish passives, actives and familiars: it just says
        // `Item { id }`. The three share the same id space, so at most one will match.
        Target::Item { id } => [ItemKind::Passive, ItemKind::Active, ItemKind::Familiar]
            .iter()
            .find_map(|k| c.item(*k, ItemId(*id)))
            .map_or(TargetSprite::Unknown, |i| TargetSprite::Found(&i.sprite)),
        Target::Trinket { id } => c
            .item(ItemKind::Trinket, ItemId(*id))
            .map_or(TargetSprite::Unknown, |i| TargetSprite::Found(&i.sprite)),
        Target::Achievement { id } => c
            .achievement(AchievementId(*id))
            .map_or(TargetSprite::Unknown, |a| TargetSprite::Found(&a.sprite)),
        Target::Character { id } => c
            .character(CharacterId(*id))
            .map_or(TargetSprite::Unknown, |p| TargetSprite::Found(&p.portrait)),
        // The game doesn't draw challenges: `gfx/challenge` doesn't exist. The only image
        // that belongs to a challenge is the achievement earned by completing it.
        Target::Challenge { number } => match c.challenge(ChallengeId(*number)) {
            None => TargetSprite::Unknown,
            Some(ch) => match ch.rewards.first().and_then(|a| c.achievement(*a)) {
                Some(a) => TargetSprite::Found(&a.sprite),
                None => TargetSprite::NoArt,
            },
        },
        // The boss: type and variant are written into the portrait file's name.
        Target::Entity { id, variant, .. } => match entity_portraits(c).get(&(*id, *variant)) {
            Some(s) => TargetSprite::Found(s),
            None => TargetSprite::Unknown,
        },
        Target::Transformation { .. }
        | Target::Stage { .. }
        | Target::Room { .. }
        | Target::Pickup { .. } => TargetSprite::NoArt,
    }
}

/// The `(type, variant) → portrait` index, derived from the file names in
/// `bossportraits.xml`: `Portrait_20.0_Monstro.png` is entity 20, variant 0.
///
/// This is not a name-based match: the game *writes* the entity's key into the file
/// name, and it's the same key the wiki uses for its `{{e|…}}`. Portraits that don't
/// declare it (`Portrait_Cadavra.png`) are left out: they're unreachable from a
/// `Target::Entity`, and no similarity-based fallback recovers them.
///
/// The subtype takes no part: portraits are declared by type and variant only, and a
/// different subtype is still the same boss (its champion versions).
fn entity_portraits(c: &Catalog) -> HashMap<(u32, u32), &SpriteRef> {
    c.bosses()
        .filter_map(|b| Some((entity_key(&b.portrait.path)?, &b.portrait)))
        .collect()
}

/// `…/Portrait_<type>.<variant>_<Name>.png` → `(type, variant)`.
fn entity_key(path: &str) -> Option<(u32, u32)> {
    let file = path.rsplit(['/', '\\']).next()?;
    let resto = file.strip_prefix("Portrait_")?;
    // The boss name follows the first `_`, and can itself contain dots: cut there
    // first, then split off type and variant.
    let chiave = resto.split('_').next()?;
    let (tipo, variante) = chiave.split_once('.')?;
    Some((tipo.parse().ok()?, variante.parse().ok()?))
}
