//! ipc — view-models for the UI. Pure logic: no I/O, no dependency on Tauri.

mod catalog_view;
mod collection;
mod error;
pub mod for_tests;
mod goals;
mod graph;
mod icon;
mod mark_art;
mod marks;
mod profile;
mod progress;
mod queue;
mod reasons;
mod resources;
mod search;
mod settings;
mod sprite_png;
mod summary;
mod target_sprite;
mod wiki;
mod wiki_target;

pub use catalog_view::{catalog_view, item_views, CatalogView, ItemKindView, ItemView, KindCounts};
pub use collection::{
    collection_view, CollectionDiagnostic, CollectionItem, CollectionTotals, CollectionView,
    LockView,
};
pub use error::IpcError;
pub use goals::{target_exists, Goal, GoalId, TargetKey, UnlockTarget};
pub use graph::{
    next_steps, plan_view, resolve_target, target_of, unlock_view, AchievementRef, GoalView,
    GraphInfo, MarkColumnView, MarkLevelView, NextSteps, OriginView, PlanDiagnostic, PlanExpansion,
    PlanStep, PlanView, RequirementView, StepsBasis, UnlockDiagnostic, UnlockNode, UnlockTotals,
    UnlockView, STEPS,
};
pub use icon::{icon_source, IconRef, MarkTier, ICON_SCHEME};
pub use mark_art::{mark_source, MarkFrames, LOBBY_ANM2, WIDGET_ANM2};
pub use marks::{
    character_for, counter_index, marks_matrix, Cell, CharacterGroup, CharacterRow, MarkArtView,
    MarksMatrix, MarksTotals, BOSSES, CHARACTERS, CHARACTER_KEYS,
};
pub use profile::{
    candidates, profile_id, resolve_active, setup_state, ActiveProfile, CandidateSource,
    CandidateView, ChoiceReason, GameView, MissingReason, ProfileId, SetupDiagnostic, SetupState,
    SteamView,
};
pub use progress::SaveProgress;
pub use queue::{
    achievement_unlocking, queue_view, GraphDeps, QueueDiagnostic, QueueInputs, QueueRow, QueueView,
};
pub use reasons::{IoReason, SaveReason, SettingsReason, StoreReason};
pub use resources::{
    archive_views, data_url, extraction_report, ArchiveMode, ArchiveView, ExtractionReport,
    SpriteView,
};
pub use search::{
    search, SaveFlags, SearchDiagnostic, SearchHit, SearchIndex, SearchMatch, SearchView,
};
pub use settings::{snap_percent, Settings, DEFAULT_SCALE, SCALE_PERCENTS};
pub use sprite_png::{crop_png, decode_rgba};
pub use summary::{save_summary, SaveDiagnostic, SaveSummary, SectionCount};
pub use target_sprite::{target_sprite, TargetSprite};
pub use wiki::{
    rfc3339_to_unix, wiki_index, wiki_info, Block, Dlc, Entry, Infobox, Inline, ListItem,
    PatchView, Section, SectionKind, Style, Target, WikiCounts, WikiIndex, WikiInfo,
    WikiMissingReason, WikiPageRef,
};
