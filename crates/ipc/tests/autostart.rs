//! Starting with Windows: the decision a login launch makes, and the shape of what crosses.
//!
//! The registry, the plugin and the window are wiring and live in `app`. What is here is the
//! part worth checking: given the arguments the process was handed, does this launch show a
//! window or stay in the tray.

use ipc::{
    launch_intent, AutostartFailure, AutostartReason, AutostartView, IpcError, LaunchIntent,
    SILENT_ARG,
};

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

#[test]
fn a_launch_with_no_arguments_opens_a_window() {
    // Somebody double-clicked the icon. The caller drops `argv[0]`, so this function sees what
    // follows — and what follows is nothing.
    assert_eq!(launch_intent(&[]), LaunchIntent::Window);
}

#[test]
fn the_argument_the_login_entry_carries_stays_in_the_tray() {
    assert_eq!(launch_intent(&args(&[SILENT_ARG])), LaunchIntent::Silent);
}

#[test]
fn an_argument_we_do_not_know_opens_a_window_even_beside_the_one_we_do() {
    // The only way to see this app is a window, and a launch that shows nothing because of a
    // typo is a launch that looks like a crash. So an unknown argument wins, on either side of
    // the one we know.
    assert_eq!(launch_intent(&args(&["--wat"])), LaunchIntent::Window);
    assert_eq!(
        launch_intent(&args(&["--wat", SILENT_ARG])),
        LaunchIntent::Window
    );
    assert_eq!(
        launch_intent(&args(&[SILENT_ARG, "--wat"])),
        LaunchIntent::Window
    );
}

#[test]
fn the_view_is_camel_case_and_the_reason_is_a_bare_string() {
    // `unavailable` is `null` when the switch can be offered, and the reason itself carries no
    // data — so it is a value on the wire and not a tagged object. `CLAUDE.md` admits no
    // exception, and the spec's §3 asked for a tag by analogy with two enums that have one
    // because a variant of each carries a field.
    let offered = serde_json::to_value(AutostartView {
        enabled: true,
        unavailable: None,
    })
    .unwrap();
    assert_eq!(
        offered,
        serde_json::json!({ "enabled": true, "unavailable": null })
    );

    let refused = serde_json::to_value(AutostartView {
        enabled: false,
        unavailable: Some(AutostartReason::NotSupported),
    })
    .unwrap();
    assert_eq!(
        refused,
        serde_json::json!({ "enabled": false, "unavailable": "notSupported" })
    );
    assert_eq!(
        serde_json::to_value(AutostartReason::RegistryUnreadable).unwrap(),
        serde_json::json!("registryUnreadable")
    );
}

#[test]
fn a_refused_write_and_an_ignored_one_are_two_answers() {
    // Not two wordings of one failure: two different things for the user to do. A refused
    // write is a policy or an antivirus standing in the way; an ignored one is the value
    // going in and Windows still saying no — `is_enabled()` is `value && approved`, so an
    // entry switched off in Task Manager's Startup tab reads as off however well it was
    // written, and there is nothing this app can do about it from here.
    assert_eq!(
        serde_json::to_value(AutostartFailure::WriteRefused).unwrap(),
        serde_json::json!("writeRefused")
    );
    assert_eq!(
        serde_json::to_value(AutostartFailure::WriteIgnored).unwrap(),
        serde_json::json!("writeIgnored")
    );
    assert_eq!(
        serde_json::to_value(IpcError::AutostartNotWritable {
            reason: AutostartFailure::WriteIgnored
        })
        .unwrap(),
        serde_json::json!({ "kind": "autostartNotWritable", "reason": "writeIgnored" })
    );
}
