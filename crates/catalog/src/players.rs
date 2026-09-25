//! `players.xml`: characters with a portrait. Name keys repeat (Isaac and Tainted Isaac
//! are both `#ISAAC_NAME`): what tells them apart is the `_b` token in the portrait
//! name (`_b.png`, `_b_dead.png`), the game's convention for Tainted forms. Verified
//! against the real file.

use crate::diagnostics::{Diagnostic, SkipReason, Source};
use crate::ids::{AchievementId, CharacterId};
use crate::sprite::SpriteRef;
use crate::text::Text;
use crate::xml::{self, Element};

const SOURCE: Source = Source::Players;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Character {
    pub id: CharacterId,
    pub name: Text,
    pub portrait: SpriteRef,
    /// The cell of `coop menu.png`; `heads` fills it in, if the map holds up.
    pub head: Option<SpriteRef>,
    pub tainted: bool,
    pub unlocked_by: Option<AchievementId>,
}

pub fn parse(bytes: &[u8], diagnostics: &mut Vec<Diagnostic>) -> Vec<Character> {
    let Some(els) = xml::read(bytes, SOURCE, diagnostics) else {
        return Vec::new();
    };
    let portraitroot = xml::root_attr(&els, "players", "portraitroot", "gfx/ui/stage");
    els.iter()
        .filter(|e| e.name == "player")
        .filter_map(|e| character_from(e, &portraitroot, diagnostics))
        .collect()
}

/// The `_b` convention as a segment of its own, not just an immediate suffix before
/// `.png`: the "dead" form of Tainted Lazarus Risen uses `..._b_dead.png`, verified
/// against the real file on 2026-09-03 (id 38 of `players.xml`).
fn is_tainted_portrait(portrait: &str) -> bool {
    let lower = portrait.to_ascii_lowercase();
    let stem = lower.rsplit_once('.').map_or(lower.as_str(), |(s, _)| s);
    stem.split('_').any(|segment| segment == "b")
}

