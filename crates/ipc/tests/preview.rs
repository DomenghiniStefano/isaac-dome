//! The welcome's three counts. Slot 0 is neither an achievement nor an item — the same
//! `.skip(1)` the Unlock and Collection views already count by — and a section that was not
//! read reports `Unread`, never `Read { done: 0 }`: a zero is a profile at the start.

use core_save::{Kind, Save, Section};
use ipc::{preview_of, CandidatePreview, PreviewCount};
use serde_json::json;

fn section(kind: Kind, bytes: Vec<u8>) -> Section {
    let per = kind.bytes_per_entry().unwrap_or(1);
    Section {
        kind,
        count: (bytes.len() / per) as u32,
        declared_size: 0,
        offset: 0,
        bytes,
    }
}

fn save(sections: Vec<Section>) -> Save {
    Save {
        unknown_0x10: 0,
        sections,
        diagnostics: Vec::new(),
    }
}

/// Four flags: slot 0 (which is no achievement) plus three achievements, two of them done.
fn three_achievements() -> Section {
    section(Kind::Achievements, vec![1, 1, 0, 1])
}

#[test]
fn slot_zero_is_in_neither_the_numerator_nor_the_denominator() {
    let p = preview_of(&save(vec![three_achievements()]));
    assert_eq!(p.achievements, PreviewCount::Read { done: 2, of: 3 });
}

#[test]
fn a_section_that_was_not_read_says_so_rather_than_counting_zero() {
    let p = preview_of(&save(vec![three_achievements()]));
    assert_eq!(p.items, PreviewCount::Unread, "no section 4 in this save");
    assert_eq!(p.marks, PreviewCount::Unread, "no section 2 in this save");
    assert_eq!(
        p.unreadable_cells, 0,
        "no counters section is not 408 unreadable cells: it is nothing to say"
    );
}

#[test]
fn an_empty_counters_section_is_read_and_every_cell_is_unreadable() {
    let p = preview_of(&save(vec![section(Kind::Counters, Vec::new())]));
    assert_eq!(p.marks, PreviewCount::Read { done: 0, of: 0 });
    assert_eq!(p.unreadable_cells, 34 * 12);
}

/// The wire shape. Silent in TypeScript when it is wrong, so it is pinned here.
#[test]
fn the_json_is_camel_case_and_the_count_is_tagged() {
    let p = CandidatePreview {
        achievements: PreviewCount::Read { done: 379, of: 640 },
        items: PreviewCount::Unread,
        marks: PreviewCount::Read { done: 92, of: 120 },
        unreadable_cells: 8,
    };
    assert_eq!(
        serde_json::to_value(p).unwrap(),
        json!({
            "achievements": { "kind": "read", "done": 379, "of": 640 },
            "items": { "kind": "unread" },
            "marks": { "kind": "read", "done": 92, "of": 120 },
            "unreadableCells": 8
        })
    );
}
