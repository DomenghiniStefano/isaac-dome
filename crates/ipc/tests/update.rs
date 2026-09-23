//! Updating the app: the phases, what may follow what, and the shape of what crosses.
//!
//! The plugin, the network and the installer are wiring and live in `app`. What is here is
//! the part worth checking: a state machine that a window draws, and a progress reading that
//! decides how often every window is told to read again.
//!
//! Spec: `docs/superpowers/specs/2026-09-20-app-update-design.md`.

use ipc::for_tests::update_at;
use ipc::{UpdateFailure, UpdatePhase, UpdateReason, UpdateState};
use wiki::{Block, Inline, Style};

const VERSION: &str = "0.1.0";

fn downloading(state: &UpdateState) -> Option<u8> {
    match state.phase() {
        UpdatePhase::Downloading { percent, .. } => *percent,
        other => panic!("expected a download in progress, found {other:?}"),
    }
}

#[test]
fn a_launch_has_asked_nothing_yet() {
    // Not "up to date": nobody has looked. The screen has to be able to say which of the two
    // it is, because "no update" and "no check" are different sentences.
    assert_eq!(UpdateState::default().phase(), &UpdatePhase::Idle);
}

#[test]
fn a_check_while_one_is_running_is_not_a_second_check() {
    let mut state = UpdateState::default();
    assert!(state.begin_check());
    assert!(!state.begin_check());
    assert_eq!(state.phase(), &UpdatePhase::Checking);
}

#[test]
fn a_check_while_downloading_leaves_the_download_alone() {
    // The button is disabled during a download, so this is a second window or a race. Either
    // way the bytes on their way in are worth more than a fresh question.
    let mut state = UpdateState::default();
    state.begin_check();
    state.begin_download("0.2.0".into());
    state.advance(50, Some(100));

    assert!(!state.begin_check());
    assert_eq!(downloading(&state), Some(50));
}

#[test]
fn a_check_after_an_answer_is_allowed() {
    // Three resting phases, and from each of them the button has to work: the app sits in the
    // tray for days and "up to date" goes stale.
    for phase in [
        UpdatePhase::UpToDate,
        UpdatePhase::Ready {
            version: "0.2.0".into(),
            notes: vec![],
        },
        UpdatePhase::Failed {
            reason: UpdateFailure::Offline,
        },
    ] {
        let mut state = update_at(phase.clone());
        assert!(state.begin_check(), "a check was refused from {phase:?}");
    }
}

#[test]
fn the_percentage_only_moves_on_a_whole_point() {
    // Every window is told to read again on a `true`, so this decides whether a download is
    // a hundred events or a hundred thousand. The bar cannot show a fraction anyway.
    let mut state = UpdateState::default();
    state.begin_download("0.2.0".into());

    assert!(state.advance(1, Some(1000)), "0% → 0.1% is a first reading");
    assert_eq!(downloading(&state), Some(0));
    assert!(!state.advance(8, Some(1000)), "0.9% is still 0%");
    assert!(state.advance(1, Some(1000)), "1% is a new point");
    assert_eq!(downloading(&state), Some(1));
}

#[test]
fn a_download_with_no_length_announced_has_no_percentage() {
    // No `Content-Length`. An indeterminate bar is the truth; a bar inventing a number is not,
    // and nothing is gained by telling the windows about bytes they cannot draw.
    let mut state = UpdateState::default();
    state.begin_download("0.2.0".into());

    assert!(!state.advance(4096, None));
    assert_eq!(downloading(&state), None);
}

#[test]
fn the_percentage_never_passes_a_hundred() {
    // The length is the server's claim, not a measurement: a body longer than announced must
    // read as finished, never as 104%.
    let mut state = UpdateState::default();
    state.begin_download("0.2.0".into());
    state.advance(130, Some(100));

    assert_eq!(downloading(&state), Some(100));
}

#[test]
fn a_chunk_arriving_after_the_end_changes_nothing() {
    // The download failed, or it finished, and a callback is still in flight. A late chunk
    // must not put the bar back on a screen that has moved on.
    for phase in [
        UpdatePhase::Failed {
            reason: UpdateFailure::Offline,
        },
        UpdatePhase::Ready {
            version: "0.2.0".into(),
            notes: vec![],
        },
        UpdatePhase::Idle,
    ] {
        let mut state = update_at(phase.clone());
        assert!(!state.advance(10, Some(100)));
        assert_eq!(state.phase(), &phase);
    }
}

#[test]
fn a_second_download_counts_from_zero() {
    // A retry after a failure reuses the state. Bytes left over from the attempt before would
    // make the bar start halfway and reach the end early.
    let mut state = UpdateState::default();
    state.begin_download("0.2.0".into());
    state.advance(50, Some(100));
    state.fail(UpdateFailure::Offline);

    state.begin_download("0.2.0".into());
    assert_eq!(downloading(&state), None);
    state.advance(10, Some(100));
    assert_eq!(downloading(&state), Some(10));
}

#[test]
fn the_version_is_answered_with_no_network_at_all() {
    // Whatever happened out there, the screen can always say which build this is.
    let view = UpdateState::default().view(VERSION, None);
    assert_eq!(view.current_version, VERSION);
    assert_eq!(view.phase, UpdatePhase::Idle);
    assert_eq!(view.unavailable, None);
}

