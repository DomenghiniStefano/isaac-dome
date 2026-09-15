//! Parser counters: they end up in the dataset's `meta`, so a rebuild tells whether the
//! parser has lost ground. Never an error: everything degrades and is counted.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostics {
    /// Unresolved `Ref`s, by template name (`i`, `c`, `e`, …).
    pub unresolved: BTreeMap<String, u32>,
    /// Templates listed neither among the links nor among the layout ones, by name.
    pub unknown_templates: BTreeMap<String, u32>,
    /// Discarded sections, by original title.
    pub discarded_sections: BTreeMap<String, u32>,
    /// Pages with an infobox but no usable `id`.
    pub pages_without_id: u32,
    /// Lines made of nothing but `}}`: they close a template that opened on an earlier
    /// line, which the line-by-line pass never saw as a single template. The line is
    /// dropped so it doesn't cut the list around it in two; the wrapper it closed stays
    /// unrepresented, and this counts how often that happens.
    ///
    /// `default` because a dataset built by an earlier parser simply doesn't carry the
    /// key, and refusing to load a whole snapshot over a missing diagnostic counter is
    /// the opposite of degrading. Any counter added here later wants the same.
    #[serde(default)]
    pub orphan_closers: u32,
    /// `dlc` codes the parser does not recognize, by code. The parameter concatenates them
    /// without a separator (`a+nr`), so a wiki-side typo or a new edition would otherwise
    /// leave an entry with fewer editions than it declares, and nothing would say so.
    ///
    /// `default` for the same reason as `orphan_closers`.
    #[serde(default)]
    pub unknown_dlc_codes: BTreeMap<String, u32>,
    /// Transformation pages whose two statements of the item set do not agree: the
    /// infobox's `items` and the body's own tables. Neither is dropped — the set is their
    /// union — and this counts how often the wiki contradicts itself, which on the live
    /// snapshot it does (Guppy's infobox omits the trinket its body lists).
    ///
    /// A counter and not an error: it is the wiki's inconsistency, not ours, and a
    /// disagreement that never happened would mean the cross-check has gone silent.
    ///
    /// `default` for the same reason as `orphan_closers`.
    #[serde(default)]
    pub transformation_sources_disagree: u32,
    /// HTML entities the inline parser's closed list does not cover, by name. The text is
    /// **kept** rather than dropped — it is the wiki's content, and `&` is an ordinary
    /// character in it — so this is the only thing that makes a new entity visible instead
    /// of shipped. Three of them reached the screen as `&comma;` until 2026-09-14, in
    /// pickup quotes, because nothing counted them.
    ///
    /// `default` for the same reason as `orphan_closers`.
    #[serde(default)]
    pub unknown_entities: BTreeMap<String, u32>,
    /// `{{infobox …}}` templates whose name is in no kind, by name. The page is extracted,
    /// the infobox is skipped, and until 2026-09-14 that skip was silent — which is how a
    /// page could enter the snapshot and produce **zero** entries with nothing to show for
    /// it (B45: four character pages state two characters with `infobox characters`, and
    /// the whole shape was invisible from inside the repo).
    ///
    /// A page we never download cannot be counted here, so this does not replace the
    /// snapshot's own coverage check. It answers the other half: what we downloaded and
    /// did not understand.
    ///
    /// `default` for the same reason as `orphan_closers`.
    #[serde(default)]
    pub unknown_infoboxes: BTreeMap<String, u32>,
    /// `{{dlc|…}}` spans whose editions and their page's have none in common. The wiki
    /// draws its own error there ("Incompatible DLC with current context"), so this counts
    /// a contradiction on the wiki's side, not a gap on ours: the words are kept and the
    /// badge is dropped, because a span valid in no edition is worse than the same span
    /// unqualified.
    ///
    /// **0 on the snapshot of 2026-09-15**, and the one span that would have landed here is
    /// worth knowing about: Lilith's `{{without context|{{dlc|na}}}}` — the wiki's own
    /// wrapper for "ignore the page context here" — which sits under `== Trivia ==` and is
    /// discarded before it is ever parsed. If this counter ever moves, that template is the
    /// first thing to look for.
    ///
    /// `default` for the same reason as `orphan_closers`.
    #[serde(default)]
    pub spans_outside_their_page: u32,
}

impl Diagnostics {
    pub fn unresolved(&mut self, template: &str) {
        *self.unresolved.entry(template.to_string()).or_default() += 1;
    }

