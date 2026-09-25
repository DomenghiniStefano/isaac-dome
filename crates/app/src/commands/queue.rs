//! The plan queue: the only screen that writes, and the five commands behind it.

use tauri::AppHandle;

use ipc::{GraphDeps, IpcError};
use store::{store_error, store_unavailable};

use crate::events::{announce, PLAN_CHANGED};
use crate::icons::icon_url;

use crate::state::{
    achievement_flags, catalog_now, CatalogState, GraphState, ResourcesState, StoreState,
};
use catalog::Catalog;
/// What every queue command needs, gathered once so the five read the same way.
struct QueuePieces<'a> {
    catalog: Option<&'a Catalog>,
    bosses: &'a ipc::BossKeys,
    graph: Option<&'a graph::build::Graph>,
    flags: Option<Vec<bool>>,
}

fn queue_pieces<'a>(
    app: &AppHandle,
    catalog: &'a CatalogState,
    resources: &'a ResourcesState,
    graph: &'a GraphState,
) -> Result<QueuePieces<'a>, IpcError> {
    let c = catalog_now(app, resources, catalog);
    Ok(QueuePieces {
        catalog: c,
        bosses: catalog.bosses(c),
        graph: c.and_then(|c| graph.get(c)),
        flags: achievement_flags(app)?,
    })
}

/// Reads the queue and turns it into the view. Never writes: the goals import is its own
/// command, precisely so that a read stays a read.
fn queue_view_now(
    app: &AppHandle,
    store: &StoreState,
    pieces: &QueuePieces<'_>,
) -> Result<ipc::QueueView, IpcError> {
    let (queue, goals_pending, reason) = match store.lock(app) {
        Ok(guard) => {
            let read = guard.queue();
            let queued: std::collections::BTreeSet<graph::AchievementId> = match &read {
                Ok(Ok(q)) => q.rows().iter().map(|r| r.achievement).collect(),
                _ => std::collections::BTreeSet::new(),
            };
            // A goal counts as pending while nothing in the queue stands for it. Without a
            // catalog we can't tell, and claiming zero would be a guess: none are reported.
            let pending = match (guard.goals(), pieces.catalog) {
                (Ok(goals), Some(c)) => ipc::goals_pending(c, &goals.goals, &queued),
                _ => 0,
            };
            match read {
                Ok(inner) => (inner, pending, None),
                Err(e) => (Ok(plan::Queue::default()), pending, Some((&e).into())),
            }
        }
        Err(reason) => (Ok(plan::Queue::default()), 0, Some(reason)),
    };
    Ok(ipc::queue_view(
        ipc::QueueInputs {
            catalog: pieces.catalog,
            bosses: pieces.bosses,
            dataset: wiki::Dataset::embedded().ok(),
            flags: pieces.flags.as_deref(),
            graph: pieces.graph,
            eval: None,
            progress: None,
            queue: queue.as_ref(),
            goals_pending,
            store_reason: reason,
        },
        icon_url,
    ))
}

/// Reads the queue, hands it to the edit, writes it back. The only function here that
/// writes, so "a read never writes" has exactly one place to check.
fn queue_mutate(
    app: &AppHandle,
    store: &StoreState,
    pieces: &QueuePieces<'_>,
    edit: impl FnOnce(&mut plan::Queue, &graph::build::Graph, Option<&[bool]>),
) -> Result<(), IpcError> {
    let Some(g) = pieces.graph else {
        // No catalog, no graph, no way to keep the order honest: the queue is left exactly
        // as it is rather than reordered against nothing.
        return Err(IpcError::CatalogUnavailable);
    };
    let guard = store
        .lock(app)
        .map_err(|reason| IpcError::StoreUnavailable { reason })?;
    // A document that won't parse must not be silently replaced by an edited empty one:
    // editing would destroy a plan written by a version that knew more than this one.
    let mut q = match guard.queue() {
        Ok(Ok(q)) => q,
        Ok(Err(_)) => return Err(store_unavailable(ipc::StoreReason::QueueUnparseable)),
        Err(e) => return Err(store_error(e)),
    };
    edit(&mut q, g, pieces.flags.as_deref());
    guard.set_queue(&q).map_err(store_error)?;
    // Every write to the queue passes through here, so every write tells the other windows:
    // announcing at the four call sites instead would be four chances to forget one.
    announce(app, PLAN_CHANGED);
    Ok(())
}

/// The ids a move has to reason about: what is already queued, plus what is about to be.
fn ids_for(
    q: &plan::Queue,
    achievement: graph::AchievementId,
    chain: &[graph::AchievementId],
) -> Vec<graph::AchievementId> {
    let mut ids: Vec<graph::AchievementId> = q.rows().iter().map(|r| r.achievement).collect();
    ids.push(achievement);
    ids.extend(chain.iter().copied());
    ids
}

