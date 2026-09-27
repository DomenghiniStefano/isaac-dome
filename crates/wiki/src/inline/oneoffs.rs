//! Template arms with no family of their own: a curse or a status linked to the wiki's shared
//! concept page for it, a game-mode qualifier kept as text (every positional argument, not
//! only the first), and a stat-change note with no icon to draw here. Each used to fall to the
//! unknown-template fallback — Decision 10 gives it a real arm instead.

use crate::resolver::Resolver;
use crate::template::Template;
use crate::{Diagnostics, Inline};

use super::{recurse_into_arg, Out};

/// `{{cu|Darkness}}` and `{{curse|darkness}}`, two spellings of the same template (57 and 25
/// occurrences): the curse's own name, linked to the wiki's single "Curses" page — the
/// corpus's own wikilinks (`[[curses]]`, `[[curses|curse]]`) never point at a page for one
/// curse alone, so this doesn't invent one either.
pub(super) fn curse(arg: &str, out: &mut Out) {
    out.push(Inline::Concept {
        page: "Curses".to_string(),
        label: arg.trim().to_string(),
    });
}

/// `{{blindfolded}}`: a status with its own page, named on Lilith's infobox and inline on a
/// monster's page ("does not fire when Blindfolded"). It takes no argument at all, so it
/// cannot go through `resolve`, which needs one to build a label from.
pub(super) fn blindfolded(out: &mut Out) {
    out.push(Inline::Concept {
        page: "Blindfolded".to_string(),
        label: "Blindfolded".to_string(),
    });
}

/// `{{mode|greed|greedier}}`: every positional argument, joined, not only the first. The
/// unknown-template fallback kept `greed` and dropped `greedier` in silence, on Mom's Heart's
/// own page — the same shape the fallback loses a second positional argument everywhere.
pub(super) fn mode(t: &Template, r: &Resolver, d: &mut Diagnostics, out: &mut Out, depth: u32) {
    for (i, a) in t.args.iter().enumerate() {
        if i > 0 {
            out.buf.push('/');
        }
        recurse_into_arg(a, r, d, out, depth);
    }
}

/// `{{tear delay down|2}}`: a stat-change note with no icon to draw here, so it reads as one.
/// `no bullet=yes`, the only named parameter the corpus uses, only tells the site whether to
/// draw its own bullet in front of the note — nothing to keep once the note already sits
/// inside a list item.
pub(super) fn tear_delay_down(arg: &str, out: &mut Out) {
    out.buf.push_str(&format!("-{} Tear Delay", arg.trim()));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inline::parse_inline;
    use crate::resolver::fixtures::test_resolver;
    use crate::Style;

    fn p(s: &str) -> (Vec<Inline>, Diagnostics) {
        let r = test_resolver();
        let mut d = Diagnostics::default();
        (parse_inline(s, &r, &mut d), d)
    }

    fn text(t: &str) -> Inline {
        Inline::Text {
            text: t.into(),
            style: Style::Plain,
        }
    }

    /// Mom's Heart: `{{cu|Darkness}}` inside a sentence.
    #[test]
    fn cu_and_curse_link_to_the_shared_curses_page() {
        let (v, d) = p("the {{cu|Darkness}} effect and {{Curse|maze}}");
        assert!(v.iter().any(
            |i| matches!(i, Inline::Concept { page, label } if page == "Curses" && label == "Darkness")
        ));
        assert!(v.iter().any(
            |i| matches!(i, Inline::Concept { page, label } if page == "Curses" && label == "maze")
        ));
        assert!(d.unknown_templates.is_empty(), "{:?}", d.unknown_templates);
    }

    #[test]
    fn blindfolded_names_its_own_concept_with_no_argument() {
        let (v, d) = p("Large Zit does not fire when {{Blindfolded}}");
        assert!(v.iter().any(
            |i| matches!(i, Inline::Concept { page, label } if page == "Blindfolded" && label == "Blindfolded")
        ));
        assert!(d.unknown_templates.is_empty(), "{:?}", d.unknown_templates);
    }

    /// Mom's Heart: `{{mode|greed|greedier}}` — both words, not only the first. Plain text
    /// merges around a template's output the same way it does around any other, so the whole
    /// sentence comes back as one node.
    #[test]
    fn mode_keeps_every_positional_argument() {
        let (v, d) = p("in {{mode|greed|greedier}} Greed Mode");
        assert_eq!(v, vec![text("in greed/greedier Greed Mode")]);
        assert!(d.unknown_templates.is_empty(), "{:?}", d.unknown_templates);
    }

    #[test]
    fn mode_with_one_argument_has_no_separator() {
        let (v, _) = p("{{mode|normal}}");
        assert_eq!(v, vec![text("normal")]);
    }

    #[test]
    fn tear_delay_down_reads_as_a_stat_change() {
        let (v, d) = p("{{tear delay down|2}}");
        assert_eq!(v, vec![text("-2 Tear Delay")]);
        assert!(d.unknown_templates.is_empty(), "{:?}", d.unknown_templates);

        // Cancer's own markup: `no bullet=yes` carries nothing to keep.
        let (v, _) = p("{{tear delay down|2|no bullet=yes}}");
        assert_eq!(v, vec![text("-2 Tear Delay")]);
    }
}
