//! `achievements.xml`: entries with literal English text (two quote styles, which
//! quick-xml normalizes) and, for some of them, the unlock condition in the XML comment
//! that precedes the element — 637 entries and 283 conditions in the Repentance+ file of
//! 2026-09-04 (`tests/real_data.rs`). The condition is data: it's kept raw, and the graph
//! (M2) interprets it, not this crate.

use crate::diagnostics::{Diagnostic, SkipReason, Source};
use crate::ids::AchievementId;
use crate::sprite::SpriteRef;
use crate::xml::{self, Element};

const SOURCE: Source = Source::Achievements;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Achievement {
    pub id: AchievementId,
    /// Literal English, e.g. `You unlocked "Magdalene"`. Not a key.
    pub text: String,
    /// The XML comment right before the element, if there is one.
    pub unlock_condition: Option<String>,
    /// The `steam_description` attribute, where present (from the Afterbirth files onward).
    /// For nine Repentance achievements it's the only place that says which challenge they
    /// reward.
    pub steam_description: Option<String>,
    pub sprite: SpriteRef,
}

pub fn parse(bytes: &[u8], diagnostics: &mut Vec<Diagnostic>) -> Vec<Achievement> {
    let Some(els) = xml::read(bytes, SOURCE, diagnostics) else {
        return Vec::new();
    };
    let gfxroot = xml::root_attr(&els, "achievements", "gfxroot", "gfx/ui/achievement");
    els.iter()
        .filter(|e| e.name == "achievement")
        .filter_map(|e| achievement_from(e, &gfxroot, diagnostics))
        .collect()
}

fn achievement_from(e: &Element, gfxroot: &str, d: &mut Vec<Diagnostic>) -> Option<Achievement> {
    let id = xml::required_id(e, "id", SOURCE, d)?;
    let gfx = xml::required_attr(e, "gfx", id, SkipReason::MissingSprite, SOURCE, d)?;
    Some(Achievement {
        id: AchievementId(id),
        // Without text the achievement stays a node in the graph: an empty label beats
        // losing it.
        text: e.attr("text").unwrap_or("").to_string(),
        unlock_condition: e.comment_before.clone(),
        steam_description: e.attr("steam_description").map(str::to_string),
        sprite: SpriteRef::whole(format!("{gfxroot}/{gfx}")),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const ACH: &[u8] = b"<achievements gfxroot=\"gfx/ui/achievement/\">
\t<!-- have 7 or more max red hearts at one time -->
\t<achievement id=\"1\" text='You unlocked \"Magdalene\"' gfx=\"Achievement_Magdalene.png\" />
\t<achievement id=\"2\" text=\"&quot;Card Against Humanity&quot; has appeared in the basement\" gfx=\"Achievement_CAH.png\" />
\t<!-- a --><!-- b -->
\t<achievement id=\"3\" text=\"c\" gfx=\"Achievement_C.png\" />
\t<achievement text=\"no id\" gfx=\"x.png\" />
\t<achievement id=\"5\" text=\"no gfx\" />
</achievements>";

    fn parsed() -> (Vec<Achievement>, Vec<Diagnostic>) {
        let mut d = Vec::new();
        let a = parse(ACH, &mut d);
        (a, d)
    }

    #[test]
    fn text_is_literal_with_either_quote_style_and_entities_resolved() {
        let (a, _) = parsed();
        assert_eq!(a[0].text, "You unlocked \"Magdalene\"");
        assert_eq!(
            a[1].text,
            "\"Card Against Humanity\" has appeared in the basement"
        );
    }

    #[test]
    fn the_comment_right_before_is_the_unlock_condition_and_it_does_not_carry_over() {
        let (a, _) = parsed();
        assert_eq!(
            a[0].unlock_condition.as_deref(),
            Some("have 7 or more max red hearts at one time")
        );
        assert_eq!(
            a[1].unlock_condition, None,
            "the comment applies to a single element only"
        );
        assert_eq!(
            a[2].unlock_condition.as_deref(),
            Some("b"),
            "the last comment before the element wins"
        );
    }

    #[test]
    fn sprite_uses_the_declared_gfxroot() {
        let (a, _) = parsed();
        assert_eq!(
            a[0].sprite.path,
            "gfx/ui/achievement/Achievement_Magdalene.png"
        );
    }

    #[test]
    fn malformed_rows_are_skipped_with_a_reason() {
        let (a, d) = parsed();
        assert_eq!(a.len(), 3);
        assert!(d.contains(&Diagnostic::ElementSkipped {
            source: Source::Achievements,
            id: None,
            reason: SkipReason::MissingId
        }));
        assert!(d.contains(&Diagnostic::ElementSkipped {
            source: Source::Achievements,
            id: Some(5),
            reason: SkipReason::MissingSprite
        }));
    }

    #[test]
    fn the_skips_are_reported_in_file_order_and_nothing_else_is() {
        let (_, d) = parsed();
        assert_eq!(
            d,
            vec![
                Diagnostic::ElementSkipped {
                    source: Source::Achievements,
                    id: None,
                    reason: SkipReason::MissingId
                },
                Diagnostic::ElementSkipped {
                    source: Source::Achievements,
                    id: Some(5),
                    reason: SkipReason::MissingSprite
                },
            ]
        );
    }

    #[test]
    fn a_malformed_id_is_skipped_without_an_id() {
        let mut d = Vec::new();
        let a = parse(
            b"<achievements><achievement id=\"x\" gfx=\"a.png\" /></achievements>",
            &mut d,
        );
        assert!(a.is_empty());
        assert_eq!(
            d,
            vec![Diagnostic::ElementSkipped {
                source: Source::Achievements,
                id: None,
                reason: SkipReason::MalformedId
            }]
        );
    }

    #[test]
    fn without_a_gfxroot_the_sprite_takes_the_game_folder_and_no_text_is_an_empty_label() {
        let mut d = Vec::new();
        let a = parse(
            b"<achievements><achievement id=\"1\" gfx=\"a.png\" /></achievements>",
            &mut d,
        );
        assert_eq!(a[0].sprite.path, "gfx/ui/achievement/a.png");
        assert_eq!(a[0].text, "");
        assert!(d.is_empty());
    }

    #[test]
    fn junk_is_empty_with_one_diagnostic() {
        let mut d = Vec::new();
        assert!(parse(b"<achievements><achievement", &mut d).is_empty());
        assert_eq!(
            d,
            vec![Diagnostic::SourceUnreadable {
                source: Source::Achievements
            }]
        );
    }

    #[test]
    fn steam_description_is_kept_and_absent_when_the_attribute_is_missing() {
        let ach = b"<achievements gfxroot=\"gfx/ui/achievement/\">
\t<achievement id=\"517\" text=\"x\" gfx=\"a.png\" steam_name=\"Dirty Mind\" steam_description=\"Complete Challenge 36.\" />
\t<achievement id=\"1\" text=\"y\" gfx=\"b.png\" />
</achievements>";
        let mut d = Vec::new();
        let a = parse(ach, &mut d);
        assert_eq!(
            a[0].steam_description.as_deref(),
            Some("Complete Challenge 36.")
        );
        assert_eq!(a[1].steam_description, None);
    }
}