    pub fn unknown_template(&mut self, name: &str) {
        *self.unknown_templates.entry(name.to_string()).or_default() += 1;
    }

    pub fn unknown_infobox(&mut self, name: &str) {
        *self.unknown_infoboxes.entry(name.to_string()).or_default() += 1;
    }

    pub fn orphan_closer(&mut self) {
        self.orphan_closers += 1;
    }

    pub fn unknown_dlc_code(&mut self, code: &str) {
        *self.unknown_dlc_codes.entry(code.to_string()).or_default() += 1;
    }

    pub fn unknown_entity(&mut self, name: &str) {
        *self.unknown_entities.entry(name.to_string()).or_default() += 1;
    }

    pub fn discarded_section(&mut self, title: &str) {
        *self
            .discarded_sections
            .entry(title.to_string())
            .or_default() += 1;
    }

    /// Adds the counters of another pass (a page) into this one (the snapshot).
    ///
    /// `other` is **destructured field by field**, with no `..`, so a counter added to the
    /// struct and not to this function breaks the build. It is not a style choice:
    /// `unknown_infoboxes` was added on 2026-09-14 and left out here, and the dataset it
    /// exists to fill shipped `{}` for it with the suite green.
    pub fn merge(&mut self, other: &Diagnostics) {
        let Diagnostics {
            unresolved,
            unknown_templates,
            discarded_sections,
            pages_without_id,
            orphan_closers,
            unknown_dlc_codes,
            transformation_sources_disagree,
            unknown_entities,
            unknown_infoboxes,
            spans_outside_their_page,
        } = other;
        for (k, v) in unresolved {
            *self.unresolved.entry(k.clone()).or_default() += v;
        }
        for (k, v) in unknown_templates {
            *self.unknown_templates.entry(k.clone()).or_default() += v;
        }
        for (k, v) in discarded_sections {
            *self.discarded_sections.entry(k.clone()).or_default() += v;
        }
        for (k, v) in unknown_dlc_codes {
            *self.unknown_dlc_codes.entry(k.clone()).or_default() += v;
        }
        for (k, v) in unknown_entities {
            *self.unknown_entities.entry(k.clone()).or_default() += v;
        }
        for (k, v) in unknown_infoboxes {
            *self.unknown_infoboxes.entry(k.clone()).or_default() += v;
        }
        self.pages_without_id += pages_without_id;
        self.orphan_closers += orphan_closers;
        self.transformation_sources_disagree += transformation_sources_disagree;
        self.spans_outside_their_page += spans_outside_their_page;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `build` counts a page into a fresh `Diagnostics` and merges it into the snapshot's,
    /// so a counter this function forgets is a counter that reads **zero** in the shipped
    /// dataset however often it fires. `unknown_infoboxes` was exactly that: added on
    /// 2026-09-14 so a page whose infobox has no kind would be loud, and shipping `{}` ever
    /// since, because the merge never mentioned it.
    ///
    /// The test is written on a value with every field set, so a field added later and not
    /// merged fails here rather than in a reading of `meta` nobody does.
    #[test]
    fn a_merge_carries_every_counter_it_has() {
        let mut page = Diagnostics::default();
        page.unresolved("i");
        page.unknown_template("m");
        page.discarded_section("Trivia");
        page.unknown_dlc_code("zz");
        page.unknown_entity("frac");
        page.unknown_infobox("infobox nothing");
        page.pages_without_id = 1;
        page.orphan_closers = 1;
        page.transformation_sources_disagree = 1;
        page.spans_outside_their_page = 1;

        let mut all = Diagnostics::default();
        all.merge(&page);
        all.merge(&page);

        assert_eq!(all.unresolved.get("i"), Some(&2));
        assert_eq!(all.unknown_templates.get("m"), Some(&2));
        assert_eq!(all.discarded_sections.get("Trivia"), Some(&2));
        assert_eq!(all.unknown_dlc_codes.get("zz"), Some(&2));
        assert_eq!(all.unknown_entities.get("frac"), Some(&2));
        assert_eq!(all.unknown_infoboxes.get("infobox nothing"), Some(&2));
        assert_eq!(all.pages_without_id, 2);
        assert_eq!(all.orphan_closers, 2);
        assert_eq!(all.transformation_sources_disagree, 2);
        assert_eq!(all.spans_outside_their_page, 2);
    }
}
