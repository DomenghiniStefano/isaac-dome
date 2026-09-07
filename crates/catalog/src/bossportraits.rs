//! `bossportraits.xml`: 103 bosses with literal name and portrait. Two declared
//! portraits (*The Beast*, *Cadavra*) don't exist in the archives: the catalog reports
//! them as-is from the file, and it's up to whoever resolves the sprite to find out.

use crate::diagnostics::{Diagnostic, SkipReason, Source};
use crate::ids::{AchievementId, BossId};
use crate::items::normalize_root;
use crate::sprite::SpriteRef;
use crate::xml::{elements, Element};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Boss {
    pub id: BossId,
    pub name: String,
    pub portrait: SpriteRef,
    pub unlocked_by: Option<AchievementId>,
}

pub fn parse(bytes: &[u8], diagnostics: &mut Vec<Diagnostic>) -> Vec<Boss> {
    let els = match elements(bytes) {
        Ok(els) => els,
        Err(_) => {
            diagnostics.push(Diagnostic::SourceUnreadable {
                source: Source::BossPortraits,
            });
            return Vec::new();
        }
    };
    let root = els
        .iter()
        .find(|e| e.name == "bosses")
        .and_then(|e| e.attr("root"))
        .map(normalize_root)
        .filter(|r| !r.is_empty())
        .unwrap_or_else(|| "gfx/ui/boss".to_string());

    els.iter()
        .filter(|e| e.name == "boss")
        .filter_map(|e| boss_from(e, &root, diagnostics))
        .collect()
}

fn boss_from(e: &Element, root: &str, d: &mut Vec<Diagnostic>) -> Option<Boss> {
    let skip = |id: Option<u32>, reason: SkipReason, d: &mut Vec<Diagnostic>| {
        d.push(Diagnostic::ElementSkipped {
            source: Source::BossPortraits,
            id,
            reason,
        });
        None
    };
    let Some(raw_id) = e.attr("id") else {
        return skip(None, SkipReason::MissingId, d);
    };
    let Ok(id) = raw_id.parse::<u32>() else {
        return skip(None, SkipReason::MalformedId, d);
    };
    let Some(portrait) = e.attr("portrait") else {
        return skip(Some(id), SkipReason::MissingSprite, d);
    };
    let Some(name) = e.attr("name") else {
        return skip(Some(id), SkipReason::MissingName, d);
    };
    Some(Boss {
        id: BossId(id),
        name: name.to_string(),
        portrait: SpriteRef::whole(format!("{root}/{portrait}")),
        unlocked_by: e
            .attr("achievement")
            .and_then(|a| a.parse().ok())
            .map(AchievementId),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const B: &[u8] = b"<bosses root=\"gfx/ui/boss/\" anm2=\"versusscreen.anm2\">
\t<boss id=\"1\" name=\"Monstro\" nameimage=\"BossName_20.0_Monstro.png\" portrait=\"Portrait_20.0_Monstro.png\" pivotX=\"96\" pivotY=\"132\" />
\t<boss id=\"9\" name=\"Famine\" portrait=\"Portrait_Famine.png\" achievement=\"5\" />
\t<boss id=\"100\" name=\"The Beast\" portrait=\"Portrait_The Beast.png\" />
\t<boss name=\"NoId\" portrait=\"x.png\" />
\t<boss id=\"7\" name=\"NoPortrait\" />
</bosses>";

    #[test]
    fn portrait_uses_the_declared_root_and_keeps_spaces_in_names() {
        let mut d = Vec::new();
        let b = parse(B, &mut d);
        assert_eq!(b[0].portrait.path, "gfx/ui/boss/Portrait_20.0_Monstro.png");
        assert_eq!(
            b[2].portrait.path, "gfx/ui/boss/Portrait_The Beast.png",
            "the space in the file name stays: that's how the game has it"
        );
        assert_eq!(b[0].name, "Monstro");
        assert_eq!(b[1].unlocked_by, Some(AchievementId(5)));
        assert_eq!(b[0].unlocked_by, None);
    }

    #[test]
    fn malformed_rows_are_skipped() {
        let mut d = Vec::new();
        let b = parse(B, &mut d);
        assert_eq!(b.len(), 3);
        assert!(d.contains(&Diagnostic::ElementSkipped {
            source: Source::BossPortraits,
            id: None,
            reason: SkipReason::MissingId
        }));
        assert!(d.contains(&Diagnostic::ElementSkipped {
            source: Source::BossPortraits,
            id: Some(7),
            reason: SkipReason::MissingSprite
        }));
    }
}
