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
}

impl Diagnostics {
    pub fn unresolved(&mut self, template: &str) {
        *self.unresolved.entry(template.to_string()).or_default() += 1;
    }

    pub fn unknown_template(&mut self, name: &str) {
        *self.unknown_templates.entry(name.to_string()).or_default() += 1;
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
    pub fn merge(&mut self, other: &Diagnostics) {
        for (k, v) in &other.unresolved {
            *self.unresolved.entry(k.clone()).or_default() += v;
        }
        for (k, v) in &other.unknown_templates {
            *self.unknown_templates.entry(k.clone()).or_default() += v;
        }
        for (k, v) in &other.discarded_sections {
            *self.discarded_sections.entry(k.clone()).or_default() += v;
        }
        for (k, v) in &other.unknown_dlc_codes {
            *self.unknown_dlc_codes.entry(k.clone()).or_default() += v;
        }
        for (k, v) in &other.unknown_entities {
            *self.unknown_entities.entry(k.clone()).or_default() += v;
        }
        self.pages_without_id += other.pages_without_id;
        self.orphan_closers += other.orphan_closers;
        self.transformation_sources_disagree += other.transformation_sources_disagree;
    }
}
