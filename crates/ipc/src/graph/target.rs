//! A goal key resolved against the current catalog, and a catalog edge as the UI sees it.

use catalog::{BossId, Catalog, ChallengeId, CharacterId, ItemId, Unlock};
use wiki::Dataset;

use crate::goals::{TargetKey, UnlockTarget};
use crate::icon::IconRef;
use crate::target_sprite::BossKeys;
use crate::wiki_target::{self, page_of};

/// A saved key as the UI sees it: name resolved against the current catalog, and an
/// icon if the sprite can be extracted. `None` when the catalog no longer knows the
/// key: this is the one place that decides whether a goal resolves.
pub fn resolve_target(
    c: &Catalog,
    bosses: &BossKeys,
    key: &TargetKey,
    dataset: Option<&Dataset>,
    icon: &mut impl FnMut(&IconRef) -> Option<String>,
) -> Option<UnlockTarget> {
    let english = catalog::Language::English;
    let mut resolved = crate::goals::Resolved::default();
    match *key {
        TargetKey::Item { item_kind: k, id } => {
            let i = c.item(k, ItemId(id))?;
            resolved.name = c.text(&i.name, english).to_string();
            resolved.icon_url = icon(&IconRef::Item { kind: k, id });
            resolved.page = page_of(dataset, wiki_target::item(i));
        }
        TargetKey::Character { id } => {
            let ch = c.character(CharacterId(id))?;
            resolved.name = c.text(&ch.name, english).to_string();
            // The base and Tainted forms carry the same name key: without the flag the two
            // go out as one character (`docs/BACKLOG.md` B28).
            resolved.tainted = ch.tainted;
            resolved.page = page_of(dataset, wiki_target::character(ch));
        }
        TargetKey::Boss { id } => {
            let b = c.boss(BossId(id))?;
            resolved.name = b.name.clone();
            // A row `boss_keys` leaves without a key names no page: `None`, never a guessed
            // variant (`wiki_target::boss`).
            resolved.page = wiki_target::boss(bosses, b).and_then(|t| page_of(dataset, t));
        }
        TargetKey::Challenge { id } => {
            let ch = c.challenge(ChallengeId(id))?;
            resolved.rewards = ch.rewards.iter().map(|a| a.0).collect();
            resolved.name = ch.name.clone();
            resolved.page = page_of(dataset, wiki_target::challenge(ch));
        }
    }
    Some(key.view(resolved))
}

/// A catalog edge as the UI sees it. The edge is born from the catalog that resolves it
/// (`build_unlocks` constructs it straight from the entities themselves), so the key is
/// always there; if one day it weren't, the row stays nameless instead of vanishing.
pub fn target_of(
    c: &Catalog,
    bosses: &BossKeys,
    u: &Unlock,
    dataset: Option<&Dataset>,
    icon: &mut impl FnMut(&IconRef) -> Option<String>,
) -> UnlockTarget {
    let key = key_of(u);
    resolve_target(c, bosses, &key, dataset, icon)
        .unwrap_or_else(|| key.view(crate::goals::Resolved::default()))
}

/// The key of a catalog edge. The catalog's ids are newtypes; at the boundary they aren't.
fn key_of(u: &Unlock) -> TargetKey {
    match *u {
        Unlock::Item { kind, id } => TargetKey::Item {
            item_kind: kind,
            id: id.0,
        },
        Unlock::Character { id } => TargetKey::Character { id: id.0 },
        Unlock::Boss { id } => TargetKey::Boss { id: id.0 },
        Unlock::Challenge { id } => TargetKey::Challenge { id: id.0 },
    }
}
