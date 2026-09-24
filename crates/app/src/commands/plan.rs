//! The goals: the plan's older half, still its own document in the database.

use tauri::AppHandle;

use ipc::IpcError;
use store::{plan_parts, store_error, store_unavailable};

use crate::events::{announce, PLAN_CHANGED};
use crate::icons::icon_url;

use crate::state::{CatalogState, ResourcesState, StoreState};

#[tauri::command]
pub fn plan(
    app: AppHandle,
    store: tauri::State<'_, StoreState>,
    catalog: tauri::State<'_, CatalogState>,
    resources: tauri::State<'_, ResourcesState>,
) -> Result<ipc::PlanView, IpcError> {
    // The catalog is needed to resolve saved keys into names and icons: without it,
    // the goals are still visible and the view says why they have no name.
    let resources = resources.get();
    let c = resources.and_then(|rs| catalog.get_or_build(rs));
    // Database that won't open, or a query that fails: expected cases, the plan comes
    // out empty and says why.
    let (goals, unreadable, unavailable) = match store.lock(&app) {
        Ok(guard) => plan_parts(guard.goals()),
        Err(reason) => (Vec::new(), Vec::new(), Some(reason)),
    };
    Ok(ipc::plan_view(
        c,
        wiki::Dataset::embedded().ok(),
        goals,
        unreadable,
        unavailable,
        icon_url,
    ))
}

#[tauri::command]
pub fn add_goal(
    app: AppHandle,
    store: tauri::State<'_, StoreState>,
    catalog: tauri::State<'_, CatalogState>,
    resources: tauri::State<'_, ResourcesState>,
    target: ipc::TargetKey,
) -> Result<ipc::PlanView, IpcError> {
    // The target must exist in the catalog: a goal for a made-up id doesn't get saved.
    // Without a catalog it can't be verified, and the UI needs to be able to say
    // "install the game" instead of "this item doesn't exist".
    let resources = resources.get();
    let c = resources.and_then(|rs| catalog.get_or_build(rs));
    match c {
        None => return Err(IpcError::CatalogUnavailable),
        Some(c) if !ipc::target_exists(c, &target) => return Err(IpcError::UnknownTarget),
        Some(_) => {}
    }
    let since_epoch = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let created_unix = since_epoch.as_secs() as i64;
    let goal = ipc::Goal {
        id: ipc::GoalId::new(created_unix, goal_nonce(since_epoch.subsec_nanos())),
        target,
        created_unix,
        note: None,
    };
    let guard = store.lock(&app).map_err(store_unavailable)?;
    guard.add_goal(&goal).map_err(store_error)?;
    // The goals and the queue are one screen: one event for both.
    announce(&app, PLAN_CHANGED);
    let read = guard.goals().map_err(store_error)?;
    Ok(ipc::plan_view(
        c,
        wiki::Dataset::embedded().ok(),
        read.goals,
        read.unreadable,
        None,
        icon_url,
    ))
}

#[tauri::command]
pub fn remove_goal(
    app: AppHandle,
    store: tauri::State<'_, StoreState>,
    catalog: tauri::State<'_, CatalogState>,
    resources: tauri::State<'_, ResourcesState>,
    id: ipc::GoalId,
) -> Result<ipc::PlanView, IpcError> {
    // Same as `plan`: the view returned carries the remaining goals, and naming them
    // needs the catalog.
    let resources = resources.get();
    let c = resources.and_then(|rs| catalog.get_or_build(rs));
    let guard = store.lock(&app).map_err(store_unavailable)?;
    // Idempotent: removing an id that's already gone isn't an error.
    let _removed = guard.remove_goal(&id).map_err(store_error)?;
    announce(&app, PLAN_CHANGED);
    let read = guard.goals().map_err(store_error)?;
    Ok(ipc::plan_view(
        c,
        wiki::Dataset::embedded().ok(),
        read.goals,
        read.unreadable,
        None,
        icon_url,
    ))
}

/// The entropy a goal id is hashed from, drawn here because `ipc` reads no clock (card #81, V2):
/// the clock's nanoseconds, the pid, the address of a fresh allocation, and a counter, so two
/// goals added within one nanosecond of each other still differ.
fn goal_nonce(nanos: u32) -> u64 {
    use std::hash::{Hash, Hasher};
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let mut h = std::collections::hash_map::DefaultHasher::new();
    nanos.hash(&mut h);
    std::process::id().hash(&mut h);
    let probe = Box::new(0u8);
    (&*probe as *const u8 as usize).hash(&mut h);
    COUNTER.fetch_add(1, Ordering::Relaxed).hash(&mut h);
    h.finish()
}