#[test]
fn a_development_build_says_so_rather_than_saying_off() {
    // The same shape `AutostartView` uses, for the same reason: "off" and "impossible" are two
    // different answers, and a boolean would give them one.
    let view = UpdateState::default().view(VERSION, Some(UpdateReason::NotSupported));
    assert_eq!(view.unavailable, Some(UpdateReason::NotSupported));
}

#[test]
fn the_view_is_camel_case_and_the_reason_is_a_bare_string() {
    let view = UpdateState::default().view(VERSION, Some(UpdateReason::NotSupported));
    assert_eq!(
        serde_json::to_value(view).unwrap(),
        serde_json::json!({
            "currentVersion": "0.1.0",
            "phase": { "kind": "idle" },
            "unavailable": "notSupported",
        })
    );
}

#[test]
fn every_phase_is_tagged_and_its_fields_are_camel_case() {
    // What this pins is the tag and the variant names. **It does not yet pin
    // `rename_all_fields`**, and saying so is the point: every field inside these variants is
    // one word, so removing that attribute changes no byte of this JSON — checked by removing
    // it, on 2026-09-20, and all fourteen tests stayed green. The attribute is on the enum for
    // the day a two-word field lands, and on that day this test starts earning its comment.
    // Until then the camelCase claim that is real is `currentVersion`, below.
    let shapes = [
        (UpdatePhase::Idle, serde_json::json!({ "kind": "idle" })),
        (
            UpdatePhase::Checking,
            serde_json::json!({ "kind": "checking" }),
        ),
        (
            UpdatePhase::UpToDate,
            serde_json::json!({ "kind": "upToDate" }),
        ),
        (
            UpdatePhase::Downloading {
                version: "0.2.0".into(),
                percent: Some(42),
            },
            serde_json::json!({ "kind": "downloading", "version": "0.2.0", "percent": 42 }),
        ),
        (
            UpdatePhase::Ready {
                version: "0.2.0".into(),
                notes: vec![Block::Paragraph {
                    inline: vec![Inline::Text {
                        text: "what changed".into(),
                        style: Style::Plain,
                    }],
                }],
            },
            serde_json::json!({
                "kind": "ready",
                "version": "0.2.0",
                "notes": [{ "kind": "paragraph", "inline": [{ "kind": "text", "text": "what changed", "style": "plain" }] }],
            }),
        ),
        (
            UpdatePhase::Failed {
                reason: UpdateFailure::Rejected,
            },
            serde_json::json!({ "kind": "failed", "reason": "rejected" }),
        ),
    ];
    for (phase, expected) in shapes {
        assert_eq!(serde_json::to_value(&phase).unwrap(), expected, "{phase:?}");
    }
}

#[test]
fn every_failure_is_a_bare_string() {
    // Fieldless, so a value on the wire and not a tagged object — the repo's rule with zero
    // exceptions. A variant that ever carries data takes the whole enum with it.
    let names = [
        (UpdateFailure::Offline, "offline"),
        (UpdateFailure::NotPublished, "notPublished"),
        (UpdateFailure::Rejected, "rejected"),
        (UpdateFailure::InstallFailed, "installFailed"),
        (UpdateFailure::Unknown, "unknown"),
    ];
    for (failure, name) in names {
        assert_eq!(
            serde_json::to_value(failure).unwrap(),
            serde_json::json!(name)
        );
    }
    assert_eq!(
        serde_json::to_value(UpdateReason::NotSupported).unwrap(),
        serde_json::json!("notSupported")
    );
}

#[test]
fn asking_to_install_nothing_is_the_one_error_that_crosses() {
    // Everything a user can run into — no network, no release, a signature that did not
    // verify — is a phase in the payload. This is not one of those: the button only exists in
    // `Ready`, so a window asking from anywhere else is a defect, and a defect is an `Err`.
    assert_eq!(
        serde_json::to_value(ipc::IpcError::UpdateNotReady).unwrap(),
        serde_json::json!({ "kind": "updateNotReady" })
    );
}

/// The notes cross as blocks, never as the markdown they arrived in: the manifest is not
/// signed, so its text reaches the window as words and never as markup. No notes at all and
/// notes that are only whitespace are the same thing to draw.
#[test]
fn the_notes_are_read_into_blocks_when_the_download_is_ready() {
    let mut state = UpdateState::default();
    state.ready(
        "0.2.0".into(),
        Some("### What changed\n\n- **One.**".into()),
    );
    let UpdatePhase::Ready { notes, .. } = state.phase() else {
        panic!("expected ready, found {:?}", state.phase());
    };
    assert_eq!(notes, &ipc::release_notes("### What changed\n\n- **One.**"));
    assert!(matches!(
        notes.first(),
        Some(Block::Heading { level: 3, .. })
    ));

    for none in [None, Some("  \n".to_string())] {
        let mut state = UpdateState::default();
        state.ready("0.2.0".into(), none);
        assert!(matches!(
            state.phase(),
            UpdatePhase::Ready { notes, .. } if notes.is_empty()
        ));
    }
}
