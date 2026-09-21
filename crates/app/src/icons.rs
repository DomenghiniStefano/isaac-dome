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
    let trim = reference.trims_to_drawing();
    let png = match &reference {
        // The one reference that is a picture of several: the widget's paper with the
        // symbols the profile has earned laid on it, at the offsets the anm2 declares.
        ipc::IconRef::Widget { fills } => app
            .state::<MarkFramesState>()
            .get(rs)
            .and_then(|frames| ipc::widget_source(fills, frames))
            .and_then(|art| widget_bytes(rs, &art)),
        ipc::IconRef::Mark { column, tier } => app
            .state::<MarkFramesState>()
            .get(rs)
            .and_then(|frames| ipc::mark_source(*column, *tier, frames))
            .and_then(|sprite| sprite_bytes(rs, &sprite, trim)),
        ipc::IconRef::Achievement { .. }
        | ipc::IconRef::Item { .. }
        | ipc::IconRef::Head { .. }
        | ipc::IconRef::Page { .. }
        | ipc::IconRef::Room { .. } => app
            .state::<CatalogState>()
            .get_or_build(rs)
            .and_then(|c| ipc::icon_source(c, &reference).cloned())
            .and_then(|sprite| sprite_bytes(rs, &sprite, trim)),
    };
    let Some(png) = png else {
        return no_icon(404);
    };
    let mut r = tauri::http::Response::new(png);
    r.headers_mut().insert(
        tauri::http::header::CONTENT_TYPE,
        tauri::http::HeaderValue::from_static("image/png"),
    );
    r
}

/// The file a sprite names, cropped when it names a piece of a sheet, and shrunk to its own
/// drawing when the reference asks for that (`IconRef::trims_to_drawing`).
///
/// **A trim that fails keeps the crop.** The picture is then drawn where the game's rectangle
/// puts it, which is what every build before this one did: degrade, never fail.
fn sprite_bytes(rs: &ResourceSet, sprite: &catalog::SpriteRef, trim: bool) -> Option<Vec<u8>> {
    let file = rs.read(&sprite.path)?;
    let png = match sprite.rect {
        None => file,
        Some(r) => ipc::crop_png(&file, r.x, r.y, r.w, r.h)?,
    };
    if !trim {
        return Some(png);
    }
    Some(ipc::trim_opaque(&png).unwrap_or(png))
}

/// The widget, drawn: the paper first, then every mark over it.
///
/// **A mark that can't be read is left out, and the paper is not.** Losing one symbol costs
/// one column of the emblem; losing the paper costs the picture, and `ipc::overlay` says the
/// same thing about its own two arguments. The rest of this file's rule holds too — nothing
/// here guesses, and every failure is a 404 the band draws as "no picture".
fn widget_bytes(rs: &ResourceSet, art: &ipc::WidgetArt) -> Option<Vec<u8>> {
    let paper = sprite_bytes(rs, &art.paper, false)?;
    let marks: Vec<(Vec<u8>, i32, i32)> = art
        .marks
        .iter()
        .filter_map(|(sprite, x, y)| Some((sprite_bytes(rs, sprite, false)?, *x, *y)))
        .collect();
    let pieces: Vec<(&[u8], i32, i32)> = marks
        .iter()
        .map(|(png, x, y)| (png.as_slice(), *x, *y))
        .collect();
    ipc::overlay(&paper, &pieces)
}
