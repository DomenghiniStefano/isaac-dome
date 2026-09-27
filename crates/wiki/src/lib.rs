//! wiki — the wiki's text sections as a typed tree. A pure crate except two spots: `raw`
//! reads `dataset/raw/` from disk, and `Dataset::read_dir`/`write_dir` do the same for
//! `dataset/wiki/`. No network, no game archive.

mod blocks;
mod build;
mod dataset;
mod dead_links;
mod diagnostics;
mod editions;
#[cfg(feature = "test-api")]
pub mod for_tests;
mod infobox;
mod inline;
mod inline_tree;
mod model;
mod page;
#[cfg(feature = "test-api")]
mod parent_check;
mod raw;
mod resolver;
mod sections;
mod template;
mod title;
mod transformation;
mod unlock_condition;

pub use blocks::parse_blocks;
pub use build::build;
#[cfg(all(feature = "embedded", feature = "test-api"))]
pub use dataset::embedded_len;
pub use dataset::{
    write_if_changed, Counts, Dataset, DatasetError, Meta, Patch, Source, HOST, SCHEMA_VERSION,
};
pub use dead_links::{dead_links, DeadLinks};
pub use diagnostics::Diagnostics;
pub use editions::{parse_code, Editions};
#[cfg(feature = "test-api")]
pub use infobox::IGNORED_PARAMS;
pub use infobox::{
    entry_facts, extract_infoboxes, infobox_from, EntryFacts, InfoboxKind, RawInfobox,
};
pub use inline::{parse_inline, plain};
pub use model::{
    Block, CollectibleTemplate, Dlc, Entry, Infobox, Inline, ListItem, Section, SectionKind, Style,
    Target,
};
pub use page::{parse_page, EntryKey, PageKind};
pub use raw::{
    page_file_name, ArticleCategory, IndexEntry, Raw, RawError, RawPage, CONTENT_TEMPLATES,
};
pub use resolver::{
    in_current_edition, is_layout_template, key, Corrections, Excluded, Resolution, Resolver, Row,
    Tables, CORRECTED_TABLES,
};
pub use sections::{is_excluded_section, normalize_title, section_kind, split_page, RawSection};
pub use template::{parse_template_at, Template};
pub use title::canonical_title;
