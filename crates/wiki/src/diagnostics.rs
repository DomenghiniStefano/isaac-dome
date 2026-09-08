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
        self.pages_without_id += other.pages_without_id;
        self.orphan_closers += other.orphan_closers;
    }
}
