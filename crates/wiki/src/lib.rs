//! wiki — the wiki's text sections as a typed tree. A pure crate except for one spot:
//! `raw` reads `dataset/raw/` from disk. No network, no game archive.

mod blocks;
mod build;
mod dataset;
mod diagnostics;
mod infobox;
mod inline;
mod model;
mod page;
mod raw;
mod resolver;
mod sections;
mod template;

pub use blocks::parse_blocks;
pub use build::build;
pub use dataset::{Counts, Dataset, DatasetError, Meta, Patch, Source, SCHEMA_VERSION};
pub use diagnostics::Diagnostics;
pub use infobox::{extract_infoboxes, infobox_from, InfoboxKind, RawInfobox};
pub use inline::parse_inline;
pub use model::{
    Block, Dlc, Entry, Infobox, Inline, ListItem, Section, SectionKind, Style, Target,
};
pub use page::{parse_page, EntryKey, PageKind};
pub use raw::{page_file_name, IndexEntry, Raw, RawError, RawPage};
pub use resolver::{
    in_current_edition, is_layout_template, key, Corrections, Resolution, Resolver, Row, Tables,
    CORRECTED_TABLES, DLC_REPENTANCE_PLUS,
};
pub use sections::{section_kind, split_page, RawSection};
pub use template::{parse_template_at, Template};
