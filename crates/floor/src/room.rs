use serde::{Deserialize, Serialize};

/// The room kinds a painted minimap can carry. Closed on purpose: a kind the grid cannot
/// name is a kind no rule may quietly treat as "normal".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RoomKind {
    Start,
    Normal,
    Boss,
    Treasure,
    Shop,
    Curse,
    Challenge,
    Sacrifice,
    Arcade,
    Library,
    Miniboss,
    Secret,
    SuperSecret,
    UltraSecret,
}

/// One variant, deliberately. The spec's decision 4 is "1x1 now, shapes later", and a
/// one-variant enum is what makes "later" a compile error instead of a rewrite.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Shape {
    Single,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cell {
    Empty,
    Room { kind: RoomKind, shape: Shape },
}
