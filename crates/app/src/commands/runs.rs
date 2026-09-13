//! The run archive, read out of the store. Wiring: the fold happened when the log was read.

use tauri::AppHandle;

use ipc::{IpcError, RunSource, RunsDiagnostic, RunsInputs, RunsView};
use store::SourceKind;

use crate::state::*;

#[tauri::command]
pub(crate) fn runs(
    app: AppHandle,
    store: tauri::State<'_, StoreState>,
    catalog: tauri::State<'_, CatalogState>,
    resources: tauri::State<'_, ResourcesState>,
    archive: tauri::State<'_, ArchiveState>,
) -> Result<RunsView, IpcError> {
    let rs = resources.get();
    let catalog = rs.and_then(|rs| catalog.get_or_build(rs));
    let mut diagnostics = Vec::new();
    let mut sources = Vec::new();

    match store.lock(&app) {
        Ok(guard) => {
            let version = archive.rules().version();
            match guard.sources() {
                Ok(rows) => {
                    let mut unreadable = 0;
                    for row in rows {
                        let name = match (row.kind, &row.key) {
                            (SourceKind::Session, Some(n)) => {
                                RunSource::Session { name: n.clone() }
                            }
                            // A session row with no name cannot be told from a launch, and the
                            // launch is the honest reading: it is the source with no name.
                            (SourceKind::Session, None) | (SourceKind::Log, _) => RunSource::Live,
                        };
                        match guard.cached_runs(row.id, version) {
                            Ok(Some(runs)) => sources.push((name, runs)),
                            // No cache under these rules is not an error: the source will be
                            // folded again the next time its log is read.
                            Ok(None) => {}
                            Err(_) => unreadable += 1,
                        }
                    }
                    if unreadable > 0 {
                        diagnostics.push(RunsDiagnostic::UnreadableEvents { count: unreadable });
                    }
                }
                Err(e) => diagnostics.push(RunsDiagnostic::StoreUnavailable {
                    reason: (&e).into(),
                }),
            }
        }
        Err(reason) => diagnostics.push(RunsDiagnostic::StoreUnavailable { reason }),
    }

    Ok(ipc::runs_view(RunsInputs {
        sources,
        catalog,
        diagnostics,
    }))
}
