//! The `isaac://` icon protocol: the URL a row carries, and the bytes it resolves to.
//!
//! This is wiring by construction — it reads the game's archives and answers an HTTP
//! request — which is why it lives in the Tauri crate and not in `ipc`. The *shape* of a
//! reference (`ipc::IconRef`, `ICON_SCHEME`, the crop) is decided in the pure crate; what
//! is decided here is the one thing a pure crate cannot know.

use tauri::{AppHandle, Manager};
use unpack::ResourceSet;

use crate::state::{CatalogState, MarkFramesState, ResourcesState};

/// The URL the webview can actually fetch for an icon.
///
/// Rows carry one of these instead of a base64 image. On the real profile that took the
/// `unlock` payload from about 7 MB to under half of one, and it hands the lazy loading,
/// the caching and the de-duplication to the browser instead of to code we would have to
/// write and test in the UI.
///
/// **The platform difference lives here and nowhere else.** On Windows the webview never
/// sees a custom scheme: Tauri rewrites it to an `http://<scheme>.localhost/` origin.
/// Getting this wrong raises nothing at all — the picture simply never appears — which is
/// exactly the failure mode `unpack` already taught us to distrust.
pub(crate) fn icon_url(r: &ipc::IconRef) -> Option<String> {
    let path = r.to_path();
    Some(if cfg!(windows) {
        format!("http://{}.localhost/{path}", ipc::ICON_SCHEME)
    } else {
        format!("{}://localhost/{path}", ipc::ICON_SCHEME)
    })
}

/// A response with no body. Built without `?` or `unwrap`: this runs on data a webview
/// handed us, and a malformed request must degrade to "no image", never kill the process.
fn no_icon(status: u16) -> tauri::http::Response<Vec<u8>> {
    let mut r = tauri::http::Response::new(Vec::new());
    if let Ok(s) = tauri::http::StatusCode::from_u16(status) {
        *r.status_mut() = s;
    }
    r
}

/// Serves one icon: parse the reference, find which piece of which file it is, read it.
///
/// Every step answers 404 rather than guessing. Achievements and items are whole files;
/// the completion marks and the co-op menu heads are cells of a sheet, so their sprite
/// carries a `rect` and the answer is the crop.
pub(crate) fn icon_bytes(app: &AppHandle, path: &str) -> tauri::http::Response<Vec<u8>> {
    let Some(reference) = ipc::IconRef::parse(path) else {
        return no_icon(400);
    };
    let resources = app.state::<ResourcesState>();
    let Some(rs) = resources.get() else {
        // The game isn't installed: expected, not an error worth logging.
        return no_icon(404);
    };
    let sprite = match reference {
        ipc::IconRef::Mark { column, tier } => app
            .state::<MarkFramesState>()
            .get(rs)
            .and_then(|frames| ipc::mark_source(column, tier, frames)),
        ipc::IconRef::Achievement { .. }
        | ipc::IconRef::Item { .. }
        | ipc::IconRef::Head { .. }
        | ipc::IconRef::Page { .. } => app
            .state::<CatalogState>()
            .get_or_build(rs)
            .and_then(|c| ipc::icon_source(c, &reference).cloned()),
    };
    let Some(sprite) = sprite else {
        return no_icon(404);
    };
    let Some(png) = sprite_bytes(rs, &sprite) else {
        return no_icon(404);
    };
    let mut r = tauri::http::Response::new(png);
    r.headers_mut().insert(
        tauri::http::header::CONTENT_TYPE,
        tauri::http::HeaderValue::from_static("image/png"),
    );
    r
}

/// The file a sprite names, cropped when it names a piece of a sheet.
fn sprite_bytes(rs: &ResourceSet, sprite: &catalog::SpriteRef) -> Option<Vec<u8>> {
    let file = rs.read(&sprite.path)?;
    match sprite.rect {
        None => Some(file),
        Some(r) => ipc::crop_png(&file, r.x, r.y, r.w, r.h),
    }
}
