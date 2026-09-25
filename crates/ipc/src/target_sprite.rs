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

use std::collections::{BTreeMap, HashMap, HashSet};

use catalog::{AchievementId, Catalog, ChallengeId, CharacterId, ItemId, ItemKind, SpriteRef};
use wiki::{Dataset, Target};

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

/// The image for a wiki reference, if the catalog knows which one it is. `bosses` is the
/// catalog's [`boss_keys`], settled once by whoever holds the catalog.
///
/// Exhaustive over `Target`: a new variant added to the wiki must break the build here,
/// not silently turn into a page with no figure.
pub fn target_sprite<'a>(c: &'a Catalog, bosses: &BossKeys, t: &Target) -> TargetSprite<'a> {
    match t {
        // The wiki doesn't distinguish passives, actives and familiars: it just says
        // `Item { id }`. The three share the same id space, so at most one will match.
        Target::Item { id } => c
            .collectible(ItemId(*id))
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
        // The boss: the portrait of the row `bosses` gives this type and variant to.
        Target::Entity { id, variant, .. } => match entity_portrait(c, bosses, (*id, *variant)) {
            Some(s) => TargetSprite::Found(s),
            None => TargetSprite::Unknown,
        },
        Target::Transformation { .. }
        | Target::Stage { .. }
        | Target::Room { .. }
        | Target::Concept { .. } => TargetSprite::NoArt,
    }
}

/// The portrait of the row `bosses` gives `(type, variant)` to. A key is given to one row at
/// most, so there is at most one.
///
/// The subtype takes no part: portraits are declared by type and variant only, and a
/// different subtype is still the same boss (its champion versions).
fn entity_portrait<'a>(
    c: &'a Catalog,
    bosses: &BossKeys,
    key: (u32, u32),
) -> Option<&'a SpriteRef> {
    c.bosses()
        .filter(|b| bosses.get(&b.name) == Some(key))
        .last()
        .map(|b| &b.portrait)
}

/// The entity key of every row of `bossportraits.xml`, by the row's name: what links a boss
/// to its page and draws it with its portrait, the same decision taken once.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BossKeys(BTreeMap<String, (u32, u32)>);

impl BossKeys {
    /// No row keyed: what a caller without a catalog hands on, where there is no boss to key.
    pub const NONE: &'static BossKeys = &BossKeys(BTreeMap::new());

    /// The key settled for the row named `name`, if one was.
    pub(crate) fn get(&self, name: &str) -> Option<(u32, u32)> {
        self.0.get(name).copied()
    }
}

/// The entity key of every row of `bossportraits.xml`, by the row's name.
///
/// Two sources, and **the wiki's own key wins**. The index answers a `Target::Entity`
/// the wiki wrote, so the wiki's notion of the key is the very thing being asked about;
/// the file name is what settles the rows the wiki doesn't name. Measured on 2026-09-21,
/// reading the file name alone left 27 of the dataset's 102 boss pages with no image:
/// seventeen portraits write no key at all (`Portrait_Dogma.png`), three write one the
/// wiki doesn't use (Tuff Twins is `19.100` in the file and `19.2` on the wiki), and two
/// differ from their page by a word (`Turdling` against *Turdlings*).
///
/// **A key two rows claim is claimed by neither.** Four are contested — `45.0` by *Mom*
/// and *Mom (Mausoleum)*, `78.0`, `406.0` and `902.0` likewise — and collecting them into
/// a map used to let the last row win in silence, which draws a boss with another's face.
/// A wrong picture is worse than none, because nothing about it looks wrong.
///
/// The name match is **equality**, never similarity: case and punctuation are dropped, a
/// leading `the` with them, and a name has to be claimed by exactly one page and one row.
/// That is the distinction this module used to miss when it said no fallback recovers
/// the unkeyed portraits — true of a fuzzy one, false of this.
///
/// **Settled once per catalog, by whoever holds it** — the app keeps it beside the catalog —
/// and handed to every lookup, so no lookup compares a cache. The dataset is a parameter:
/// without one, only the portraits' file names speak.
pub fn boss_keys(c: &Catalog, dataset: Option<&Dataset>) -> BossKeys {
    let rows: Vec<(&str, &str)> = c
        .bosses()
        .map(|b| (b.name.as_str(), b.portrait.path.as_str()))
        .collect();
    let pages = dataset.map(wiki_boss_keys).unwrap_or_default();
    BossKeys(
        merge_keys(&rows, &pages)
            .into_iter()
            .map(|(name, key)| (name.to_string(), key))
            .collect(),
    )
}

