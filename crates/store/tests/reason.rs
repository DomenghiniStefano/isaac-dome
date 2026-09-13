//! N2: the reason a `StoreError` carries crosses the IPC, so it travels as a variant and
//! not as a sentence. SQLite's own message stops here — it can contain the file path, and
//! a path carries the Windows username.

use store::StoreError;

#[test]
fn a_store_error_maps_to_the_typed_reason_that_names_it() {
    assert_eq!(
        ipc::StoreReason::from(&StoreError::Unreadable {
            reason: "no such table: plan_queue".to_string(),
        }),
        ipc::StoreReason::Unreadable
    );
    assert_eq!(
        ipc::StoreReason::from(&StoreError::NewerSchema {
            found: 7,
            supported: 1,
        }),
        ipc::StoreReason::NewerSchema {
            found: 7,
            supported: 1,
        }
    );
}

/// The property, not the value: whatever SQLite wrote, none of it survives the mapping.
#[test]
fn sqlites_own_message_never_reaches_the_typed_reason() {
    let noisy = StoreError::Unreadable {
        reason: r"unable to open database file C:\Users\someone\AppData\isaacdome.db".to_string(),
    };
    let json = serde_json::to_value(ipc::StoreReason::from(&noisy)).expect("it serializes");
    let text = json.to_string();
    assert!(!text.contains("someone"), "{text}");
    assert!(!text.contains("AppData"), "{text}");
}
