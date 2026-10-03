//! The `isaac://` icon protocol: the URL a row carries, and the bytes it resolves to.
//!
//! This is wiring by construction — it reads the game's archives and answers an HTTP
//! request — which is why it lives in the Tauri crate and not in `ipc`. The *shape* of a
//! reference (`ipc::IconRef`, `ICON_SCHEME`, the crop) is decided in the pure crate; what
//! is decided here is the one thing a pure crate cannot know.

use tauri::{AppHandle, Manager};
use unpack::ResourceSet;

use crate::state::{
    catalog_now, AchievementBackingState, CatalogState, MarkFramesState, ResourcesState,
};

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
    let Some(rs) = resources.get(app) else {
        // The game isn't installed: expected, not an error worth logging.
        return no_icon(404);
    };
    let placement = reference.placement();
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
            .and_then(|frames| {
                let column = *ipc::MarkColumnView::ALL.get(*column)?;
                ipc::mark_source(column, *tier, frames)
            })
            .and_then(|sprite| sprite_bytes(rs, &sprite, placement)),
        // The stand-in for a picture that did not resolve: a file of the game named by the
        // boundary itself, so there is no catalog row to look it up in.
        ipc::IconRef::Unknown => sprite_bytes(rs, &ipc::unknown_source(), placement),
        ipc::IconRef::Achievement { .. }
        | ipc::IconRef::Item { .. }
        | ipc::IconRef::Head { .. }
        | ipc::IconRef::Page { .. }
        | ipc::IconRef::Entity { .. }
        | ipc::IconRef::Room { .. } => catalog_icon(app, &resources, rs, &reference),
    };
    // An achievement's drawing never stands bare: the game shows it on the popup's paper.
    let png = match png {
        Some(drawing) if reference.is_achievement_drawing() => {
            Some(achievement_on_paper(app, rs, drawing))
        }
        other => other,
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

/// A reference the catalog resolves: a sprite it names, or an entity's `.anm2` composed here.
fn catalog_icon(
    app: &AppHandle,
    resources: &ResourcesState,
    rs: &ResourceSet,
    reference: &ipc::IconRef,
) -> Option<Vec<u8>> {
    let placement = reference.placement();
    // The catalog is built from the same archives as `rs`: `ResourcesState` opens them once.
    let state = app.state::<CatalogState>();
    let catalog = catalog_now(app, resources, &state)?;
    let bosses = state.bosses(Some(catalog));
    let dataset = wiki::Dataset::embedded().ok();
    match ipc::icon_source(catalog, bosses, dataset, reference)? {
        ipc::IconSource::Sprite(sprite) => sprite_bytes(rs, sprite, placement),
        // A non-boss entity's own picture: not a sheet crop the catalog names, but a document
        // (its `.anm2`) whose layers are composed at request time — reading 1337 rows' files
        // at startup for pictures most sessions never open is exactly what
        // `catalog::Entity`'s own doc comment says not to do.
        ipc::IconSource::Entity { anm2_path } => {
            entity_bytes(rs, anm2_path).map(|png| ipc::place(png, placement))
        }
        // The portrait if the archives hold it, the boss's own `.anm2` if they don't.
        ipc::IconSource::Portrait {
            portrait,
            otherwise,
        } => sprite_bytes(rs, portrait, placement)
            .or_else(|| entity_bytes(rs, otherwise?).map(|png| ipc::place(png, placement))),
    }
}

/// An achievement's drawing on the unlock popup's paper, where the popup's anm2 rests it. A
/// paper that cannot be read leaves the drawing alone (`ipc::on_paper`).
fn achievement_on_paper(app: &AppHandle, rs: &ResourceSet, drawing: Vec<u8>) -> Vec<u8> {
    let backing = app.state::<AchievementBackingState>();
    let Some(backing) = backing.get(rs) else {
        return drawing;
    };
    let paper = sprite_bytes(rs, &backing.paper, ipc::Placement::AsDeclared);
    ipc::on_paper(paper.as_deref(), drawing, backing.drawing_at)
}

/// The file a sprite names, cropped when it names a piece of a sheet, with its drawing placed
/// where the reference says (`IconRef::placement`). A placement that fails keeps the crop
/// (`ipc::place`): degrade, never fail.
fn sprite_bytes(
    rs: &ResourceSet,
    sprite: &catalog::SpriteRef,
    placement: ipc::Placement,
) -> Option<Vec<u8>> {
    let file = rs.read(&sprite.path)?;
    let png = match sprite.rect {
        None => file,
        Some(r) => ipc::crop_png(&file, r.x, r.y, r.w, r.h)?,
    };
    Some(ipc::place(png, placement))
}

/// A non-boss entity's own picture: its `.anm2`'s default animation, one piece per layer,
/// laid out on a blank canvas sized to what they draw (`ipc::compose_entity_art`).
///
/// **Every piece is read, and a missing one is left out — the canvas is not.** A monster
/// missing an overlay layer is still mostly itself; a monster with none of its layers is no
/// picture at all, the same distinction `widget_bytes` draws between a mark and the paper.
fn entity_bytes(rs: &ResourceSet, anm2_path: &str) -> Option<Vec<u8>> {
    let doc = rs.read(anm2_path)?;
    let frames = catalog::anm2_frames(&doc)?;
    let default_animation = catalog::anm2_default_animation(&doc)?;
    let art = ipc::compose_entity_art(anm2_path, &frames, &default_animation)?;
    let pieces: Vec<(Vec<u8>, i32, i32)> = art
        .layers
        .iter()
        .filter_map(|(sprite, x, y)| {
            Some((
                sprite_bytes(rs, sprite, ipc::Placement::AsDeclared)?,
                *x,
                *y,
            ))
        })
        .collect();
    if pieces.is_empty() {
        return None;
    }
    let refs: Vec<(&[u8], i32, i32)> = pieces
        .iter()
        .map(|(p, x, y)| (p.as_slice(), *x, *y))
        .collect();
    let canvas = ipc::blank_canvas(art.width, art.height)?;
    ipc::overlay(&canvas, &refs)
}

/// The widget, drawn: the paper first, then every mark over it.
///
/// **A mark that can't be read is left out, and the paper is not.** Losing one symbol costs
/// one column of the emblem; losing the paper costs the picture, and `ipc::overlay` says the
/// same thing about its own two arguments. The rest of this file's rule holds too — nothing
/// here guesses, and every failure is a 404 the band draws as "no picture".
fn widget_bytes(rs: &ResourceSet, art: &ipc::WidgetArt) -> Option<Vec<u8>> {
    let paper = sprite_bytes(rs, &art.paper, ipc::Placement::AsDeclared)?;
    let marks: Vec<(Vec<u8>, i32, i32)> = art
        .marks
        .iter()
        .filter_map(|(sprite, x, y)| {
            Some((
                sprite_bytes(rs, sprite, ipc::Placement::AsDeclared)?,
                *x,
                *y,
            ))
        })
        .collect();
    let pieces: Vec<(&[u8], i32, i32)> = marks
        .iter()
        .map(|(png, x, y)| (png.as_slice(), *x, *y))
        .collect();
    let composed = ipc::overlay(&paper, &pieces)?;
    // Centred **after** composing, never before: the marks are already on the paper at the
    // offsets the anm2 gave, so moving the finished picture moves all twelve pieces together
    // and none of them relative to another. The game's crop leaves the sheet up against its
    // own left edge with eleven empty pixels on the right, which in a square frame reads as a
    // picture nobody centred — see `centre_opaque` for why it is not a trim.
    Some(ipc::centre_opaque(&composed).unwrap_or(composed))
}