/// The rules, apart from the catalog so they can be read and tested on rows written by
/// hand. `rows` is `(name, portrait path)`.
///
/// Three tiers, weakest evidence last: the page the row's **name** is, the page its
/// portrait's **file name** is, and the key that file name **declares**. A tier assigns a
/// key only to a row that has none yet, and only to a key no earlier tier spoke about —
/// including one it dropped as contested, because a key two rows wanted is not freed by
/// refusing it to both.
///
/// The order is what keeps *Mom (Mausoleum)* off *Mom*'s page: the two share a portrait,
/// so the file name reaches the same page for both, and only the row's own name tells them
/// apart.
fn merge_keys<'a>(
    rows: &[(&'a str, &'a str)],
    wiki: &HashMap<String, (u32, u32)>,
) -> HashMap<&'a str, (u32, u32)> {
    type Tier<'t> = &'t dyn Fn(&str, &str) -> Option<(u32, u32)>;
    let page = |s: &str| wiki.get(&normalized(s)).copied();
    let tiers: [Tier; 3] = [
        &|name, _| page(name),
        &|_, path| page(portrait_stem(path)?),
        &|_, path| entity_key(path),
    ];

    let mut out: HashMap<&'a str, (u32, u32)> = HashMap::new();
    let mut spoken: HashSet<(u32, u32)> = HashSet::new();
    for tier in tiers {
        let mut claims: HashMap<(u32, u32), Vec<&'a str>> = HashMap::new();
        for (name, path) in rows {
            if out.contains_key(name) {
                continue;
            }
            match tier(name, path) {
                Some(k) if !spoken.contains(&k) => claims.entry(k).or_default().push(name),
                _ => {}
            }
        }
        for (k, names) in &claims {
            spoken.insert(*k);
            if let [only] = names[..] {
                out.insert(only, *k);
            }
        }
    }
    out
}

/// The dataset's boss pages as `normalized title → (type, variant)`. A title two pages
/// share names neither of them. The subtype is dropped here, as it is in the lookup.
fn wiki_boss_keys(ds: &Dataset) -> HashMap<String, (u32, u32)> {
    let titled = ds.bosses.iter().filter_map(|(key, entry)| {
        Some((
            normalized(&entry.title),
            Dataset::boss_key_type_and_variant(key)?,
        ))
    });
    let seen = titled.fold(
        HashMap::<String, Option<(u32, u32)>>::new(),
        |mut seen, (title, key)| {
            seen.entry(title)
                .and_modify(|v| *v = None)
                .or_insert(Some(key));
            seen
        },
    );
    seen.into_iter()
        .filter_map(|(t, k)| Some((t, k?)))
        .collect()
}

/// Case, spaces and punctuation dropped, and a leading `the` with them: the wiki writes
/// *The Horny Boys* where the game writes `Horny Boys`. Nothing else is normalized —
/// anything looser stops being equality.
fn normalized(s: &str) -> String {
    let n: String = s
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .flat_map(char::to_lowercase)
        .collect();
    n.strip_prefix("the").map_or(n.clone(), str::to_string)
}

/// `…/Portrait_902.0_Wormwood.png` → `Wormwood`, `…/Portrait_Shell.png` → `Shell`.
fn portrait_stem(path: &str) -> Option<&str> {
    let rest = portrait_file(path)?;
    let rest = rest.strip_suffix(".png").unwrap_or(rest);
    match rest.split_once('_') {
        Some((head, name)) if portrait_key(head).is_some() => Some(name),
        _ => Some(rest),
    }
}

/// `…/Portrait_<type>.<variant>_<Name>.png` → `(type, variant)`.
fn entity_key(path: &str) -> Option<(u32, u32)> {
    // The boss name follows the first `_`, and can itself contain dots: cut there
    // first, then split off type and variant.
    portrait_key(portrait_file(path)?.split('_').next()?)
}

/// A portrait's file name after `Portrait_`, `.png` left on. `None` for any other file.
fn portrait_file(path: &str) -> Option<&str> {
    path.rsplit(['/', '\\']).next()?.strip_prefix("Portrait_")
}

