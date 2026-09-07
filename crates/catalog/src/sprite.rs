//! Where an image lives. `catalog` never reads pixels: it produces logical paths and crops.

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

/// A whole file (`rect: None`) or a crop of a sheet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpriteRef {
    /// Logical path without root or archive: `unpack::ResourceSet` resolves it.
    pub path: String,
    pub rect: Option<Rect>,
}

impl SpriteRef {
    pub fn whole(path: String) -> SpriteRef {
        SpriteRef { path, rect: None }
    }
}