#[tauri::command]
pub fn queue(
    app: AppHandle,
    store: tauri::State<'_, StoreState>,
    catalog: tauri::State<'_, CatalogState>,
    resources: tauri::State<'_, ResourcesState>,
    graph: tauri::State<'_, GraphState>,
) -> Result<ipc::QueueView, IpcError> {
    let pieces = queue_pieces(&app, &catalog, &resources, &graph)?;
    queue_view_now(&app, &store, &pieces)
}

#[tauri::command]
pub fn queue_add(
    app: AppHandle,
    achievement: u32,
    store: tauri::State<'_, StoreState>,
    catalog: tauri::State<'_, CatalogState>,
    resources: tauri::State<'_, ResourcesState>,
    graph: tauri::State<'_, GraphState>,
) -> Result<ipc::QueueView, IpcError> {
    // The IPC speaks in bare numbers; the queue in typed ids (card #81, V12).
    let achievement = graph::AchievementId(achievement);
    let pieces = queue_pieces(&app, &catalog, &resources, &graph)?;
    queue_mutate(&app, &store, &pieces, |q, g, flags| {
        let chain = g.missing_chain(achievement, &graph::evaluate::FlagsOnly(flags));
        let deps = GraphDeps::new(g, flags, &ids_for(q, achievement, &chain));
        q.enqueue(achievement, &chain, &deps);
    })?;
    queue_view_now(&app, &store, &pieces)
}

#[tauri::command]
pub fn queue_remove(
    app: AppHandle,
    achievement: u32,
    store: tauri::State<'_, StoreState>,
    catalog: tauri::State<'_, CatalogState>,
    resources: tauri::State<'_, ResourcesState>,
    graph: tauri::State<'_, GraphState>,
) -> Result<ipc::QueueView, IpcError> {
    let achievement = graph::AchievementId(achievement);
    let pieces = queue_pieces(&app, &catalog, &resources, &graph)?;
    queue_mutate(&app, &store, &pieces, |q, _g, _flags| q.remove(achievement))?;
    queue_view_now(&app, &store, &pieces)
}

#[tauri::command]
pub fn queue_move(
    app: AppHandle,
    achievement: u32,
    after: Option<u32>,
    store: tauri::State<'_, StoreState>,
    catalog: tauri::State<'_, CatalogState>,
    resources: tauri::State<'_, ResourcesState>,
    graph: tauri::State<'_, GraphState>,
) -> Result<ipc::QueueView, IpcError> {
    let pieces = queue_pieces(&app, &catalog, &resources, &graph)?;
    queue_mutate(&app, &store, &pieces, |q, g, flags| {
        let ids: Vec<graph::AchievementId> = q.rows().iter().map(|r| r.achievement).collect();
        let deps = GraphDeps::new(g, flags, &ids);
        // The row the drop landed under, not an index: the view the screen drew leaves
        // completed and unresolved rows out, so its positions are not the document's.
        q.move_after(
            graph::AchievementId(achievement),
            after.map(graph::AchievementId),
            &deps,
        );
    })?;
    queue_view_now(&app, &store, &pieces)
}

/// The one-off move from the old goals table. A goal is a target; the queue holds
/// achievements, so each target is resolved to the achievement that unlocks it. A target
/// nothing unlocks is skipped rather than guessed at, and the goals table is left
/// untouched — so the step is repeatable and reversible.
#[tauri::command]
pub fn queue_import_goals(
    app: AppHandle,
    store: tauri::State<'_, StoreState>,
    catalog: tauri::State<'_, CatalogState>,
    resources: tauri::State<'_, ResourcesState>,
    graph: tauri::State<'_, GraphState>,
) -> Result<ipc::QueueView, IpcError> {
    let pieces = queue_pieces(&app, &catalog, &resources, &graph)?;
    let Some(c) = pieces.catalog else {
        return Err(IpcError::CatalogUnavailable);
    };
    // Both arms used to answer with the same sentence, which threw away which of the two
    // had happened. The reason is a variant now, so each says what it actually knows.
    let targets: Vec<ipc::TargetKey> = match store.lock(&app) {
        Ok(guard) => match guard.goals() {
            Ok(read) => read.goals.into_iter().map(|g| g.target).collect(),
            Err(e) => return Err(store_error(e)),
        },
        Err(reason) => return Err(store_unavailable(reason)),
    };
    queue_mutate(&app, &store, &pieces, |q, g, flags| {
        for target in &targets {
            let Some(achievement) = ipc::achievement_unlocking(c, target).map(graph::AchievementId)
            else {
                continue;
            };
            let chain = g.missing_chain(achievement, &graph::evaluate::FlagsOnly(flags));
            let deps = GraphDeps::new(g, flags, &ids_for(q, achievement, &chain));
            q.enqueue(achievement, &chain, &deps);
        }
    })?;
    queue_view_now(&app, &store, &pieces)
}