/// `<type>.<variant>` → the two numbers: the key a portrait's file name may start with,
/// and the one reading of it behind both functions above.
fn portrait_key(head: &str) -> Option<(u32, u32)> {
    let (kind, variant) = head.split_once('.')?;
    Some((kind.parse().ok()?, variant.parse().ok()?))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wiki(pairs: &[(&str, (u32, u32))]) -> HashMap<String, (u32, u32)> {
        pairs
            .iter()
            .map(|(t, k)| (normalized(t), *k))
            .collect::<HashMap<_, _>>()
    }

    #[test]
    fn a_portrait_with_no_key_in_its_name_is_reached_through_the_page_title() {
        let rows = [("Dogma", "gfx/ui/boss/Portrait_Dogma.png")];
        let keys = merge_keys(&rows, &wiki(&[("Dogma", (950, 0))]));
        assert_eq!(keys.get("Dogma"), Some(&(950, 0)));
    }

    #[test]
    fn the_page_title_beats_the_key_written_in_the_file_name() {
        // Tuff Twins: `19.100` in the file, `19.2` on the wiki. The lookup answers a
        // reference the wiki wrote, so the wiki's key is the one to hold.
        let rows = [("Tuff Twins", "gfx/ui/boss/Portrait_19.100_TuffTwins.png")];
        let keys = merge_keys(&rows, &wiki(&[("Tuff Twins", (19, 2))]));
        assert_eq!(keys.get("Tuff Twins"), Some(&(19, 2)));
    }

    #[test]
    fn the_portraits_file_name_names_the_page_when_the_row_does_not() {
        // The row is `Horny Boys`, the page is *The Horny Boys*, the file agrees with
        // the page. Equality after normalization, on either of the row's two names.
        let rows = [("Horny Boys", "gfx/ui/boss/Portrait_HornyBoys.png")];
        let keys = merge_keys(&rows, &wiki(&[("The Horny Boys", (920, 0))]));
        assert_eq!(keys.get("Horny Boys"), Some(&(920, 0)));
    }

    #[test]
    fn a_row_the_wiki_does_not_name_keeps_the_key_from_its_file_name() {
        let rows = [("Monstro", "gfx/ui/boss/Portrait_20.0_Monstro.png")];
        let keys = merge_keys(&rows, &wiki(&[]));
        assert_eq!(keys.get("Monstro"), Some(&(20, 0)));
    }

    #[test]
    fn a_key_two_file_names_declare_is_given_to_neither() {
        // `45.0` is written by both Mom's portrait and Mom (Mausoleum)'s. With no page to
        // separate them, a map would hand the key to whichever came last: the other boss's
        // face, with nothing about it looking wrong.
        let rows = [
            ("Mom", "gfx/ui/boss/Portrait_45.0_Mom.png"),
            ("Mom (Mausoleum)", "gfx/ui/boss/Portrait_45.0_Mom.png"),
        ];
        let keys = merge_keys(&rows, &wiki(&[]));
        assert_eq!(keys.get("Mom"), None);
        assert_eq!(keys.get("Mom (Mausoleum)"), None);
    }

    #[test]
    fn the_page_settles_a_key_two_file_names_declare() {
        let rows = [
            ("Mom", "gfx/ui/boss/Portrait_45.0_Mom.png"),
            ("Mom (Mausoleum)", "gfx/ui/boss/Portrait_45.0_Mom.png"),
        ];
        let keys = merge_keys(&rows, &wiki(&[("Mom", (45, 0))]));
        assert_eq!(keys.get("Mom"), Some(&(45, 0)));
        assert_eq!(
            keys.get("Mom (Mausoleum)"),
            None,
            "the key is Mom's: the other row takes nothing, not the next best thing"
        );
    }

    #[test]
    fn a_row_moved_by_its_page_does_not_keep_the_key_it_declared() {
        // Wormwood's file says `902.0`, which is The Rainmaker's page. Once the wiki moves
        // Wormwood to `62.3`, `902.0` has to be free for the row that page names.
        let rows = [
            ("Wormwood", "gfx/ui/boss/Portrait_902.0_Wormwood.png"),
            ("The Rainmaker", "gfx/ui/boss/Portrait_902.0_Rainmaker.png"),
        ];
        let keys = merge_keys(
            &rows,
            &wiki(&[("Wormwood", (62, 3)), ("The Rainmaker", (902, 0))]),
        );
        assert_eq!(keys.get("Wormwood"), Some(&(62, 3)));
        assert_eq!(keys.get("The Rainmaker"), Some(&(902, 0)));
    }

    #[test]
    fn a_title_two_pages_share_names_neither_row() {
        // Two rows reaching one page is the same ambiguity as two pages under one title,
        // and gets the same answer: no key, so no picture.
        let rows = [
            ("Gemini", "gfx/ui/boss/Portrait_Gemini.png"),
            ("Gemini", "gfx/ui/boss/Portrait_Gemini2.png"),
        ];
        let keys = merge_keys(&rows, &wiki(&[("Gemini", (79, 0))]));
        assert!(keys.is_empty());
    }

    /// Loose: the type and variant of a dataset key, whatever follows them — the subtype takes
    /// no part in finding a portrait.
    #[test]
    fn a_dataset_key_gives_its_first_two_numbers_whatever_follows() {
        assert_eq!(Dataset::boss_key_type_and_variant("20.0.0"), Some((20, 0)));
        assert_eq!(Dataset::boss_key_type_and_variant("19.2.1"), Some((19, 2)));
        assert_eq!(Dataset::boss_key_type_and_variant("20.0"), Some((20, 0)));
        assert_eq!(Dataset::boss_key_type_and_variant("20.0.x"), Some((20, 0)));
        assert_eq!(
            Dataset::boss_key_type_and_variant("20.0.0.0"),
            Some((20, 0))
        );
        for refused in ["20", "x.0.0", "20.x.0", "", ".0.0"] {
            assert_eq!(
                Dataset::boss_key_type_and_variant(refused),
                None,
                "{refused:?}"
            );
        }
    }

    /// The key a portrait's file name declares: the two numbers before the first `_`.
    #[test]
    fn a_portrait_declares_its_key_before_the_first_underscore() {
        assert_eq!(
            entity_key("gfx/ui/boss/Portrait_902.0_Wormwood.png"),
            Some((902, 0))
        );
        assert_eq!(
            entity_key(r"gfx\ui\boss\Portrait_19.100_TuffTwins.png"),
            Some((19, 100))
        );
        // No `_` after the key: the whole rest is read, `.png` included, and it is no key.
        assert_eq!(entity_key("gfx/ui/boss/Portrait_20.0.png"), None);
        assert_eq!(entity_key("gfx/ui/boss/Portrait_20.0"), Some((20, 0)));
        assert_eq!(entity_key("gfx/ui/boss/Portrait_Dogma.png"), None);
        assert_eq!(entity_key("gfx/ui/boss/Portrait_1.2.3_X.png"), None);
        assert_eq!(entity_key("gfx/ui/boss/20.0_Monstro.png"), None);
    }

    #[test]
    fn the_stem_keeps_a_head_that_is_not_a_key() {
        // A head with three numbers is not a key, so the stem is the whole rest.
        assert_eq!(
            portrait_stem("gfx/ui/boss/Portrait_1.2.3_X.png"),
            Some("1.2.3_X")
        );
        assert_eq!(
            portrait_stem("gfx/ui/boss/Portrait_Big_Horn.png"),
            Some("Big_Horn")
        );
        assert_eq!(portrait_stem("Portrait_20.0.png"), Some("20.0"));
        assert_eq!(portrait_stem("gfx/ui/boss/Portrait_Shell"), Some("Shell"));
    }

    #[test]
    fn the_stem_is_the_name_after_the_key_when_there_is_one() {
        assert_eq!(
            portrait_stem("gfx/ui/boss/Portrait_902.0_Wormwood.png"),
            Some("Wormwood")
        );
        assert_eq!(
            portrait_stem("gfx/ui/boss/Portrait_Shell.png"),
            Some("Shell")
        );
        assert_eq!(
            portrait_stem("gfx/ui/boss/Portrait_The Beast.png"),
            Some("The Beast")
        );
        assert_eq!(portrait_stem("gfx/ui/boss/other.png"), None);
    }

    #[test]
    fn normalization_is_equality_and_nothing_looser() {
        assert_eq!(normalized("Mom's Heart"), normalized("Moms Heart"));
        assert_eq!(normalized("The Horny Boys"), normalized("Horny Boys"));
        assert_ne!(
            normalized("Turdling"),
            normalized("Turdlings"),
            "a plural is a different name: only the file name reaches that page"
        );
        assert_ne!(normalized("Mom"), normalized("Mom (Mausoleum)"));
    }
}