fn character_from(e: &Element, portraitroot: &str, d: &mut Vec<Diagnostic>) -> Option<Character> {
    let id = xml::required_id(e, "id", SOURCE, d)?;
    let portrait = xml::required_attr(e, "portrait", id, SkipReason::MissingSprite, SOURCE, d)?;
    let name = xml::required_attr(e, "name", id, SkipReason::MissingName, SOURCE, d)?;
    Some(Character {
        id: CharacterId(id),
        name: Text::from_attr(name),
        portrait: SpriteRef::whole(format!("{portraitroot}/{portrait}")),
        head: None,
        tainted: is_tainted_portrait(portrait),
        unlocked_by: e
            .attr("achievement")
            .and_then(|a| a.parse().ok())
            .map(AchievementId),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const PLAYERS: &[u8] = b"<players root=\"gfx/characters/costumes/\" portraitroot=\"gfx/ui/stage/\" nameimageroot=\"gfx/ui/boss/\">
\t<player id=\"0\" name=\"#ISAAC_NAME\" skin=\"Character_001_Isaac.png\" hp=\"6\" nameimage=\"PlayerName_01_Isaac.png\" portrait=\"PlayerPortrait_Isaac.png\" birthright=\"#ISAAC_BIRTHRIGHT\" />
\t<player id=\"1\" name=\"#MAGDALENE_NAME\" skin=\"Character_002_Magdalene.png\" achievement=\"1\" portrait=\"PlayerPortrait_Magdalene.png\" birthright=\"#MAGDALENE_BIRTHRIGHT\" />
\t<player id=\"21\" name=\"#ISAAC_NAME\" skin=\"Character_001b_Isaac.png\" achievement=\"474\" portrait=\"PlayerPortrait_Isaac_b.png\" birthright=\"#ISAAC_B_BIRTHRIGHT\" />
\t<player name=\"#NO_ID\" portrait=\"x.png\" />
\t<player id=\"9\" name=\"#NO_PORTRAIT\" />
</players>";

    fn parsed() -> (Vec<Character>, Vec<Diagnostic>) {
        let mut d = Vec::new();
        let c = parse(PLAYERS, &mut d);
        (c, d)
    }

    #[test]
    fn portrait_uses_the_declared_portraitroot() {
        let (c, _) = parsed();
        assert_eq!(c[0].portrait.path, "gfx/ui/stage/PlayerPortrait_Isaac.png");
        assert!(
            c[0].head.is_none(),
            "the head comes from the anm2, not from here"
        );
    }

    #[test]
    fn tainted_is_the_b_suffix_of_the_portrait_and_keys_repeat() {
        let (c, _) = parsed();
        assert!(!c[0].tainted);
        assert!(c[2].tainted);
        assert_eq!(
            c[0].name, c[2].name,
            "same name, two characters: that's how it is in the file"
        );
        assert_eq!(c[2].id, CharacterId(21));
        assert_eq!(c[2].unlocked_by, Some(AchievementId(474)));
        assert_eq!(c[0].unlocked_by, None, "Isaac doesn't unlock");
    }

    #[test]
    fn the_b_token_is_tainted_even_with_a_suffix_after_it() {
        // Tainted Lazarus Risen (id 38 of players.xml, verified on 2026-09-03) uses
        // "..._b_dead.png": the "_b.png" suffix alone isn't enough.
        assert!(is_tainted_portrait("PlayerPortrait_Lazarus_b_dead.png"));
        assert!(is_tainted_portrait("PlayerPortrait_Isaac_b.png"));
        assert!(!is_tainted_portrait("PlayerPortrait_Bluebaby.png"));
        assert!(!is_tainted_portrait("PlayerPortrait_Bethany.png"));
    }

    #[test]
    fn malformed_rows_are_skipped_with_a_reason() {
        let (c, d) = parsed();
        assert_eq!(c.len(), 3);
        assert!(d.contains(&Diagnostic::ElementSkipped {
            source: Source::Players,
            id: None,
            reason: SkipReason::MissingId
        }));
        assert!(d.contains(&Diagnostic::ElementSkipped {
            source: Source::Players,
            id: Some(9),
            reason: SkipReason::MissingSprite
        }));
    }

    #[test]
    fn the_portrait_is_checked_before_the_name_and_a_malformed_id_carries_no_id() {
        let mut d = Vec::new();
        let c = parse(
            b"<players>
<player id=\"x\" name=\"#A\" portrait=\"a.png\" />
<player id=\"4\" />
<player id=\"5\" portrait=\"b.png\" />
</players>",
            &mut d,
        );
        assert!(c.is_empty());
        let skipped = |id, reason| Diagnostic::ElementSkipped {
            source: Source::Players,
            id,
            reason,
        };
        assert_eq!(
            d,
            vec![
                skipped(None, SkipReason::MalformedId),
                skipped(Some(4), SkipReason::MissingSprite),
                skipped(Some(5), SkipReason::MissingName),
            ]
        );
    }

    #[test]
    fn without_a_portraitroot_the_portrait_takes_the_game_folder() {
        let mut d = Vec::new();
        let c = parse(
            b"<players portraitroot=\"resources/\"><player id=\"0\" name=\"#I\" portrait=\"p.png\" achievement=\"x\" /></players>",
            &mut d,
        );
        assert_eq!(c[0].portrait.path, "gfx/ui/stage/p.png");
        assert_eq!(c[0].unlocked_by, None, "a malformed link is no link");
    }

    #[test]
    fn junk_is_empty_with_one_diagnostic() {
        let mut d = Vec::new();
        assert!(parse(b"<players><player", &mut d).is_empty());
        assert_eq!(
            d,
            vec![Diagnostic::SourceUnreadable {
                source: Source::Players
            }]
        );
    }
}
